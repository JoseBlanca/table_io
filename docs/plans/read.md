# The plan of the reading of an xlsx

Written on 28 September 2026. It builds the whole of `docs/specs/read.md`,
"xlsx_rs: an xlsx read into cells": the library crate that reads the first
visible sheet of an xlsx into its cells, the binding crate that exports
`readXlsx`, and the package `js/xlsx_rs` that popnei_web installs, up to
the package packed as a release would be. The release itself is not in
the plan: it is made by hand, on the owner's order, once the repository
is on GitHub (`docs/architecture.md`, section 5). State: draft, for the
owner's approval. The breakdown was approved by the owner on 28 September
2026. Branch `plan/read`, report `docs/reports/read.md`.

## In and out

In: every rule, refusal and case of the spec, and every check of its "How
it is verified" that runs in xlsx_rs.

Out:

- The release, its tag and its notes: step 5 of the work, by hand, on the
  owner's order.
- What popnei_web does with the package, and its tests: its stage-4
  plan, `docs/plans/individuals-pca.md` there, work package 9, which
  waits for the release and copies `excel_en.xlsx`, `encrypted.xlsx` and
  `individuals_10000.xlsx` from `tests/data/`.
- Writing an xlsx: popnei_web's stage 6, and a spec of its own.

The spec has no open point left. What is not there yet is the owner's
eight files of "Made by the owner". Each has its test written in task 4.3
and marked `#[ignore = "waits for tests/data/<file>, made by the owner"]`.
When one arrives while the plan runs, the orchestrator adds a task to
take the `#[ignore]` off and assert the cells the owner says the file
shows; what it gives that the spec does not expect is a finding for the
spec, and a stop for the owner. Until `excel_en.xlsx` and
`encrypted.xlsx` exist, the test of the package reads
`tests/data/written.xlsx` and a CSV in their place, as the spec says.

## What has to be in place

Checked on 28 September 2026 on the owner's Mac, on commit 10f25b1:

- `rustc --version` gives 1.98.0, `rustup target list --installed` lists
  `wasm32-unknown-unknown`, and `wasm-bindgen --version` gives 0.2.128.
- `node --version` gives v26.8.2 and `npm --version` 11.19.1.
- calamine 0.36.1, with its default features off, and wasm-bindgen
  0.2.128 build for `wasm32-unknown-unknown` in release: a crate of trial
  in `tmp/wasm_trial/` that opens an xlsx with calamine built to a wasm of
  507,723 bytes.
- calamine 0.36.1 and rust_xlsxwriter 0.99.1 are in the cargo registry,
  `~/.cargo/registry/src/`.

On 10f25b1 there is no `Cargo.toml` and no `js/xlsx_rs/`, so every check
below fails there: cargo says it could not find `Cargo.toml`, and npm has
no package to build. From work package 1 on, every check of the `coding`
skill runs.

## 1. The whole path, with little in it

**What it gives:** popnei_web's light worker could load the package and
read an xlsx of text, numbers and booleans at any place of its first
visible sheet, and is refused a CSV named `.xlsx`, an old `.xls`, a file
calamine cannot open and a first sheet with no value.

**Deliverables:**

1. The workspace builds and passes the checks. Check: the six commands of
   the `coding` skill, "Before the work is called done", pass, the last
   `cd js/xlsx_rs && npm run build && npm test`.
2. The sheet read. Check: `cargo test -p xlsx_rs --test sheet -- --list`
   lists at least 5 tests, and they pass: the rows of the spec's table
   "Written by the tests" for the table at A1, the table at C3, the row
   with its second cell not written, the hidden first sheet, and the first
   sheet with no value.
3. The refusals of the first bytes and of calamine. Check: `cargo test -p
   xlsx_rs --test refusals -- --list` lists at least 4 tests, and they
   pass: the bytes of `id,pop\n` and the empty bytes, `NotXlsx`; the
   first 500 bytes of an xlsx, `Unreadable` with calamine's message; the
   eight bytes of a compound file followed by zeros, `OldExcel`.
4. The package, loaded under node. Check: `npm test` in `js/xlsx_rs`
   gives its `init` the bytes of the `.wasm`, reads the CSV as
   `notXlsx`, reads `tests/data/written.xlsx` with `refusal` `""` and its
   sheet's name and rectangle, and fails when `wasm/xlsx_rs.d.ts` differs
   from `js/xlsx_rs/test/xlsx_rs.d.ts`, which is the spec's declarations
   without the comments added there. That the comparison can fail is shown
   once by changing a line of the kept file, and the change is not
   committed.
5. The size. Check: the report gives `wasm/xlsx_rs_bg.wasm` and
   `wasm/xlsx_rs.js`, raw and with `gzip -9 -c | wc -c`, beside the
   295,475 and 2,962 bytes gzipped of the spec, "Its size".

**Stands on:** nothing but what is in place above.

**Tasks:**

- [ ] 1.1 The workspace: `Cargo.toml` with the two crates, the lints of
  `.claude/skills/coding/lints.toml` and the release profile of the spec,
  "Its dependencies"; `clippy.toml`; `rust-toolchain.toml`;
  `.cargo/config.toml` with the alias `wasm-check`, which checks both
  crates for `wasm32-unknown-unknown` with the warnings denied; the
  manifests of `crates/xlsx_rs` and `crates/xlsx_rs_js` with the pinned
  dependencies; `Cargo.lock`. The crates hold only what makes them build.
  Serves 1.
- [ ] 1.2 The library crate: the types of "The Rust interface",
  `read_first_sheet`, and the parts of "What it does" that deliverables 2
  and 3 test: "The sheet read" but its limit, the empty, text, number and
  boolean cells of "Each cell", and the refusals 1 to 4 and 7 of "The
  refusals". The test that writes `tests/data/written.xlsx`, in
  `crates/xlsx_rs/tests/write_fixtures.rs`, marked `#[ignore]` and run
  once by hand, and the file committed: the spec's "The package, built"
  says what it holds. Serves 2 and 3. Needs 1.1.
- [ ] 1.3 The binding crate and the package: `XlsxRead` and `readXlsx`
  of "The Rust interface"; `js/xlsx_rs/` with `package.json`, its
  `build`, a `prepack` that copies `LICENSE`, a README, the kept
  declarations and the node test of deliverable 4; the size of
  deliverable 5 in the commit message. Serves 1, 4 and 5. Needs 1.2.

**What could go wrong:** the declarations wasm-bindgen generates may not
be the spec's, which a trial crate generated with a struct of the same
fields; a difference is a finding for the spec, not an edit of the kept
file to match. Which of calamine's errors mean refusal 1, the password,
is known from its source only, `XlsxError::Password`, until the owner's
`encrypted.xlsx` exists.

## 2. Every cell but the dates, the merged ranges and the limit

**What it gives:** every kind of cell of the spec but dates and
durations, as a user of Excel sees it; a population merged over rows in
each of them; a stray value far from the table refused before the rest
of the sheet is read; and an error calamine does not know refused with
its text.

**Deliverables:**

1. The cells. Check: `cargo test -p xlsx_rs --test cells -- --list`
   lists at least 11 tests, and they pass, one for each case of the
   spec's table "Written by the tests": the number with the format `000`;
   the six formulas; `0.1 + 0.2`; the text with spaces; the text with a
   line break; the text of several fonts.
2. The merged ranges. Check: `cargo test -p xlsx_rs --test merged --
   --list` lists at least 2 tests, and they pass: the population over
   rows 2 to 4, and the name over two columns of the header.
3. The limit and the unknown error. Check: `cargo test -p xlsx_rs --test
   refusals -- --list` lists at least 7 tests, the 4 of work package 1
   and three more, which pass: A1 and XFD200, `SheetTooLarge` of 200
   rows and 16,384 columns; XFD1 and column A down to row 200,
   `SheetTooLarge` at row 123; the formula saved with `#GETTING_DATA`,
   `CellError`.

**Stands on:** work package 1.

**Tasks:**

- [ ] 2.1 The rest of "Each cell" but the dates and the durations,
  every arm of calamine's `DataRef` named as the coding skill asks, and
  refusal 5 of "The refusals". Serves 1 and 3, its last case. Needs 1.2.
- [ ] 2.2 "Merged cells". Serves 2. Can run beside 2.1: it touches the
  laying out of the rectangle and not the cell of one value, and its own
  test file.
- [ ] 2.3 The limit of "The sheet read", refusal 6: the rectangle checked
  as each cell arrives, and the read stopped at the first cell that
  passes the limit. A task of its own, since a limit checked after the
  read gives the same refusal and breaks the memory the spec promises.
  The test of XFD1 has, besides the spec's cells, a formula saved with
  `#GETTING_DATA` at A150: a read that went on past row 123 meets it and
  gives `CellError`, where the spec gives `SheetTooLarge`. The test says
  in a comment why that cell is there. Serves 3. Needs 2.1.

**What could go wrong:** a number that is not finite is text by the spec,
and rust_xlsxwriter may not write one; if it cannot, the conversion of
one value is tested alone, the one helper test the spec allows besides
the dates, and the report says so.

## 3. The dates

**What it gives:** a date as `2024-05-13`, a date with its time, a time
alone and a duration as the spec's "Each cell" gives them, in both date
systems, and a number outside Excel's range of dates as the number.

**Deliverables:**

1. The cell of a date, alone. Check: `cargo test -p xlsx_rs --lib date
   -- --list` lists at least 4 tests, and they pass: the four values of
   the 1904 system of the spec's table.
2. The dates through `read_first_sheet`. Check: `cargo test -p xlsx_rs
   --test dates -- --list` lists at least 11 tests, one for each value of
   the spec's row of dates, and they pass.

**Stands on:** work package 2.

**Tasks:**

- [ ] 3.1 The module of the dates, `crates/xlsx_rs/src/date.rs`: the
  rounding to the millisecond, the days and the milliseconds split in
  whole numbers, the parts from calamine, the numbers below 0 and after
  9999, the durations; with its tests of the 1904 system. A task and a
  commit of its own: a wrong date is a wrong cell and no failure. Serves
  1.
- [ ] 3.2 The dates in the cells of a sheet, with the date system of the
  workbook. Serves 2. Needs 3.1.

**What could go wrong:** 45425.9999999999 must give the next day, which
calamine's own parts do not, as the spec's second trial saw; it is the
case that shows the rounding is done first.

## 4. The checks that hold for any cell

**What it gives:** no damaged file ends popnei_web's light worker; the
package read under node with every kind of cell; the large file of
popnei_web's test; and the tests of the owner's files waiting for them.

**Deliverables:**

1. No file panics. Check: `cargo test -p xlsx_rs --test no_panic --
   --list` lists at least 2 tests, the copies cut short and the copies
   with a byte flipped, and they pass. That they can fail is shown once by
   a `panic!` put in `read_first_sheet` for one length, and not
   committed.
2. The package with every kind of cell. Check: `npm test` in
   `js/xlsx_rs` asserts every cell of `tests/data/written.xlsx`, the same
   as the Rust test of that file gives.
3. The large file. Check: `tests/data/individuals_10000.xlsx` is
   committed, written by an ignored test of `write_fixtures.rs`, and a
   test reads it as 10,001 rows of 20 columns.
4. The owner's files. Check: `cargo test -p xlsx_rs --test owner_files
   -- --list --ignored` lists at least 8 tests, one for each file of
   "Made by the owner" not yet in `tests/data/`, each ignored with the
   name of its file; and a test of the package for `excel_en.xlsx` and
   `encrypted.xlsx` that is skipped, with that name, while the file is
   not there.

**Stands on:** work package 3.

**Tasks:**

- [ ] 4.1 The test that no file panics, over a file written by the test
  with the kinds the spec names ("How it is verified", "No file makes it
  panic"). Serves 1.
- [ ] 4.2 The node test over every cell of `written.xlsx`, and
  `individuals_10000.xlsx` with its test. Serves 2 and 3. Can run beside
  4.1.
- [ ] 4.3 The tests of the owner's files, with the cells "Made by the
  owner" says each shows. Serves 4. Can run beside 4.1 and 4.2.

**What could go wrong:** the test of 4.1 runs one read for each byte of
its file twice; at a few milliseconds a read and a file of about 6 KB it
takes about a minute, an estimate. If it takes more than two, the file is
made smaller and the report says so. A panic inside calamine is a stop
for the owner.

## At the end

- The package packed as a release: `npm run build` and `npm pack` in
  `js/xlsx_rs` give `xlsx_rs-0.1.0.tgz`, which is not committed; unpacked
  into `tmp/`, it holds `wasm/`, `README.md`, `LICENSE` and
  `package.json`, and a node script there imports it by its path and reads
  `written.xlsx`.
- The sizes of the `.wasm` and its JavaScript, raw and gzipped, in the
  report, beside those of the spec.
- `cargo test` prints the count of ignored tests, and the report names
  the files they wait for.
- The report tells the owner what the release waits for: the GitHub
  repository, and their order.
