# The architecture of table_io

Revised on 2 October 2026, when the owner decided that xlsx_rs becomes
table_io, a library that imports a table from a CSV, a TSV or an xlsx as
typed columns and exports typed columns to a CSV or an xlsx, for
popnei_web and for Vavilov Explorer (`objectives.md`). The owner decided
the same day that table_io is one abstraction that both applications use
for these files, so that neither reads or writes them with code of its
own, and that its interface is the best one for that, popnei_web being
changed to it rather than table_io keeping what xlsx_rs exported. The
version before, of 28 September 2026, described xlsx_rs, which read the
first visible sheet of an xlsx into its cells for popnei_web; that
reading holds, and is one step of section 2 here. This document gives
the parts of table_io, what an import and an export go through, the
invariants the code keeps, the two contracts, what the rename from
xlsx_rs changes, and how a release is made. What each module does is in
its spec under `specs/`; what Vavilov Explorer needs is in its
`docs/table_io-needs.md`, and the rules popnei_web reads a table by
today, in TypeScript, which become table_io's, in popnei_web's
`docs/specs/worker/individuals.md`.

## 1. Three layers

table_io keeps the three layers xlsx_rs had, as popnei has for its own
wasm package, with the same reasons.

- **The library crate, `crates/table_io`.** Plain Rust, with no
  wasm-bindgen in it, where everything an import and an export do is.
  `cargo test` runs it natively, which is where almost every test of the
  project is. Vavilov Explorer depends on this crate.
- **The binding crate, `crates/table_io_js`.** The functions the package
  exports to JavaScript, through wasm-bindgen, the tool that makes a Rust
  function callable from JavaScript and writes the JavaScript and the
  TypeScript declarations that load and call it. Each is a thin wrapper
  over a function of the library, and copies what it gives into values
  JavaScript reads. It holds no rule of its own: an `if` about a value
  there is in the wrong crate.
- **The package, `js/table_io/`.** What wasm-bindgen generates from the
  binding crate, the `.wasm`, its JavaScript and its declarations, with a
  `package.json`, a README and the licenses, and a test under node, the
  JavaScript runtime outside the browser, of the package as it is
  released. Beside the exported functions, wasm-bindgen's JavaScript has
  an `init`, which downloads and starts the `.wasm` and which a page
  awaits before its first call.

The two crates are one cargo workspace at the root, with the lints of
popnei (`.claude/skills/coding/lints.toml`) in its `Cargo.toml`. The
layout:

```
Cargo.toml  Cargo.lock  rust-toolchain.toml  clippy.toml  .cargo/config.toml
crates/table_io/       the library crate, and its tests in tests/
crates/table_io_js/    the binding crate
js/table_io/           the package: package.json, README.md, test/
tests/data/            the files the owner makes, xlsx and CSV
docs/                  objectives, architecture, specs/, plans/, reports/
```

## 2. What an application calls

The public interface is the same for every format, and the same in Rust
and in the package. An application names no format: it gives the bytes
of a file and gets a table, or gives a table and the format it wants and
gets bytes.

- **The import** takes the bytes of one file, already in memory, the
  limits the caller accepts, and the options of a text file, the
  encoding, the separator and the decimal mark, each found or set by the
  caller. It gives the **table** or a refusal. The table holds four
  things: the column that names the individuals, the first of the file,
  as text; the other columns, each with its name, its number in the file counted from 1,
  for an xlsx its column in the sheet, its type and its values in that
  type, the missing ones marked apart from the values; how the file was
  read, its format, and for a text file the encoding, the separator, the
  decimal mark and the line of the first character that could not be
  decoded and stands as �, or for
  an xlsx the name of the sheet. No part of a table is given with a
  refusal.
- **The types** are four, the types the values have in the file, with
  Arrow's, the format of columns in memory that Parquet and polars use,
  as the guide, as the owner decided on 2 October 2026: integer, 64-bit
  integers, Arrow's `Int64`; float, 64-bit floats, `Float64`; boolean;
  and text. The import gives each column the narrowest that holds every
  value: integer when every value is a whole number within the range of
  a 64-bit integer, written without a decimal mark or an exponent in a
  text file, a whole number cell in an xlsx; float when every value is a
  number; boolean when every value is `TRUE` or `FALSE` in any case, or
  a boolean cell of an xlsx; text otherwise.
- **The conversion** of a column to another of the four types, called
  when the user changes it, reads the values by the rules of the import, with the
  decimal mark the import used, and gives the new column or, when some
  value does not convert, how many do not and the first of them with its
  row. The rules of a value, whether a text is missing and the number,
  the whole number or the boolean it holds, are public too.
- **The export** takes a table in the same model, the format and the
  choices of the user, and gives the bytes of a new file or a refusal
  (section 4).

The format of a file is found from its bytes, not from its name: a zip,
which every xlsx is, is read as a workbook, a file of the old Office as
the refusal of an old `.xls` or of a password, and anything else as text.
So a CSV saved with the name `.xlsx`, which popnei_web refuses today with
words that ask the user to rename it, is read as the CSV it is, and the
table says it was read as text.

What a column means is the caller's, built on these types. Whether a
column is categorical, a classification of populations, is the choice
of each application, by its own rule, and so are its levels, their order
and the codes of its rows: Vavilov Explorer makes a categorical column
of a text or an integer column with a repeated value and fewer than 20
distinct values, by the owner's rule of its `table_io-needs.md`, section
3, and warns of a column of few whole numbers that may be codes of
populations; popnei_web works out the binary, continuous and categorical
columns its analyses need, and which value of a binary column is the
case. Neither of them is in table_io. The option not taken was a
categorical type in table_io with Vavilov Explorer's rule, which would
have put one application's rule of meaning into the reading both
share.

## 3. An import, from the bytes to the table

An import goes through three steps, each a module of the library with a
spec of its own; the last two are the same code for every format.

### The cells, by the rules of the format

The first step gives the rows of **cells** of the file, each row with the
place it came from, a line of the file or a row of the sheet, so that a
refusal can name it where the user will look for it.

**A text file, CSV or TSV**, in the module `csv`. The bytes are decoded
by popnei_web's rules ("The bytes and the encoding"): UTF-16 after its
mark, UTF-8 after its own, otherwise UTF-8 when the bytes are valid
UTF-8 and Windows-1252, what Excel on Windows writes for "CSV" in
Spanish, when they are not; a file with a byte 0 that is not UTF-16 is
not text. The separator is found among the tab, `;` and `,` by counting
the cells each gives the rows, and the text is split into lines and cells
by the quotes of RFC 4180, the standard of CSV. Every cell of a text
file is text, with its spaces at the ends removed.

**An xlsx**, in the module `xlsx`, as xlsx_rs read it, by
`specs/read.md`:

1. The first bytes say what the file is. Every file of Office before
   2007, an `.xls` among them, starts with the same eight bytes, and so
   does an xlsx saved with a password, which Excel encrypts inside such a
   file; every zip starts with `PK`.
2. table_io opens the zip itself and reads every part to its end, so that
   the zip checks each part's checksum, applying in that reading every
   bound of `specs/read.md`, "What xlsx_rs reads before calamine": the
   bytes unzipped, the size of each part calamine, the Rust library that
   reads the workbook, holds whole, the texts of each table of texts,
   the merged ranges, the paths of the sheets. It takes the date system
   of the workbook, and lets go of its zip before calamine opens the same
   bytes. The sheet read is the first worksheet in the order of the tabs
   that is not hidden.
3. calamine gives the cells of that sheet one by one, and each becomes
   one of four kinds of cell: empty, text, number or boolean. A date and
   an error of Excel, such as `#N/A`, become text, the date written as
   `2024-05-13`. The rectangle of the values, from the first row and
   column with a value to the last ones, is checked against the limit as
   each cell arrives.
4. Each cell of a merged range inside the rectangle takes the value of
   the range's first cell, and the cells are laid out row after row, the
   ones not in the file empty.

### The table, by the rules both applications share

The second step, the module `table`, makes the rows of cells into the
table, by popnei_web's rules ("The rows and the cells" and "The xlsx"):
a blank row is skipped; the header is the first row that is not blank;
the empty cells at its end whose columns hold no value are dropped; the
first column names the individuals, as text, and is never missing, a
row without a name and a name in two rows being refused; every other
cell is a **value**: missing, when it is empty, `NA` or `-`, and in an
xlsx one of the seven errors of Excel, `#N/A`, `#DIV/0!`, `#NAME?`,
`#NULL!`, `#NUM!`, `#REF!` and `#VALUE!`; a text; or, from an xlsx, a number
or a boolean. A refusal is found here in the order popnei_web's spec
gives when a file has several problems, and names its place by the first
step's line or row. A quote never closed is found by the module of a
text file, as it splits the text; a row of the wrong length, which only a
text file can have, and an error of Excel in the header, which only an
xlsx can have, are found by the table step, in the order of the refusals
of `specs/import.md`, which puts them among its rules.

The rules of a value, in the module `value`, are what the table step, the
types and the conversion use: whether a text is missing, the number a
text holds with a given decimal mark, by popnei_web's rule of what a
number is, the whole number and the boolean it holds, and the text of a
value, a number of an xlsx written as JavaScript writes it, `1`, `1.5`,
`0.30000000000000004`, so that a number 1 and a text `1` in one column
are one value. The decimal mark of an xlsx is the point, since a number
of an xlsx is a number already.

### The types

The third step, the module `types`, works out the facts of each column
but the first, guesses its type, and gives its values in that type
(section 2). The conversion of a column is in the same module.

## 4. An export, from the table to the bytes

An export takes a table in the model of section 2 and the choices of the
user, and gives the bytes of a new file or a refusal (Vavilov Explorer's
`table_io-needs.md`, section 6).

- **A CSV**, in the module `csv`: the separator, the decimal mark, the
  encoding, UTF-8 with or without its mark or Windows-1252, and the text
  of a missing value, empty or `NA`, each set by the caller, every
  combination accepted, a comma as both separator and decimal mark
  among them. A float is written in the shortest form that reads back as
  the same float, and a cell is quoted by RFC 4180 when it holds the
  separator, a quote or a line break.
- **An xlsx**, in the module `xlsx`, with rust_xlsxwriter: one sheet, the
  names in the first row, a number as a number cell, a boolean as a
  boolean cell, a text as a text cell, a missing value as an empty
  cell. A categorical column of the application is given as text.
- **A refusal** names the column and the row of a value that would not
  read back as itself: a character Windows-1252 does not have, and a text
  that the import would take for missing, empty, `NA` or `-`, whatever
  the text of a missing value is.

An export and an import of each format are tested together: a table
exported with each choice of the CSV and as an xlsx, and imported again,
gives its names, values and types back, but where the values of a column
read as a narrower type than the one it was written with: a text column
whose every value is a number comes back integer or float, and one of
`TRUE` and `FALSE` boolean; and a float column whose every value is
whole comes back integer from an xlsx, which keeps the number and not
how it was written. The export spec gives the full list.

## 5. Each format a feature

The library has two cargo features, `csv` and `xlsx`, both on by
default, each with the import and the export of its format; the modules
`table`, `value` and `types` are not behind a feature. A file of a format
whose feature is left out is refused as such. calamine, the zip crate,
quick-xml and rust_xlsxwriter are dependencies of the feature `xlsx`
only, so that a build with `csv` alone compiles none of them; the import
and the export of a CSV need no dependency. Vavilov Explorer takes the
defaults.

The package is built with both features, so that popnei_web reads a CSV
and an xlsx with it, and exports what the binding crate exports: the
import, the conversion and the rules of a value. The export joins the
package when popnei_web exports a table, since rust_xlsxwriter, its
writer of xlsx, was measured at about 0.35 MB gzipped in a `.wasm` that
exported it (popnei_web's `docs/technology.md`, 24 September 2026), and
would be downloaded by every user of popnei_web who loads a table. The
`.wasm` holds only what the exported functions reach, since the linker
leaves the rest out.

Every user of popnei_web who loads a table, a CSV included, then
downloads the package, where today a user with a CSV downloads nothing.
The package of xlsx_rs was 0.30 MB gzipped, measured on 29 September
2026, and a user of popnei_web downloads 0.71 MB of popnei's own wasm
already. What the import of a text file and the types add to it has not
been measured; it is measured on the first build of the package
(section 14).

## 6. What the code keeps, in every module

- **Nothing larger than the caller's limits.** The largest number of
  bytes and of cells an import accepts are given by the caller with each
  import, since popnei_web accepts 20 MB and 2,000,000 cells, and
  Vavilov Explorer, which runs natively with more memory, would set
  larger ones that it has not chosen. A sheet is checked against the
  limit of cells as each cell arrives, so that a stray value far from the
  table is refused when it is read; the whole sheet is never built at
  once: calamine's `worksheet_range`, which builds the sheet in memory
  before table_io can look at its size, is not called.
- **No panic.** In WebAssembly a panic is a trap, which ends popnei_web's
  light worker, the thread of its tab that reads the user's files. In
  Vavilov Explorer, whose release build is compiled to stop the program
  at a panic (`panic = "abort"` in its `Cargo.toml`), it ends the
  application, and the backend cannot catch it, as `catch_unwind` would
  in a build that unwinds. So the lints deny what panics outside the
  tests, and every way the input can be wrong is a refusal or an error
  (section 7). calamine's code cannot be read line by line for this; a
  file found to make it panic is a finding for the spec and an issue for
  calamine.
- **One thread, no clock, no file system, no network.** The library reads
  bytes it is given and returns bytes it writes, and calls nothing of the
  host, so it runs the same in a browser, in node and natively.
- **Nothing of either application's own choices.** table_io gives the
  kind of each refusal and its data, not its words; takes the limits
  from the caller; and has no default of an export, which is the
  application's.

## 7. What fails, and how it reaches the caller

An import ends in one of three ways.

- **A refusal**, a value: a file the user can mend or replace, each kind
  with what its words need, the line, or the row and the column of the
  sheet, the separator used, the names involved, the sheet. The kinds
  are those of popnei_web's `IndividualsFileError`, its list of the ways
  a file of individuals is refused, that are about the file and not about
  the browser. It is a value and not an error so that its fields reach
  the application as fields and not as text to be taken apart again.
- **An error**: a zip calamine cannot open as a workbook or whose sheet
  it cannot read, a file cut short or damaged, a file past one of the
  bounds table_io checks before calamine. It carries the message of the
  zip crate, of calamine or of table_io, whichever failed, which the
  application writes where whoever reports the problem finds it, and
  shows as a file that could not be read.
- **The table.**

An export ends in the bytes or a refusal, and a conversion in the column
or the count of the values that do not convert. In Rust each is a
`Result`. In the package a refusal and an unreadable file are both
values JavaScript reads, the second with its message, and a JavaScript
`Error` is thrown only for a defect of the caller, an option that is not
one of the package's (`specs/package.md`). No type of calamine or of
rust_xlsxwriter is in the public interface, so that an application does
not depend on which version of either is inside.

## 8. The two contracts

table_io has two users, and each is written against a part of it that
does not change unseen.

### popnei_web, through the package

popnei_web's light worker imports the package, awaits its `init`, calls
the import with the bytes of the user's file, its options and its limits,
reads what it gives and frees what wasm holds. What it reads is what
wasm-bindgen declares, which the spec of the package gives line by line:
those declarations are the contract.

- The first release of table_io declares the interface of section 2 and
  nothing of xlsx_rs's `readXlsx`, by the owner's decision of 2 October
  2026. popnei_web is changed to it from its own session, when the owner
  decides, and until then stays on xlsx_rs's release `js-v0.1.0-dev.1`,
  which works as before. What popnei_web changes then is in section 9.
- A change of the declarations after that, a field, a name, a kind of
  refusal, is a new release here and, in the same piece of work, a change
  of the code of popnei_web that reads them and a new URL in its
  `package.json`. Here the declarations are kept in git, in
  `js/table_io/test/table_io.d.ts`, and the test of the package fails
  when what wasm-bindgen generates differs from them.
- A change inside, a value read differently, is a new release too, and
  popnei_web takes it when it changes the URL. What a user sees change
  goes into the notes of the release.
- While the two are changed together, popnei_web installs the local
  build of the package, packed with `npm pack` in `js/table_io` and
  installed there with `npm install --no-save` and the absolute path of
  the `.tgz`, which changes neither its `package.json` nor its lockfile.
  Not a link: popnei_web's development server refuses to serve a `.wasm`
  through one, "403 Forbidden", as its review of 28 September 2026 saw.
  What popnei_web commits is always a release.

### Vavilov Explorer, through the Rust interface

Vavilov Explorer depends on the library crate by git, at a revision of
this repository it pins, as its owner decided (`table_io-needs.md`,
section 8), with every feature, and builds it natively with its own
backend. What it is written against is the public Rust interface of the
library, which the specs give, and its compiler checks it when it moves
its pin. So:

- the revisions it pins are commits of `main` that are pushed, since it
  fetches them from GitHub, and the repository is public so that its
  build needs no token;
- a change of the public interface is a change in Vavilov Explorer in the
  same piece of work, made from its own session when it moves its pin,
  and the commit here says what changed for a caller;
- a change inside, a value read differently, is named in its commit
  message for whoever moves the pin, since Vavilov Explorer has no notes
  of a release to read.

One thing is asked of Vavilov Explorer's own `Cargo.toml`, from its own
session: `[profile.dev.package.calamine] overflow-checks = false`, so
that its development build does not stop at a panic of calamine on a
damaged xlsx that its release build and table_io's tests pass through
(section 11).

What Vavilov Explorer's `table_io-needs.md` asked of table_io and is now
its own, by the owner's decision of 2 October 2026 on the types: the
categorical type, the guess of which columns are categorical, their
levels in alphabetical order and the code of each row, and the warning
of a column of few whole numbers. Its import makes them from the integer
and text columns table_io gives, and its export gives a categorical
column to table_io as text.

## 9. What the rename changes

- **The crates.** `crates/xlsx_rs` becomes `crates/table_io`, and
  `crates/xlsx_rs_js` becomes `crates/table_io_js`, moved with `git mv` so
  that their history follows them. The reading of the cells of an xlsx
  becomes the module `xlsx`, private to the library, since an
  application calls the import.
- **The version.** The workspace goes from `0.1.0` to `0.2.0`, so that no
  tag or file of table_io repeats one of xlsx_rs's: its first package is
  `table_io-0.2.0.tgz`, on the tag `js-v0.2.0-dev.1`.
- **The package.** Named `table_io`, in `js/table_io/`, its files
  `wasm/table_io.js`, `wasm/table_io_bg.wasm` and `wasm/table_io.d.ts`,
  built with `--out-name table_io`, with the declarations of section 2.
- **The repository on GitHub**, `github.com/JoseBlanca/xlsx_rs`, which
  the owner renames `github.com/JoseBlanca/table_io` themselves, as they
  decided on 2 October 2026. The address of `origin` in this checkout is
  then set to the new one. GitHub's page "Renaming a repository", read
  on 2 October 2026, says that everything at the old address but a
  GitHub Pages site is sent on to the new one, issues and `git clone`
  and `git fetch` among them, and does not name the files of a release.
  popnei_web installs `js-v0.1.0-dev.1` by its old URL,
  `https://github.com/JoseBlanca/xlsx_rs/releases/download/js-v0.1.0-dev.1/xlsx_rs-0.1.0.tgz`,
  so right after the rename that URL is downloaded and its hash compared
  with the one in popnei_web's `package-lock.json`. If it fails,
  popnei_web's `npm ci` fails until its `package.json` names the release
  at its new address, `github.com/JoseBlanca/table_io/releases/...`, the
  same file with the same hash, a change of one line there. A new
  repository named `xlsx_rs` under the owner's account would end the
  sending on, and only that account can make one.
- **The release and its URL.** Releases are made as before (section 10),
  with the new names:
  `https://github.com/JoseBlanca/table_io/releases/download/js-v0.2.0-dev.1/table_io-0.2.0.tgz`
  for the first. None is made without the owner's order.
- **What popnei_web changes to move to table_io**, when the owner decides
  it, from popnei_web's own session, written down here and not done
  from here: the URL in its `package.json`; the name of the package in
  the one line of its light worker that imports it, and the rule of its
  lint that lets only that file import it; the package loaded for every
  table and not only for an xlsx; its reader of a text file in
  TypeScript, `src/worker/individuals/`, and its inference of the types,
  replaced by the import; its four types replaced by the four of table_io,
  with the roles its analyses need worked out from them (section 2), in
  its project, its project file and its screens; the words of the
  refusals that change, the CSV renamed `.xlsx` that is now read; and
  the line of the local install in its `docs/architecture.md`, section 6;
  and it keeps its check of the size of a file before it reads its bytes,
  since the package sees the bytes only once they are in the memory of
  the wasm.
  A project file popnei_web saved with its four types is opened by
  popnei_web reading the table again or mapping the types, which is
  popnei_web's to decide.
- **The documents and the skills.** `CLAUDE.md`, the skills, the
  subagents, the README of the package and the documents under `docs/`
  name table_io where they named xlsx_rs, and take the new scope where it
  changes a rule: the coding skill's "Nothing of popnei_web's rules: no
  value is made missing, no space is trimmed, no header is found" holds
  now of the step of the cells alone, since the table step is where those
  rules are. The plans and the reports of xlsx_rs stay as they were
  written, a record of that work.
- **The folder on the owner's Mac**, `/Users/jose/devel/xlsx_rs`, is the
  owner's to rename, after this work is merged, since its worktrees are
  inside it and popnei_web's line of the local install names it.

## 10. The package and its releases

The package is named `table_io`, of `"type": "module"`, and exports
`./wasm/table_io.js` with its declarations; its version is the version
of the workspace. Its `build` compiles the binding crate for
`wasm32-unknown-unknown` in release, then runs `wasm-bindgen --target
web --remove-name-section --out-name table_io` over it, which writes the
JavaScript, the declarations and `table_io_bg.wasm` into `wasm/`, which
git ignores. `npm pack` builds and tests the package before it packs it.

A release is a pre-release on GitHub, on a tag `js-v0.2.0-dev.1`, then
`dev.2` and on, with the `.tgz` attached, and popnei_web names the file
by its URL, with its hash in popnei_web's lockfile. A tag is used once and
never moved, since a file that changed under the same URL no longer
matches that hash. The repository is public, under the owner's account,
so that popnei_web's `npm ci` and Vavilov Explorer's `cargo build`
download it with no token.

Releases are made by hand, as popnei's are, and later by a workflow shared
with popnei, as the owner decided on 28 September 2026. By hand, with the
owner's order, from a commit of `main` that is pushed:

1. The checks of the `coding` skill pass on that commit.
2. `npm run build` and `npm test` in `js/table_io`.
3. The size, `gzip -9 -c` of `wasm/table_io_bg.wasm` and of
   `wasm/table_io.js`, raw and gzipped.
4. `npm pack`.
5. The tag, pushed, and the pre-release with the `.tgz`, `gh release
   create <tag> --prerelease`, whose notes say the commit, the date, Rust
   and wasm-bindgen's versions, the tests that ran and the ones that did
   not, the sizes of step 3, what changed since the release before, and
   the line that installs it.

Nothing checks that the `.tgz` of a release was built from the commit its
tag names while releases are made on the owner's Mac: the hash in
popnei_web's lockfile says only that the file of a URL never changed. A
workflow that builds the package on the tag would check it.

## 11. Dependencies and the toolchain

- **calamine 0.36.1**, pinned, with its default features off, which reads
  the xlsx. Behind the feature `xlsx`.
- **zip 8.6.0 and quick-xml 0.41.0**, pinned to the versions and the
  features calamine takes them with, so that a build holds one copy of
  each: table_io reads the zip and some of its XML itself before calamine
  (section 3). Behind the feature `xlsx`.
- **rust_xlsxwriter 0.99.1**, pinned, which the tests have used since 28
  September 2026 to write their xlsx files in memory, becomes a
  dependency of the library, behind the feature `xlsx`, to write the
  export. It takes zip 8.6.0, the copy calamine takes. Approved by the
  owner on 2 October 2026.
- **wasm-bindgen 0.2.128**, pinned, since its command line refuses a crate
  of another version; the version popnei pins. The binding crate alone.
- **Rust 1.98.0**, the stable release of 18 August 2026, named in
  `rust-toolchain.toml` with the target `wasm32-unknown-unknown`, and the
  one on the owner's Mac; Vavilov Explorer's `Cargo.toml` asks for 1.98
  too.

The import and the export of a CSV, its encodings Windows-1252 and
UTF-16 among them, need no dependency: the standard library decodes UTF-8
and UTF-16, and Windows-1252 is a table of 32 characters over the 224 it
shares with Unicode's first 256. A new dependency is pure Rust, builds
for `wasm32-unknown-unknown`, has its size in the package measured, and is
approved by the owner before it is added. An upgrade of one is a commit
of its own, with the tests and the size before and after. `Cargo.lock` is
committed.

calamine is compiled with its checks of integer overflow off in the
builds of the tests too, `[profile.dev.package.calamine] overflow-checks
= false`, as the release build that makes the package has them: calamine
adds and subtracts row and column numbers with plain operators, which
panic in a build of the tests and wrap in the package (review of work
package 2 of `plans/read.md`, 28 September 2026). A profile of a
dependency is set only by the workspace that builds it, so this line
holds in table_io's builds and not in Vavilov Explorer's: its release
build wraps as the package does, and its development build would stop
at such a panic, for a damaged xlsx, unless its own `Cargo.toml` has the
same line, which is asked of it (section 8).

The release profile is `opt-level = 3`, LTO and one codegen unit, as
popnei_web's `docs/technology.md` measured it.

## 12. How it is verified

At four levels, each named in the specs with the cases it holds.

- **cargo test, natively**, at the public functions of section 2: files
  written in memory by the tests, with rust_xlsxwriter for an xlsx and
  as literal bytes for a text file, each case a few lines that say what
  the file holds and the literal table or refusal it gives; the cases of
  popnei_web's `docs/specs/worker/individuals.md`, its tables at
  `readCsv`, `readSheet` and `cellNumber`, with their literals, where
  table_io keeps popnei_web's rule, and with the new answer where the
  owner's types or this document change it; the files the owner makes in
  Excel, LibreOffice and Google Sheets, in `tests/data/`, whose tests
  assert what the owner says each file shows; the export and the import
  again of section 4, for each format and every choice of the CSV; and
  one test that no input panics, a small xlsx and a small CSV each cut
  short at every length and with each of their bytes changed in turn,
  every copy giving a table, a refusal or an error.
- **The speed**, measured by a release build on a CSV of 100,000 rows and
  50 columns written by the test, run by hand and written in the report
  of the plan with the model of the Mac (`objectives.md`, goal 7).
- **The package under node**, as it is released: its `init` given the
  bytes of the `.wasm`, the import over the owner's files, an xlsx and a
  CSV, and the declarations it generates compared with the ones kept in
  git.
- **In the two applications**, each with its own tests when it takes a
  release or a revision. A finding there about a value is a finding for
  the spec of table_io, fixed here.

## 13. What comes later

- **The export in the package**, when popnei_web exports a table, its
  report of stage 6 among them (section 5).
- **A workflow that makes the releases**, shared with popnei.

## 14. Decided by the owner

The owner approved this document and `objectives.md` on 2 October 2026,
and decided the same day:

1. **One `.wasm` for every table.** The package holds the import of both
   formats in one `.wasm`, so a user of popnei_web with a CSV downloads
   the reader of xlsx too, almost all of the 0.30 MB of xlsx_rs's `.wasm`
   (popnei_web's `docs/technology.md` measured calamine at 0.29 MB
   gzipped in a crate of three libraries, on 24 September 2026), once,
   after which the browser keeps it. Its size is measured on the first
   build. The option not taken: two builds of the binding crate in the
   package, one with the feature `csv` alone, for a smaller download of
   a CSV, not measured, against two `.wasm` to build, test and release
   and a choice of the file's format back in popnei_web.
2. **rust_xlsxwriter 0.99.1 as a dependency of the library**, behind the
   feature `xlsx`, for the export of an xlsx (section 11).
