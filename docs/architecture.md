# The architecture of xlsx_rs

Written on 28 September 2026, before any code, from the design the owner
approved that day for popnei_web, section 6 of its
`docs/architecture.md`, "The files wasm, the package of xlsx_rs", and from
the first spec of xlsx_rs, `specs/read.md`, which popnei_web wrote before
this repository existed and which moved here whole. This document gives
the parts of xlsx_rs, what a read goes through, the invariants the code
keeps, the interface with popnei_web, and how a release is made. What
xlsx_rs is for is in `objectives.md`. The license is in section 9.

## 1. Three layers

xlsx_rs has the three layers popnei has for its wasm package, with the
same reasons.

- **The library crate, `crates/xlsx_rs`.** Plain Rust, with no
  wasm-bindgen in it, where everything the read does is: the choice of the
  sheet, the cells, the dates, the merged ranges, the refusals. `cargo
  test` runs it natively, which is where almost every test of the project
  is, since the functions wasm-bindgen generates cannot run natively.
- **The binding crate, `crates/xlsx_rs_js`.** The one exported function,
  `readXlsx`, a thin wrapper that calls the library and copies what it
  gives into `XlsxRead`, a struct whose fields JavaScript reads: the code
  of a refusal, or none, the name of the sheet, its first row and column,
  its size, and its cells row after row. It holds no rule of its own: an
  `if` about a cell there is in the wrong crate.
- **The package, `js/xlsx_rs/`.** What wasm-bindgen generates from the
  binding crate, the `.wasm`, its JavaScript and its declarations, with a
  `package.json`, a README and the license, and a test under node, the
  JavaScript runtime outside the browser, of the package as it is
  released. It has no TypeScript of its own: popnei_web calls one
  function and the `init` that loads the wasm.

The two crates are one cargo workspace at the root, with the lints of
popnei (`.claude/skills/coding/lints.toml`) in its `Cargo.toml`. The files
the owner makes by hand for the tests are in `tests/data/`. The layout:

```
Cargo.toml  Cargo.lock  rust-toolchain.toml  clippy.toml  .cargo/config.toml
crates/xlsx_rs/        the library crate, and its tests in tests/
crates/xlsx_rs_js/     the binding crate
js/xlsx_rs/            the package: package.json, README.md, test/
tests/data/            the xlsx files the owner makes
docs/                  objectives, architecture, specs/, plans/, reports/
```

## 2. A read, from the bytes to the cells

A read takes the bytes of one file, already in memory, and the largest
number of cells the caller accepts, and gives the cells of one sheet or a
refusal. It runs through these steps, whose rules are the spec's
(`specs/read.md`):

1. The first bytes say what the file is. Every file of Office before
   2007, an `.xls` among them, starts with the same eight bytes, and so
   does an xlsx saved with a password, which Excel encrypts inside such a
   file; every zip, which an xlsx is, starts with `PK`. Anything else is
   refused before calamine sees it.
2. calamine opens the zip as a workbook, reading from the bytes in memory, and
   gives the list of its sheets with whether each is hidden and whether
   it is a worksheet. The sheet read is the first worksheet in the order
   of the tabs that is not hidden.
3. calamine gives the cells of that sheet one by one, in the order of the
   file, and xlsx_rs turns each value into one of four kinds of cell:
   empty, text, number or boolean. A date and an error of Excel, such as
   `#N/A`, become text cells, the date written as `2024-05-13`. It keeps
   the cells with a value and the rectangle they span, from the first row
   and column with a value to the last ones, and stops at the first cell
   that makes the rectangle larger than the limit.
4. calamine gives the merged ranges of the sheet, and each cell of a
   range inside the rectangle takes the value of its first cell.
5. The cells are laid out row after row over the rectangle, the ones not
   in the file empty.

What the code keeps, in every module:

- **Nothing larger than the limit.** The rectangle is checked as each
  cell arrives, so a read never holds more cells than the caller allowed,
  and a stray value far from the table is refused when it is read and not
  after the rest of the sheet. The whole sheet is never built at once:
  calamine can also give a sheet as one rectangle built in memory, its
  function `worksheet_range`, which xlsx_rs does not use, since it would
  hold the rectangle before xlsx_rs could look at its size.
- **No panic.** In WebAssembly a panic is a trap, which ends popnei_web's
  light worker, the thread of its tab that reads the files of the user.
  So the lints deny what panics outside the tests, and every way the
  input can be wrong is a refusal or an error (section 3). calamine's own
  code cannot be read line by line for this, and a panic inside it ends
  the worker all the same; a file found to do so is a finding for the
  spec and an issue for calamine.
- **One thread, no clock, no file system, no network.** The library reads
  bytes it is given and calls nothing of the host, so it runs the same in
  a browser, in node and natively.
- **Nothing of popnei_web.** xlsx_rs gives what the file holds. Which
  cells are missing values, what the header is, the types of the columns
  and the words a user reads are popnei_web's (`objectives.md`, "Non
  goals").

## 3. What fails, and how it reaches the caller

A read ends in one of three ways.

- **A refusal**, a value: the file is not an xlsx, is an old `.xls`, has
  a password, its sheet is empty, holds an error calamine does not know,
  or is larger than the limit. Each carries what popnei_web needs to
  word it, the name of the sheet, the text of the error, the rectangle
  reached, and it is a value and not an error so that those fields cross
  as fields and not as text to be taken apart again.
- **An error**, thrown: a zip calamine cannot open as a workbook, or
  whose sheet it cannot read, a file cut short or damaged. It carries
  calamine's message, which popnei_web writes to the console for the one
  who reports it, and shows as a file that could not be read.
- **The cells.**

In Rust the library returns a `Result` whose error is either a refusal,
of its enum `Refusal`, or the message of a file it cannot read; the binding crate
turns the first into the field `refusal` of what it returns, and the
second into a JavaScript `Error` with the message. No type of calamine is
in the public interface of the library, so that a user of the crate does
not depend on which version of calamine is inside.

## 4. The contract with popnei_web

popnei_web's light worker imports the package the first time a user
loads an xlsx, awaits its `init`, calls `readXlsx` with the bytes and the
limit, reads the fields of the `XlsxRead` it gets and frees it
(popnei_web's `docs/specs/worker/individuals.md`, "The package of
xlsx_rs, loaded on first need"). What it reads is what wasm-bindgen
declares in `wasm/xlsx_rs.d.ts`, which `specs/read.md` gives line by
line, "The Rust interface": that file is the contract.

- A change of those declarations, a field, a name, a code of refusal,
  is a new release here and, in the same piece of work, a change of the
  code of popnei_web that reads them and a new URL in its `package.json`.
  popnei_web checks at build time that the struct still has the fields it
  reads, so a release that drops one fails its type check. Here the
  declarations are kept in git as `js/xlsx_rs/test/xlsx_rs.d.ts`, and the
  test of the package fails when what wasm-bindgen generates differs from
  them, so a change of the contract is never made unseen.
- A change inside, a cell read differently, is a new release too, and
  popnei_web takes it when it changes the URL. What a user sees change
  goes into the notes of the release.
- While the two are changed together, popnei_web installs the local
  build of the package, packed with `npm pack` in `js/xlsx_rs` and
  installed there with `npm install --no-save` and the absolute path of
  the `.tgz`, which changes neither its `package.json` nor its lockfile.
  Not a link, `npm link` or a `file:` path: popnei_web's development
  server refuses to serve a `.wasm` through a link, "403 Forbidden", as
  its review of 28 September 2026 saw with popnei's package, and a
  relative path names no folder from a worktree. What popnei_web commits
  is always a release.

## 5. The package and its releases

The package is named `xlsx_rs`, of `"type": "module"`, and exports
`./wasm/xlsx_rs.js` with its declarations; its version is the version of
the workspace, `0.1.0` for the first. Its `build` compiles the binding
crate for `wasm32-unknown-unknown` in release, then runs `wasm-bindgen
--target web --remove-name-section --out-name xlsx_rs` over it, which
writes the JavaScript, the declarations and `xlsx_rs_bg.wasm` into
`wasm/`, which git ignores. `npm pack` makes `xlsx_rs-0.1.0.tgz`.

A release is a pre-release on GitHub, on a tag `js-v0.1.0-dev.1`, then
`dev.2` and on, with the `.tgz` attached, and popnei_web names the file by
its URL,
`https://github.com/JoseBlanca/xlsx_rs/releases/download/js-v0.1.0-dev.1/xlsx_rs-0.1.0.tgz`,
with its hash in popnei_web's lockfile. A tag is used once and never
moved, since a file that changed under the same URL no longer matches that
hash. The repository is `github.com/JoseBlanca/xlsx_rs`, public, under
the owner's account, as the owner decided on 28 September 2026, so that
popnei_web's `npm ci` downloads it with no token.

Releases are made by hand, as popnei's are, and later by a workflow shared
with popnei, as the owner decided on 28 September 2026. By hand, with the
owner's order, from a commit of `main` that is pushed:

1. The checks of the `coding` skill pass on that commit.
2. `npm run build` and `npm test` in `js/xlsx_rs`.
3. The size, `gzip -9 -c` of `wasm/xlsx_rs_bg.wasm` and of
   `wasm/xlsx_rs.js`, raw and gzipped.
4. `npm pack`.
5. The tag, pushed, and the pre-release with the `.tgz`, `gh release
   create <tag> --prerelease`, whose notes say the commit, the date, Rust
   and wasm-bindgen's versions, the tests that ran and the ones that did
   not, the sizes of step 3, what changed since the release before, and
   the line that installs it.

Nothing checks that the `.tgz` of a release was built from the commit its
tag names while releases are made on the owner's Mac: the hash in
popnei_web's lockfile says only that the file of a URL never changed. A
workflow that builds the package on the tag would check it, and is the
one popnei is to have too.

## 6. Dependencies and the toolchain

- **calamine 0.36.1**, pinned, with its default features off: `chrono`,
  dates as chrono's types, and `picture`, the images of a workbook, are
  not needed, since xlsx_rs writes a date from calamine's own parts of
  it. It brings a reader of zip files, with its compression in Rust, and
  a parser of XML, quick-xml, none with C. The library crate alone.
- **wasm-bindgen 0.2.128**, pinned, since its command line refuses a crate
  of another version; the version popnei pins, so that the one command
  line on the owner's Mac builds both. The binding crate alone.
- **rust_xlsxwriter 0.99.1**, pinned, for the tests only, which write
  their xlsx files in memory with it; the library the writer of
  popnei_web's report is to use from its stage 6.
- **Rust 1.98.0**, the stable release of 18 August 2026, named in
  `rust-toolchain.toml` with the target `wasm32-unknown-unknown`, and the
  one on the owner's Mac. calamine 0.36.1 needs 1.88 at least.

The owner approved the three crates on 28 September 2026. A new
dependency is pure Rust, builds for `wasm32-unknown-unknown`, has its size
in the package measured, and is approved by the owner before it is added.
An upgrade of one is a commit of its own, with the tests and the size
before and after. `Cargo.lock` is committed.

calamine is compiled with its checks of integer overflow off in the
builds of the tests too, `[profile.dev.package.calamine] overflow-checks
= false`, as the release build that makes the package has them: calamine
adds and subtracts row and column numbers with plain operators, which
panic in a build of the tests and wrap in the package, so without it a
test of a damaged file would stop at a panic the package does not have
(review of work package 2, 28 September 2026).

The release profile is `opt-level = 3`, LTO and one codegen unit, as
popnei_web's `docs/technology.md` measured it. With `opt-level = "z"`,
which trades speed for size, the three crates of that measurement,
calamine, rust_xlsxwriter and zip together, were 0.47 MB gzipped, and
0.58 MB with `opt-level = 3`; it has not been measured for calamine
alone.

## 7. How it is verified

At three levels, each named in the spec with the cases it holds.

- **cargo test, natively**, at the library's public function: files
  written in memory by the tests with rust_xlsxwriter, each case a few
  lines that say what the file holds and the literal cells it gives; and
  the files the owner makes in Excel, LibreOffice and Google Sheets, in
  `tests/data/`, whose tests assert what the owner says each file shows
  in Excel. A test of a file that is not there yet is marked `#[ignore]`
  with the name of the file it waits for. And one test that no file
  panics: a small xlsx cut short at every length and with each of its
  bytes changed in turn, each copy giving a sheet, a refusal or an error.
- **The package under node**, as it is released: its `init` given the
  bytes of the `.wasm`, and `readXlsx` over two of the owner's files, the
  cells and a refusal, and until those exist over a file written by the
  tests and a CSV; and the declarations it generates compared with the
  ones kept in git. It is what checks a release before it is tagged.
- **In popnei_web**, which tests its own reading of the package under
  Vitest and the Individuals step in Chromium, Firefox and WebKit with a
  few xlsx files copied from here. A finding there about a cell is a
  finding for the spec of xlsx_rs, fixed here and released.

## 8. What comes later

- **Writing an xlsx**, the report of popnei_web from its stage 6, with
  rust_xlsxwriter, which the tests already use. It is a second exported
  function and a second spec, `specs/write.md`, and it would be in the
  same package, whose size then grows by about 0.35 MB gzipped, as
  popnei_web's `docs/technology.md` measured rust_xlsxwriter on 24
  September 2026; whether the package is then split, so that a user who
  only reads does not download the writer, is decided with that spec.
- **The zip of the report**, which popnei_web recommends putting here
  and decides with its stage 6 (its `docs/architecture.md`, section 13,
  point 14).
- **A workflow that makes the releases**, shared with popnei.

## 9. The license

xlsx_rs is under the MIT license, as popnei is, which the owner decided on
28 September 2026; the text is `LICENSE` at the root, and the package
copies it before it is packed. calamine is under MIT too, and
rust_xlsxwriter, used by the tests only and not shipped, under MIT or
Apache 2.0.
