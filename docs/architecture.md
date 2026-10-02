# The architecture of table_io

Revised on 2 October 2026, when the owner decided that xlsx_rs becomes
table_io, a library that reads and writes tables for popnei_web and for
Vavilov Explorer (`objectives.md`). The version before, of 28 September
2026, described xlsx_rs, which read the first visible sheet of an xlsx
into its cells for popnei_web alone; what it said of that reading holds,
and is section 2 here. This document gives the parts of table_io, what a
read and a write go through, how the reading shared by the two
applications is kept apart from the types each gives its columns, the
invariants the code keeps, the two contracts, what the rename from
xlsx_rs changes, and how a release is made. What each module does is in
its spec under `specs/`; what Vavilov Explorer needs is in its
`docs/table_io-needs.md`, and the rules popnei_web reads a table by, which
become table_io's, in popnei_web's `docs/specs/worker/individuals.md`.

## 1. Three layers

table_io keeps the three layers xlsx_rs had, as popnei has for its own
wasm package, with the same reasons.

- **The library crate, `crates/table_io`.** Plain Rust, with no
  wasm-bindgen in it, where everything a read and a write do is. `cargo
  test` runs it natively, which is where almost every test of the
  project is. Vavilov Explorer depends on this crate.
- **The binding crate, `crates/table_io_js`.** What the package exports
  to JavaScript, through wasm-bindgen, the tool that makes a Rust
  function callable from JavaScript and writes the JavaScript and the
  TypeScript declarations that load and call it: today the one function `readXlsx`, a thin wrapper that
  calls the library and copies what it gives into `XlsxRead`, a struct
  whose fields JavaScript reads one by one, the code of a refusal or
  none, the name of the sheet, its first row and column, its size, and
  its cells row after row. It holds no rule of its own: an `if` about a
  cell there is in the wrong crate.
- **The package, `js/table_io/`.** What wasm-bindgen generates from the
  binding crate, the `.wasm`, its JavaScript and its declarations, with a
  `package.json`, a README and the licenses, and a test under node, the
  JavaScript runtime outside the browser, of the package as it is
  released. Beside the exported function, wasm-bindgen's JavaScript has
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

## 2. A read, from the bytes to the columns

A read takes the bytes of one file, already in memory, the limits the
caller accepts, and for a CSV the options the user set, and gives the
columns of the table and how the file was read, or a refusal. It goes
through three steps, each a module of the library with a spec of its
own, and the last two are the same for every format.

### The cells, by the rules of the format

The first step gives the rows of **cells** of the file, each row with the
place it came from, a line of the file or a row of the sheet, so that a
refusal can name it as the user will look for it.

**A CSV or a TSV**, in the module `csv`. The bytes are decoded into text
by popnei_web's rules ("The bytes and the encoding"): UTF-16 after its
mark, UTF-8 after its own, otherwise UTF-8 when the bytes are valid
UTF-8 and Windows-1252, what Excel on Windows writes for "CSV" in
Spanish, when they are not; a file with a byte 0 that is not UTF-16 is
not text. The separator is found among the tab, `;` and `,` by counting
the cells each gives the rows, and the text is split into lines and cells
by the quotes of RFC 4180, the standard of CSV. Every cell of a CSV is
text, with its spaces at the ends removed. The encoding, the separator
and the decimal mark can each be set by the caller instead of found, and
the read reports the three it used, with the line of the first character
it could not decode.

**An xlsx**, in the module `xlsx`, as xlsx_rs read it, by
`specs/read.md`:

1. The first bytes say what the file is. Every file of Office before
   2007, an `.xls` among them, starts with the same eight bytes, and so
   does an xlsx saved with a password, which Excel encrypts inside such a
   file; every zip, which an xlsx is, starts with `PK`. Anything else is
   refused before calamine, the Rust library that reads the workbook,
   sees it.
2. table_io opens the zip itself and reads every part to its end, so that
   the zip checks each part's checksum, applying in that reading every
   bound of `specs/read.md`, "What xlsx_rs reads before calamine": the
   bytes unzipped, the size of each part calamine holds whole, the texts
   of each table of texts, the merged ranges, the paths of the sheets.
   It takes the date system of the workbook, and lets go of its zip
   before calamine opens the same bytes. The sheet read is the first
   worksheet in the order of the tabs that is not hidden.
3. calamine gives the cells of that sheet one by one, and each becomes
   one of four kinds of cell: empty, text, number or boolean. A date and
   an error of Excel, such as `#N/A`, become text, the date written as
   `2024-05-13`. The rectangle of the values, from the first row and
   column with a value to the last ones, is checked against the limit as
   each cell arrives.
4. Each cell of a merged range inside the rectangle takes the value of
   the range's first cell, and the cells are laid out row after row, the
   ones not in the file empty.

This step is all popnei_web takes from the package today: the cells of
an xlsx, which its TypeScript makes into the table.

### The table, by the rules both applications share

The second step, the module `table`, makes the rows of cells into the
table, by popnei_web's rules ("The rows and the cells" and "The xlsx"),
the same code for the rows of every format: a blank row is skipped; the
header is the first row that is not blank; the empty cells at its end
whose columns hold no value are dropped; the first column names the
individuals, as text, and is never missing; every other cell is a
**value**: missing, when it is empty, `NA` or `-`, and in an xlsx one of
the seven errors of Excel; a text; or, from an xlsx, a number or a
boolean. A refusal of a row, a header or a name is found here, in the
order popnei_web's spec gives when a file has several problems, and names
its place by the first step's line or row. The rules that belong to one
format stay in its module: a row of the wrong length and a quote never
closed are the CSV's, an error of Excel in the header the xlsx's.

The rules of a value, in the module `value`, are what the table step and
the types both use, and what an application calls later to read a value
as the import did: whether a text is missing, the number a text holds
with a given decimal mark, by popnei_web's rule of what a number is, the
whole number and the boolean it holds, and the text of a value, a number
of an xlsx written as JavaScript writes it, `1`, `1.5`,
`0.30000000000000004`, so that a number 1 and a text `1` in one column
are one value, as popnei_web compares them. The decimal mark of an xlsx
is the point, since a number of an xlsx is a number already.

### The types, the choice that is not shared

The third step, the module `types`, gives each column but the first its
type and its values in that type. It does so in two parts, which are how
table_io keeps the reading shared by the two applications apart from the
types each gives its columns: Vavilov Explorer's `table_io-needs.md`,
section 3, asked for that line to be drawn and left where to draw it to
table_io.

The first part is shared. The **facts of a column** are worked out from
its values by the rules of a value, once: how many values it has, how
many distinct values, compared as text, whether one appears in more than
one row, whether every value is a number by the decimal mark of the
read, whether every value is a whole number within the range of a 64-bit
integer, whether every value is a boolean, and, for a column of whole
numbers, how many distinct numbers it holds and the smallest and the
largest, which the warning of a column of few whole numbers needs. These
are the facts popnei_web's inference reads too, its binary type being
two distinct values and its continuous type three or more that are all
numbers.

The second part is the choice of a type from the facts, a function of a
few lines. table_io has one set of types, Vavilov Explorer's, numeric,
integer, text, boolean and categorical, with the owner's rule of
`table_io-needs.md`, section 3: integer when every value is whole,
numeric when every value is a number, boolean when every value is `TRUE`
or `FALSE` in any case or a boolean cell, categorical when a value
repeats and the column has fewer than 20 distinct values, and text
otherwise. These are also the types of the columns the export takes
(section 3), so a table read, changed and written is in one model. The
conversion of a column to another type, which Vavilov Explorer calls
when the user changes it, is in the same module, and says when a value
does not convert how many do not and the first of them with its row.
popnei_web's four types, and its choice of which value of a binary column
is the case, stay in popnei_web: popnei_web does not read its tables
through table_io, and if it does later, its choice is a second function
over the same facts, in table_io or in popnei_web's TypeScript, decided
then.

Two other ways were weighed. **One set of types with the rules of each
application as options** would put popnei_web's binary type and its
coding of the case into table_io, where no application calls it today,
and each option doubles the cases the tests of the types have to cover.
**The facts alone, and each application choosing its types in its own
code**, would leave in Vavilov Explorer the rule the owner set and the
conversions that rest on it, and table_io could not test that a table it
writes reads back with the same types (`objectives.md`, goal 4), since
the types would not be its own. The way taken costs one set of types
that is Vavilov Explorer's in the library both applications use. It would
be the wrong one if popnei_web's types came into table_io too: the two
sets would then sit side by side, each with its own choice over the same
facts, and the first option would cost less.

What a read gives is the columns, in the order of the file, each with its
name, its number in the file counted from 1, for an xlsx its column in
the sheet, its type and its values in that type, the missing ones marked
apart from the values; and how the file was read, the encoding, the
separator, the decimal mark and the line of the first character not
decoded for a CSV, and the name of the sheet for an xlsx. No part of a
table is given with a refusal.

## 3. A write, from the columns to the bytes

A write takes the columns, in the model of the types above, and the
choices of the user, and gives the bytes of a new file or a refusal
(Vavilov Explorer's `table_io-needs.md`, section 6).

- **A CSV**, in the module `csv`: the separator, the decimal mark, the
  encoding, UTF-8 with or without its mark or Windows-1252, and the text
  of a missing value, empty or `NA`, each set by the caller, every
  combination accepted, a comma as both separator and decimal mark
  among them. A float is written in the shortest form that reads back as
  the same float, and a cell is quoted by RFC 4180 when it holds the
  separator, a quote or a line break.
- **An xlsx**, in the module `xlsx`, with rust_xlsxwriter: one sheet, the
  names in the first row, a number as a number cell, a boolean as a
  boolean cell, a text or a category as a text cell, a missing value as
  an empty cell.
- **A refusal** names the column and the row of a value that would not
  read back as itself: a character Windows-1252 does not have, and a text
  that the read would take for missing, empty, `NA` or `-`, whatever the
  text of a missing value is.

The reading and the writing of a format are tested together: a table
written with each choice of the CSV and as an xlsx, and read again,
gives its names, values and types back, but where the rule that tells a
categorical column from a text one guesses the other: a categorical
column of 20 levels or more, or with no value in two rows, comes back
text, and a text column with a repeated value and fewer than 20 distinct
values comes back categorical.

## 4. Each format a feature

The library has two cargo features, `csv` and `xlsx`, both on by
default, each with the reading and the writing of its format; the
modules `table`, `value` and `types` are not behind a feature. Vavilov
Explorer takes the defaults. The binding crate takes the library with
the feature `xlsx` alone, so that the package builds nothing of the
reading of a CSV. calamine, the zip crate, quick-xml and rust_xlsxwriter
are dependencies of the feature `xlsx` only, so that a build with `csv`
alone compiles none of them; the reading and the writing of a CSV need
no dependency. The `.wasm` holds only what the exported function calls,
since the linker leaves out the code no export reaches, so the writer of
xlsx, inside the feature the package takes, should add nothing to its
size; that is measured on the first build of the package, and if it does,
the feature `xlsx` is split into a reading and a writing one.

## 5. What the code keeps, in every module

- **Nothing larger than the caller's limits.** The largest number of
  bytes and of cells a read accepts are given by the caller with each
  read, since popnei_web accepts 20 MB and 2,000,000 cells, and Vavilov
  Explorer, which runs natively with more memory, would set larger ones
  that it has not chosen. A sheet is checked against the limit of cells
  as each cell arrives, so that a stray value far from the table is
  refused when it is read; the whole sheet is never built at once:
  calamine's `worksheet_range`, which builds the sheet in memory before
  table_io can look at its size, is not called.
- **No panic.** In WebAssembly a panic is a trap, which ends popnei_web's
  light worker, the thread of its tab that reads the user's files. In
  Vavilov Explorer, whose release build is compiled to stop the program
  at a panic (`panic = "abort"` in its `Cargo.toml`), it ends the
  application, and the backend cannot catch it, as `catch_unwind` would
  in a build that unwinds. So
  the lints deny what panics outside the tests, and every way the input
  can be wrong is a refusal or an error (section 6). calamine's code
  cannot be read line by line for this; a file found to make it panic is
  a finding for the spec and an issue for calamine.
- **One thread, no clock, no file system, no network.** The library reads
  bytes it is given and returns bytes it writes, and calls nothing of the
  host, so it runs the same in a browser, in node and natively. Vavilov
  Explorer reads the file the user picked and writes the one they export.
- **Nothing of either application's own choices.** table_io gives the
  kind of each refusal and its data, not its words; takes the limits
  from the caller; and has no default of an export, which is Vavilov
  Explorer's dialog's. popnei_web's types are popnei_web's (section 2).

## 6. What fails, and how it reaches the caller

A read ends in one of three ways.

- **A refusal**, a value: a file the user can mend or replace, each kind
  with what its words need, the line, or the row and the column of the
  sheet, the separator used, the names involved, the sheet. The kinds
  are those of popnei_web's `IndividualsFileError`, its list of the ways
  a file of individuals is refused, that are about the file and not about
  the browser, those of a CSV and those of an xlsx, and it is a value and not
  an error so that its fields reach the application as fields and not as
  text to be taken apart again.
- **An error**: a zip calamine cannot open as a workbook or whose sheet
  it cannot read, a file cut short or damaged, a file past one of the
  bounds table_io checks before calamine. It carries the message of the
  zip crate, of calamine or of table_io, whichever failed, which the
  application writes where whoever reports the problem finds it, and
  shows as a file that could not be read.
- **The table.**

In Rust a read returns a `Result` whose error is either a refusal or the
message of a file it cannot read; a write returns the bytes or a refusal
of the export. In the package, the binding crate turns a refusal into the
field `refusal` of what it returns, and the message into a JavaScript
`Error`. No type of calamine or of rust_xlsxwriter is in the public
interface, so that an application does not depend on which version of
either is inside.

## 7. The two contracts

table_io has two users, and each is written against a part of it that
does not change unseen.

### popnei_web, through the package

popnei_web's light worker imports the package the first time a user loads
an xlsx, awaits its `init`, calls `readXlsx` with the bytes and the limit,
reads the fields of the `XlsxRead` it gets and frees it (popnei_web's
`docs/specs/worker/individuals.md`, "The package of xlsx_rs, loaded on
first need"). What it reads is what wasm-bindgen declares, which
`specs/read.md` gives line by line, "The Rust interface": those
declarations are the contract.

- A change of the declarations, a field, a name, a code of refusal, is a
  new release here and, in the same piece of work, a change of the code
  of popnei_web that reads them and a new URL in its `package.json`.
  popnei_web checks at build time that the struct still has the fields it
  reads. Here the declarations are kept in git, in
  `js/table_io/test/table_io.d.ts`, and the test of the package fails
  when what wasm-bindgen generates differs from them.
- A change inside, a cell read differently, is a new release too, and
  popnei_web takes it when it changes the URL. What a user sees change
  goes into the notes of the release.
- While the two are changed together, popnei_web installs the local
  build of the package, packed with `npm pack` in `js/table_io` and
  installed there with `npm install --no-save` and the absolute path of
  the `.tgz`, which changes neither its `package.json` nor its lockfile.
  Not a link: popnei_web's development server refuses to serve a `.wasm`
  through one, "403 Forbidden", as its review of 28 September 2026 saw.
  What popnei_web commits is always a release.

popnei_web stays on xlsx_rs's release `js-v0.1.0-dev.1` until the owner
decides otherwise. The first release of table_io declares the same
`readXlsx`, `XlsxRead` and `init`, field for field, so that moving
popnei_web to it changes the name it imports and not the code that reads
the fields (section 8).

### Vavilov Explorer, through the Rust interface

Vavilov Explorer depends on the library crate by git, at a revision of
this repository it pins, as its owner decided (`table_io-needs.md`,
section 8), with every feature, and builds it natively with its own
backend. What it is written against is the public Rust interface of the
library, the types and functions each spec gives in its "The Rust
interface", and its compiler checks it when it moves its pin. So:

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
(section 10).

## 8. What the rename changes

The rename makes xlsx_rs's names table_io's, and changes nothing a user of
popnei_web sees.

- **The crates.** `crates/xlsx_rs` becomes `crates/table_io`, and
  `crates/xlsx_rs_js` becomes `crates/table_io_js`, moved with `git mv` so
  that their history follows them. The reading of an xlsx keeps its
  functions and types, inside the module `xlsx`.
- **The version.** The workspace goes from `0.1.0` to `0.2.0`, so that no
  tag or file of table_io repeats one of xlsx_rs's: its first package is
  `table_io-0.2.0.tgz`, on the tag `js-v0.2.0-dev.1`.
- **The package.** Named `table_io`, in `js/table_io/`, its files
  `wasm/table_io.js`, `wasm/table_io_bg.wasm` and `wasm/table_io.d.ts`,
  built with `--out-name table_io`. Its declarations are xlsx_rs's,
  unchanged (section 7).
- **The repository on GitHub**, `github.com/JoseBlanca/xlsx_rs`, renamed
  `github.com/JoseBlanca/table_io` if the owner agrees: nothing is
  renamed on GitHub without the owner's order. GitHub's page "Renaming a
  repository", read on 2 October 2026, says that everything at the old
  address but a GitHub Pages site is sent on to the new one, issues and
  `git clone` and `git fetch` among them, and does not name the files
  of a release. If those are sent on too, popnei_web's `npm ci` goes on
  downloading
  `https://github.com/JoseBlanca/xlsx_rs/releases/download/js-v0.1.0-dev.1/xlsx_rs-0.1.0.tgz`,
  the same file, which its lockfile's hash checks, for as long as no new
  repository of the account takes the name `xlsx_rs`. That has not been
  tried, and is checked right after the rename by downloading that URL
  and comparing its hash with the one in popnei_web's
  `package-lock.json`; if it fails, popnei_web cannot install until its
  `package.json` names a release at the new address, or the name is
  given back. Only the owner's account can make a repository at
  `github.com/JoseBlanca/xlsx_rs`, so nobody else can take the name.
  The address of `origin` in this checkout is then set to the new one.

  The other ways: a new repository `table_io`, with this history pushed
  to it and `xlsx_rs` left as it is, archived, keeps popnei_web's URL
  without relying on GitHub sending it on, and leaves the issues and the
  first release in a repository nobody works in; and keeping the name
  `xlsx_rs` on GitHub, with the crates renamed inside, keeps every URL
  and gives a repository whose name says less than it holds. The rename
  is recommended, since it keeps one repository with its history, issues
  and releases, and its one risk, the release's URL, is checked in a
  minute after it and mended by giving the name back.
- **The release and its URL.** Releases are made as before (section 9),
  with the new names:
  `https://github.com/JoseBlanca/table_io/releases/download/js-v0.2.0-dev.1/table_io-0.2.0.tgz`
  for the first. None is made without the owner's order.
- **What popnei_web would change to move to table_io**, when the owner
  decides it, from popnei_web's own session: the URL in its
  `package.json`; the name of the package in the one line of its light
  worker that imports it; the rule of its lint that lets only that file
  import the package; and the line of the local install in its
  `docs/architecture.md`, section 6. Its code that reads the fields of
  `XlsxRead` does not change.
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

## 9. The package and its releases

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

## 10. Dependencies and the toolchain

- **calamine 0.36.1**, pinned, with its default features off, which reads
  the xlsx. Behind the feature `xlsx`.
- **zip 8.6.0 and quick-xml 0.41.0**, pinned to the versions and the
  features calamine takes them with, so that a build holds one copy of
  each: table_io reads the zip and some of its XML itself before calamine
  (section 2). Behind the feature `xlsx`.
- **rust_xlsxwriter 0.99.1**, pinned, which the tests have used since 28
  September 2026 to write their xlsx files in memory, becomes a
  dependency of the library, behind the feature `xlsx`, to write the
  export. It takes zip 8.6.0, the copy calamine takes. This needs the
  owner's approval, since it was approved for the tests alone; popnei_web's
  `docs/technology.md` measured it at about 0.35 MB gzipped in a `.wasm`
  that exported it, on 24 September 2026, which the package does not
  (section 4).
- **wasm-bindgen 0.2.128**, pinned, since its command line refuses a crate
  of another version; the version popnei pins. The binding crate alone.
- **Rust 1.98.0**, the stable release of 18 August 2026, named in
  `rust-toolchain.toml` with the target `wasm32-unknown-unknown`, and the
  one on the owner's Mac; Vavilov Explorer's `Cargo.toml` asks for 1.98
  too.

The reading and the writing of a CSV, its encodings Windows-1252 and
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
same line, which is asked of it (section 7).

The release profile is `opt-level = 3`, LTO and one codegen unit, as
popnei_web's `docs/technology.md` measured it.

## 11. How it is verified

At four levels, each named in the specs with the cases it holds.

- **cargo test, natively**, at the library's public functions: files
  written in memory by the tests, with rust_xlsxwriter for an xlsx and
  as literal bytes for a CSV, each case a few lines that say what the file
  holds and the literal table or refusal it gives; the cases of
  popnei_web's `docs/specs/worker/individuals.md`, its tables at
  `readCsv`, `readSheet` and `cellNumber`, with the same literals, so
  that the two implementations of the rules agree on every case either
  spec names; the files the owner makes in Excel, LibreOffice and Google
  Sheets, in `tests/data/`, whose tests assert what the owner says each
  file shows; the round trip of section 3, for each format and every
  choice of the CSV; and one test that no input panics, a small xlsx and
  a small CSV each cut short at every length and with each of their
  bytes changed in turn, every copy giving a table, a refusal or an
  error.
- **The speed**, measured by a release build on a CSV of 100,000 rows and
  50 columns written by the test, run by hand and written in the report
  of the plan with the model of the Mac (`objectives.md`, goal 7).
- **The package under node**, as it is released: its `init` given the
  bytes of the `.wasm`, `readXlsx` over the owner's files, and the
  declarations it generates compared with the ones kept in git.
- **In the two applications**, each with its own tests when it takes a
  release or a revision: popnei_web in Vitest and in three browsers, as
  before, and Vavilov Explorer in its own. A finding there about a value
  is a finding for the spec of table_io, fixed here.

## 12. What comes later

- **popnei_web reading its CSV with table_io**, which its owner has set
  aside: every user of popnei_web would then download the package, with
  the reading of a CSV and the table step in it, whose size has not been
  measured, where a user with a CSV downloads nothing today. It would be
  a new export of the package and a change of the contract.
- **The report of popnei_web**, an xlsx, and possibly its zip, from its
  stage 6, which would export a writer from the package; decided with
  that stage.
- **A workflow that makes the releases**, shared with popnei.

## 13. The license

table_io is under the MIT license, as popnei is, which the owner decided
on 28 September 2026; the text is `LICENSE` at the root, and the package
copies it before it is packed. calamine is under MIT, and rust_xlsxwriter
under MIT or Apache 2.0. `crates/table_io/src/xlsx/attrs.rs` is a copy of
calamine's reader of attributes, `RawAttrIter`, with calamine's notice of
copyright and license above it. The notices of the crates compiled into
the `.wasm`, 29 for `js-v0.1.0-dev.1`, are in `THIRD_PARTY_LICENSES.md`
at the root, which the package copies beside `LICENSE`; all are
permissive. Vavilov Explorer, which compiles rust_xlsxwriter too, carries
the notices of what it compiles into its own application.
