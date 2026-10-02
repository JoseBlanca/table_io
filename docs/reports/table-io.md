# The report of table_io

The work report of `docs/plans/table-io.md`, carried out on the branch
`plan/table-io` from 2 October 2026: a section for each work package,
with its deliverables, what its review found and what changed. The last
section is for whoever next revises a skill or writes a plan.

## State

Under way since 2 October 2026, when the owner approved the plan. The
objectives, the architecture, the specs and the plan were put on `main`
by the owner's order the same day, at 8bb8982; nothing of the code is on
`main`, and nothing is pushed.

## Work package 1: the rename

Built on 2 October 2026 in commits 7df5adc (task 1.1), dea97e0 (task
1.2) and e1d968d (two fixes of the orchestrator), the two tasks side by
side.

The deliverables, checked by the orchestrator:

1. `cargo test -p table_io -- --list` lists 221 tests, the names of
   `cargo test -p xlsx_rs -- --list` at the start, 7c77589.
2. `cargo tree -p table_io --no-default-features --features csv -e normal`
   lists table_io alone; the builds of `csv` alone and of `xlsx` alone
   succeed.
3. `npm run build && npm test` in `js/table_io`: 13 of 13; `npm pack`
   gives `table_io-0.2.0.tgz`. The `.wasm` went from 565,301 to 565,306
   bytes, 300,707 to 300,709 gzipped, the JavaScript from 11,249 to
   11,253 bytes.
4. `git grep -n xlsx_rs` outside the plans, the reports and `read.md`
   gives, in the code, only the citations of `read.md`'s heading "What
   xlsx_rs reads before calamine", which the orchestrator put back after
   task 1.1 had renamed them, since `read.md` keeps its text; in
   `CLAUDE.md` and `.claude/`, ten lines, each saying that xlsx_rs became
   table_io, giving its history, quoting a heading of popnei_web's spec,
   or naming the folder `/Users/jose/devel/xlsx_rs` on the owner's Mac;
   in `docs/`, the sentences that say what xlsx_rs was or released.

`cargo test --workspace`: 202 passed, 19 ignored, as before the work
package.

Changed in the plan: deliverable 4 leaves the whole of `read.md` out of
the grep, not only its top, since its note of 2 October 2026 keeps its
text as approved. The root `README.md`, which no task named, was renamed
by task 1.1 and given what table_io is by the orchestrator.

The review, on 2 October 2026, at e1d968d, by the categories spec,
tests, package and architecture:

- Fixed: 15 comments, the citation of `read.md`'s heading split over two
  lines, still named it "What table_io reads before calamine", a heading
  that does not exist, and two sentences of the past gave table_io what
  xlsx_rs had done (spec); the builds of one feature passed warnings, so
  the coding skill runs clippy with warnings denied on each (architecture);
  nothing failed when a crate of the xlsx lost its `optional`, so the
  skill's checks gain `cargo tree` with `-e normal` and a grep that fails
  then (tests, which made zip not optional and saw every other check
  pass); the plan's command of deliverable 2 named zip through the
  dev-dependency rust_xlsxwriter, and now has `-e normal` (spec). Commit
  084f82b and the one after it.
- Not taken: none. The package reviewer found nothing: the `.tgz` holds
  its 8 files, was built from e1d968d byte for byte, and its test of the
  declarations fails when they change.
- For work package 6: the feature `deflate` of zip turns on
  `deflate-zopfli`, a compressor the reading never uses; its size is
  measured when the export comes (architecture reviewer).

## Work package 2: the values

Built on 2 October 2026 by one subagent, both tasks, in commits 47b8aac
(task 2.1), 4c43635 (the spec: the row of a failed conversion past
2^32 − 1), 1e684b1 (task 2.2), ed39437 and 1db2c74 (a literal of the spec
corrected, below).

The deliverables, checked by the orchestrator:

1. and 2. `cargo test -p table_io --test values`: 36 passed after the
   review, 30 before it, with the defaults and with `csv` alone; the
   file did not exist at a6a5581, where the work package started. Every
   literal of `specs/values.md`, "How it is verified", and every row of
   its table at `convert_column` is a test.
3. `npm test` in `js/table_io`: 13 of 13.

`cargo test --workspace`: 240 passed, 19 ignored.

What the owner should know: one literal of the spec was wrong, the text
of the float 2^63 in a failed conversion, written as the integer's digits
where the spec's own rule gives JavaScript's `9223372036854776000`; the
subagent asked rather than chose, and the spec was corrected (ed39437).
`float_text` was compared with node 26.8.2's `String(x)` over 3,304,228
floats by the implementer, 570,837 and 3,240,000 more by the spec
reviewer and 6,370,000 by the numbers reviewer, with no difference; it
takes about 0.65 µs a float on the owner's Mac, against 0.33 µs for
Rust's own layout, which differs from JavaScript's on 20,892 of the
3,304,228, all ties.

The review, at 1db2c74, by the categories spec, tests, numbers, and
errors with api:

- Fixed, in the spec at 5725e67 and in the code at d0b568a: two of the
  three checks of a tie held by no test, which each reviewer showed by
  removing one, all 30 tests passing and 7,613 to 58,000 floats then
  written otherwise than node writes them (spec, tests, numbers, three
  reviewers on the same point); five floats that catch each removal
  added to the spec and the tests, by their bits. Seven conversions and
  `-Infinity` without a test (spec, tests). The cap of the row of a
  failure at 4,294,967,295 missing from the doc comment (api), now said
  and held by a test on 64-bit targets. `is_empty`, which clippy asks
  for, public but not in the spec (api). A test that could not fail
  alone, removed (tests). What a text `NA` that a caller did not mark
  missing is: a value, `None` alone being missing, which an import never
  gives otherwise; written in the spec (spec, errors).
- Not taken: the third check of a tie, which skips the exact one when no
  tie is possible, can fail no test, since without it every text is the
  same and the comparison with node 2.2 s becomes 8.7 s; kept as what it
  is, a filter of speed.
- For work package 3: `cell.rs` writes `Infinity` itself where it could
  call `float_text`, the same rule in two places (spec reviewer).

## Work package 3: the whole path for an xlsx

Built on 2 October 2026 by three subagents: task 3.1 alone (8cdae39, the
spec: when the relationships name no workbook; 9fa5cdd), then 3.2
(7ed6302, the spec: a file of blank rows alone is empty, and the first
column's name is among the names compared, both as popnei_web has them;
491659f) and 3.3 (46882aa, 2a2d51d) side by side, on files kept apart.

The deliverables, checked by the orchestrator at c243a03:

1. `tests/import_format.rs`: 15 tests passed with the defaults, 17 with
   `xlsx` alone, 8 with `csv` alone.
2. and 3. `tests/import_xlsx.rs`: 73 passed, 1 ignored for the owner's
   `libreoffice.xlsx`; every row of the table of the cells of a sheet of
   `specs/import.md`, 19 tests of the order of the refusals, 20 cases of
   the guess of the type, and the property of the rows in another order
   over 300 tables made with a fixed seed.
4. `npm test` in `js/table_io`: 33 passed, 3 skipped, the cases of a CSV,
   for work package 4; the declarations generated equal
   `test/table_io.d.ts`, and name no `readXlsx`.
5. `cargo doc` lists none of xlsx_rs's types; xlsx_rs's 9 files of tests
   moved into `src/xlsx_tests/`, the 348 tests listed the same before and
   after.

`cargo test --workspace`: 328 passed, 20 ignored at c243a03.

Changed in the plan: making xlsx_rs's types private moved from task 3.1
to task 3.3, since the binding called them until 3.3 replaced it.

What the owner should know:

- The package's `.wasm` grew from 300,682 bytes gzipped to 323,876, with
  `gzip -9`, and its JavaScript from 3,042 to 6,752: the table step, the
  types and the new binding, before any reading of a text file.
- The owner's `spill.xlsx` is refused, as a header error, `#VALUE!` at
  row 1, column 1: its first row holds the error Excel saves for a spill,
  where a name of a column would be, the rule popnei_web already has.
- A short note far to the left of a table, A10 under a table at C6,
  moves the rectangle of the sheet to column A, and the file is then
  refused as an empty individual at row 7, where the user sees nothing
  wrong; popnei_web does the same, and the spec allows it (spec
  reviewer).

The review, at c243a03, by the categories spec, tests, errors with
numbers, api, architecture and package:

- Fixed in the specs at 474a9d9: the package relationships read first,
  so that a zip of other files is not a workbook whatever its other parts
  hold, where a zipped CSV of 310 MB of genotypes had been an unreadable
  file (spec); `HowRead::decimal`, the decimal mark of a table, which the
  binding had decided itself and Vavilov Explorer needs too
  (architecture); `Display` and `std::error::Error` for `ImportError`,
  for the logs and `?`, and `Default` for `TextOptions` (api); the index
  of a column in the package a number checked, since a `u32` wrapped
  modulo 2^32 and 2^32, 1.9 or `undefined` gave another column with no
  `Error` (errors, package: two reviewers); the bytes a `Uint8Array`, an
  `ArrayBuffer` being read as no bytes, which TypeScript stops; and the
  architecture's sentence on where the ragged row and the header error
  are found, which contradicted the import spec (architecture).
- Fixed in the library at 76ff71a and 3e9eb0e: the relationships first;
  the cells of a sheet split into rows while the sheet's buffer was still
  held, every slot twice, and every cell of a text file to be its own
  `String`, about 553 MB for the 10,000,000 cells of a CSV of 20 MB of
  `0,`, measured natively: the rows are now one flat list, a text cell
  borrows from the decoded text, and names are moved and not copied, the
  table step's peak on 200,000 × 10 cells of an xlsx 103.6 MB where it
  was about 108, the import's peak, 125.3 MB, being calamine's read of
  the sheet in both (architecture); the doc comments of the column of a
  text file and of the order of the refusals, and the compound file's
  mark written once (api); the names `NA` and `-`, the tabs at the ends
  of a cell, and numbers as names beyond small whole ones, untested
  (tests, each shown by a change of the code no test caught).
- Fixed in the binding at 3130e08: the index checked as a whole number
  from 0 to the last column, 2^32, 1.5, `undefined`, −1, NaN and the
  number of columns each thrown; the decimal mark taken from the
  library; a test under node of a table at C3, `namesNumber` 3 and the
  columns 4 and 5, which no test of a table at A1 could see (tests); the
  README says the bytes are a `Uint8Array`. `npm test`: 35 passed, 3
  skipped; `cargo test --workspace`: 339 passed, 20 ignored.
- Not taken: where the binding writes a place as a line or a row, from
  the format the library gives, which is a translation and not a rule
  about a cell (architecture); refusing an `ArrayBuffer` in the package,
  which would need the crate `js-sys`, a new dependency, for a mistake
  TypeScript stops (errors, package).
