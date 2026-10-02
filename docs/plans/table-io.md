# The plan of table_io

The plan that turns xlsx_rs into table_io: one import of a CSV, a TSV or
an xlsx as typed columns and one export of typed columns, for popnei_web
and Vavilov Explorer. Written on 2 October 2026 from `docs/objectives.md`
and `docs/architecture.md`, approved by the owner that day, and from the
specs `docs/specs/values.md`, `text-files.md`, `import.md`, `export.md`,
`package.md` and `read.md`, whose open points the owner answered the same
day. The owner approved the breakdown into seven work packages on 2
October 2026, and the plan is written from it. State: approved by the
owner on 2 October 2026, under way.

The work is on the branch `plan/table-io`, in the worktree
`.claude/worktrees/table-io`, which already holds the documents and the
specs, so the branch starts from them and not from `main`; the report is
`docs/reports/table-io.md`.

## In and out

In: everything the six specs give, the rename of section 9 of the
architecture, and the documents and skills renamed with it.

Out, and where it goes:

- Wiring table_io into popnei_web or Vavilov Explorer, and installing the
  package in popnei_web: their own sessions, when the owner decides
  (`docs/architecture.md`, sections 8 and 9).
- The rename of the repository on GitHub, which the owner makes; then
  the address of `origin` is set, and the old URL of `js-v0.1.0-dev.1`
  downloaded and its hash compared with popnei_web's lockfile
  (`docs/architecture.md`, section 9). A task of no work package, done
  when the owner says the rename is made; if the download fails or its
  hash differs, the owner is told at once, since popnei_web's `npm ci`
  then fails until its `package.json` names the new address.
- Any release, push or merge: the owner's order.
- The export in the package (`docs/architecture.md`, section 5).

The owner's files that are not made yet, each with the tests that wait
for them, marked `#[ignore = "waits for tests/data/<file>, made by the
owner"]`: `excel_es.csv`, `excel_es_utf8.csv`, `excel_mac.csv`,
`excel_unicode.txt` and `libreoffice.csv` (`specs/text-files.md`, "How it
is verified"), and the look in Excel of the files of an export
(`specs/export.md`), written in the report when the owner gives it.

## What has to be in place

Checked on 2 October 2026 on the owner's Mac, and checked again by the
orchestrator before the first task:

- `rustc --version` 1.98.0, `wasm-bindgen --version` 0.2.128, `node
  --version` v26.8.2.
- On the branch at ccee919, whose workspace is `main`'s again after a
  reviewer's crate of trial, `tmp/review-text`, had been taken into it: `cargo fmt --all
  --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace`, `cargo doc` and `cargo wasm-check` pass, and
  `npm test` in `js/xlsx_rs` passes 13 of 13.
- The owner's xlsx files of `specs/read.md` are in `tests/data/`.

Every check of the `coding` skill runs from work package 1 on, the
package's under `js/table_io` from the end of work package 1.

## 1. The rename

**What it gives.** The same library and package as xlsx_rs's, under the
names of table_io, so that every later work package starts from names
that will not move: popnei_web could install the package and read an
xlsx with `readXlsx` as before, imported as `table_io`.

**Deliverables.**

1. The crates `crates/table_io` and `crates/table_io_js`, the package
   `js/table_io` named `table_io`, version 0.2.0, the repository
   `github.com/JoseBlanca/table_io` in the manifests: `cargo test -p
   table_io -- --list` lists the tests `cargo test -p xlsx_rs -- --list`
   listed at the start, their count written in the report; it fails at
   the start, there being no package `table_io`.
2. The features `csv` and `xlsx`, both on by default, calamine, zip and
   quick-xml behind `xlsx`: `cargo tree -p table_io -e normal
   --no-default-features --features csv` names none of them, the
   dev-dependency rust_xlsxwriter's zip left out by `-e normal`, and `cargo build -p table_io
   --no-default-features --features csv` and `--features xlsx` both
   succeed; at the start there is no feature `csv`.
3. `npm run build && npm test` in `js/table_io` passes the 13 tests, the
   declarations kept as `test/table_io.d.ts`; `npm pack` gives
   `table_io-0.2.0.tgz`.
4. `git grep -n xlsx_rs` outside `docs/plans/`, `docs/reports/`, the
   whole of `docs/specs/read.md`, whose text is kept as the owner approved
   it, as its note of 2 October 2026 says, and the sentences of
   `docs/objectives.md`, `docs/architecture.md` and the specs that say
   xlsx_rs became table_io or what xlsx_rs released, gives nothing; the
   list of what it gives at the start is written in the report.

**What it stands on.** Nothing.

**Tasks.**

- [x] 1.1 The crates, the package and the workspace renamed with `git
  mv`, the version, the repository, the features of
  `docs/architecture.md`, section 5, `.gitignore`, the README of the
  package and `THIRD_PARTY_LICENSES.md`'s names; no rule changes.
  Deliverables 1 to 3. From `docs/architecture.md`, sections 1, 5, 9 and
  10.
- [x] 1.2 `CLAUDE.md`, the skills under `.claude/skills/` and the
  subagents under `.claude/agents/` say table_io, its two applications
  and its two contracts; the coding skill's "Nothing of popnei_web's
  rules" holds of the step of the cells alone, and its commands name
  `js/table_io`. Deliverable 4. From `docs/architecture.md`, sections 7,
  8 and 9. Can run beside 1.1.

**What could go wrong.** A path in a test of the package or a fixture of
`tests/data` that names `xlsx_rs`: deliverable 3 finds it.

## 2. The values

**What it gives.** The rules by which both applications read a value:
whether a text is missing, the whole number, the number or the boolean it
holds, the text of a float as JavaScript writes it, and the conversion of
a column, which Vavilov Explorer calls when the user changes a type.

**Deliverables.**

1. Every literal of `specs/values.md`, "How it is verified", at
   `is_missing`, `parse_integer`, `parse_float`, `parse_boolean` and
   `float_text`, in tests of `crates/table_io/tests/values.rs`: `cargo
   test -p table_io --test values` runs them, their count in the report;
   at the start there is no such test.
2. Every row of the table at `convert_column` of the same part, in the
   same file.
3. The package built and its 13 tests passing, as at the end of work
   package 1.

**What it stands on.** Work package 1.

**Tasks.**

- [x] 2.1 The module `value`: the rules of "What a text holds" and "The
  text of a value", `float_text` with ECMAScript's layout and its ties
  to the even digit. Deliverable 1.
- [x] 2.2 The module `types`: `ColumnType`, `ColumnValues`,
  `ConversionFailure` and `convert_column`, by "The conversion of a
  column", with the bound of 2^63 checked as the spec says. Deliverable
  2. Needs 2.1.

**What could go wrong.** The ties of `float_text`: a float whose exact
value is halfway between two strings of shortest digits; the literals of
the spec and of the package's test against node are what guard it.

## 3. The whole path for an xlsx

**What it gives.** popnei_web can read an xlsx through the new interface:
`importTable` of the package gives the table, its columns typed, or a
refusal, the format with it, and `convertColumn` converts a column;
xlsx_rs's `readXlsx` is gone.

**Deliverables.**

1. The format found from the first bytes, too large, a format not built,
   and not a workbook (`specs/import.md`, "The format"), each in a test
   at `import_table`, the builds of one feature among them with the
   commands of "How it is verified".
2. Every row of the table of the cells of a sheet in `specs/import.md`,
   "How it is verified", each an xlsx written by the test and its literal
   table or refusal, in `crates/table_io/tests/import_xlsx.rs`, and the
   order of the refusals of an xlsx, each pair one file can hold; the
   owner's xlsx files of `specs/read.md`, read through `import_table` as
   the tables they show.
3. The guess of the type, "The type of a column" of `specs/values.md`,
   each case as an xlsx where the format can hold it, and the property of
   the rows in another order.
4. The binding crate and the package of `specs/package.md`: `npm test`
   in `js/table_io` passes the tests of "How it is verified" that an
   xlsx can hold, `floatText` against node's `String` among them, and the
   declarations generated equal `test/table_io.d.ts`, which are those of
   the spec with their doc comments; `readXlsx` is in neither.
5. `read_first_sheet` and its types private to the library: `cargo doc`
   lists none of them.

**What it stands on.** Work package 2.

**Tasks.**

- [x] 3.1 The public types of the import, `import_table` with the format,
  too large, a format not built, and not a workbook, which reads the
  package relationships `specs/read.md` already reads. Deliverable 1. From `specs/import.md`, "The
  format" and "The Rust interface".
- [x] 3.2 The module `table`: the rows of an xlsx made the table, by "The
  cells of an xlsx, as the table takes them" and "The rows", with the
  refusals in their order, and the guess of the type. Deliverables 2 and
  3. From `specs/import.md` and `specs/values.md`, "The type of a column".
  Needs 3.1.
- [x] 3.3 The binding crate and the package, by `specs/package.md`, with
  its test under node over `tests/data/written.xlsx` and the owner's
  `excel_en.xlsx` and `encrypted.xlsx`; xlsx_rs's types made private
  once the binding no longer calls them. Deliverables 4 and 5. Needs 3.1; can
  start beside 3.2 against its types.

**What could go wrong.** The check of the relationships for not a
workbook is new code over `specs/read.md`'s reading of the parts, which
has its own bounds; the no-panic test of work package 4 covers it.

## 4. Text files

**What it gives.** popnei_web and Vavilov Explorer can import a CSV or a
TSV, with the encoding, the separator and the decimal mark found or set,
by popnei_web's rules; and no file of either format makes the library
panic.

**Deliverables.**

1. The tests of `specs/text-files.md`, "How it is verified", over the
   bytes and the encoding, at `import_table`, in
   `crates/table_io/tests/import_text.rs`, the 256 bytes of Windows-1252
   among them.
2. Every row of the table of texts of `specs/import.md`, "How it is
   verified", and the cases of `specs/text-files.md` about the lines, the
   quotes, the separator and the decimal mark, each a literal text and
   its literal table or refusal.
3. The property of the round trip of a text file of `specs/text-files.md`,
   over tables made with a fixed seed, its number of tables in the report.
4. The no-panic test of `specs/import.md`, over an xlsx and a CSV, every
   copy giving a table, a refusal or an error.
5. The tests of the owner's text files, ignored until the files are in
   `tests/data/`, and failing if switched on without them.
6. `npm test` passes the tests of `specs/package.md` that a CSV holds:
   `written.csv` read, `raggedRow` and `duplicateIndividual` of a text
   file.

**What it stands on.** Work package 3. Its last task measures the size
of the `.wasm`, raw and gzipped, for work package 6.

**Tasks.**

- [x] 4.1 The decoding of `specs/text-files.md`, "The bytes and the
  encoding": the marks, cut short, not text, Windows-1252 by its table,
  the line not decoded. Deliverable 1.
- [x] 4.2 The split and the finding of the separator and the decimal
  mark, "A variants file", "The lines, the quotes and the cells", "The
  separator", "The decimal mark", and the ragged row of the table step.
  Deliverables 2, 3 and 5. Needs 4.1.
- [x] 4.3 The no-panic test and the package's tests of a CSV. Deliverables
  4 and 6. Needs 4.2.

**What could go wrong.** The order of the checks against popnei_web's
code, which its spec did not always say: the rows of popnei_web's
`readCsv` table are the guard, and a row that differs from popnei_web's
TypeScript is a finding for the owner, not a test bent to the code.

## 5. The export of a CSV

**What it gives.** Vavilov Explorer can export a table as a CSV with any
separator, decimal mark, encoding and text of a missing value, and every
value reads back as itself or is refused with its place.

**Deliverables.**

1. The literal bytes and the refusals of `specs/export.md`, "How it is
   verified", for a CSV, at `export_table`, in
   `crates/table_io/tests/export_csv.rs`.
2. The round trip of "What reads back" for a CSV, the 36 combinations,
   over tables made with a fixed seed, their number in the report.

**What it stands on.** Work package 4.

**Tasks.**

- [x] 5.1 The types of the export, its refusals that a CSV can meet in
  their order, and the writer of a CSV. Deliverable 1.
- [x] 5.2 The round trip of a CSV. Deliverable 2. Needs 5.1.

## 6. The export of an xlsx

**What it gives.** Vavilov Explorer can export a table as an xlsx of one
sheet, `Sheet1`, that reads back as itself or is refused with its place.

**Deliverables.**

1. The tests of `specs/export.md` for an xlsx, each file read back by
   `import_table` and by calamine, and each refusal of an xlsx, in
   `crates/table_io/tests/export_xlsx.rs`; rust_xlsxwriter moved from the
   dev-dependencies to the dependencies of the feature `xlsx`.
2. The round trip of an xlsx over the same tables as work package 5.
3. The package's `.wasm`, built with the feature `xlsx` and no export
   exported, no larger than the size measured at the end of work package
   4 and written in the report then, both sizes in the report.

**What it stands on.** Work package 5.

**Tasks.**

- [x] 6.1 The writer of an xlsx and its refusals. Deliverables 1 and 3.
- [x] 6.2 The round trip of an xlsx. Deliverable 2. Needs 6.1.

## 7. The end

**What it gives.** The numbers the owner and popnei_web need before a
release, and the package ready to be released when the owner orders it.

**Deliverables.**

1. The size of `wasm/table_io_bg.wasm` and `wasm/table_io.js`, raw and
   with `gzip -9`, beside xlsx_rs's `.wasm` of `js-v0.1.0-dev.1`,
   565,045 bytes raw and 300,646 gzipped, in the report.
2. The time of `import_table` over a CSV of 100,000 rows and 50 columns
   written by a test, ignored and run by hand with `--release`, on the
   owner's Mac, its model given, in the report, against the target of
   less than one second (`objectives.md`, goal 7); a time above it is
   reported, not worked on in this plan, since the owner will review the
   speed once the library is built.
3. `THIRD_PARTY_LICENSES.md` made again for the crates the `.wasm`
   compiles, and the README of the package saying what it exports.
4. `npm pack` gives `table_io-0.2.0.tgz`, which, unpacked and imported
   under node, reads `written.csv` and `written.xlsx`.

**What it stands on.** Work package 6.

**Tasks.**

- [ ] 7.1 The size, the speed, the licenses, the README and the pack.
  Deliverables 1 to 4.

## At the end

The final check is the sum of the work packages and deliverable 4 of
work package 7. Nothing is installed in popnei_web, released, pushed or
merged: the report says what the owner is asked, at least the merge, and
what popnei_web and Vavilov Explorer change to take table_io, from
`docs/architecture.md`, sections 8 and 9.
