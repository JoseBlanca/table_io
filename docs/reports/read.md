# The report of the reading of an xlsx

The work report of `docs/plans/read.md`, carried out on the branch
`plan/read` from 28 September 2026. It is written as the work goes, one
section for each work package, for the owner, who decides the merge and
the release; its last section is for whoever next revises a skill or
writes a plan.

## State

Under way. Work package 1 is done; work package 2 is being built.
The owner is asked whether xlsx_rs may check the zip's checksums with the
`zip` crate, a new direct dependency (below, work package 1).

## Work package 1: the whole path

- 1.1, 5f7d9ac: the workspace. `cargo wasm-check` was shown to fail,
  status 101, on a dead function, so it denies warnings.
- 1.2, 567ec53: `read_first_sheet` with the plain cells and the refusals
  1 to 4 and 7. `cargo test -p xlsx_rs --test sheet` 7 passed,
  `--test refusals` 5 passed. Beyond the spec's table: a table at D2,
  where the first row and column differ, which the table at C3 cannot
  tell apart; a chart sheet before the worksheet; and "no visible
  worksheet", written as a hidden worksheet with a visible chart sheet.
  `written.xlsx` is written with a fixed creation date, so that a rerun
  gives the same bytes. The arms of calamine's values that later tasks
  fill are provisional and reached by no test: a whole number and a date
  as the number, an ISO text and an error as their text. `max_cells` is
  unused until task 2.3, under an `#[expect]` that breaks the build when
  2.3 uses it.
- 1.3, 8dfe4b7: the binding and the package. The declarations
  wasm-bindgen generated agreed with the spec's line for line; it also
  copies the doc comments and adds `initSync` and its types, and the spec
  was reworded to say the kept file is that whole file, 05cec6a.

Deliverables, checked on the last commit of the work package, 598119b:

1. The six checks of the coding skill pass.
2. `cargo test -p xlsx_rs --test sheet`: 12 passed.
3. `cargo test -p xlsx_rs --test refusals`: 9 passed.
4. `npm test` in `js/xlsx_rs`: 7 pass, 0 fail. A line of the kept
   declarations changed outside `InitOutput` fails it.
5. The size, with `gzip -9`: `xlsx_rs_bg.wasm` 508,736 bytes raw and
   280,447 gzipped, against 295,475 gzipped in the spec's trial;
   `xlsx_rs.js` 11,185 and 3,009, against 2,962.

The review, five reviewers over spec, tests, errors, api with numbers,
and package with architecture. What mattered, all fixed, 817daa4 for the
spec and acb5f74 to 598119b for the code:

- A compound file cut short made calamine's reader of compound files
  panic, at 11,234 of the 40,960 lengths of an encrypted xlsx the
  reviewer wrote: a trap that ends popnei_web's light worker. xlsx_rs now
  finds `EncryptedPackage` in the bytes itself and does not call calamine
  for a compound file; the spec's refusal 1 says so.
- A file of 5,467 bytes with a value at A1 and one at GR1048576 made the
  lay-out of 209,715,200 cells trap in the wasm. The cells are now
  reserved with `try_reserve_exact`; the limit of task 2.3 will refuse
  such a sheet first.
- The test of the package asserted only the header of `written.xlsx`: a
  number given to JavaScript as a text passed it. It now asserts every
  cell with its type, and `emptySheet` and a table at C2.
- Seven cases with no test: a very hidden first sheet, a hidden row and
  column, a formatted blank far from the values, a rectangle whose first
  column is not in its first row, an empty zip, the empty text, and the
  rectangle of a refusal passed by position in the binding.
- `npm pack` packed whatever `wasm/` held; it now builds and tests first.
  The `.wasm` held 38 paths of the owner's home folder; none now.
- The spec said calamine keeps the spaces at the ends of a text; it
  keeps them only when the file marks the text `xml:space="preserve"`,
  as Excel does.

For the owner, a question, asked in chat on 28 September 2026: calamine
stops reading a sheet at its last cell, so the checksum of the zip is
not checked, and 10 of 7,120 copies of a file with one byte changed
were read as a sheet with wrong or missing cells and no error. Checking
every checksum first needs the `zip` crate, 8.6.0, already inside the
package through calamine, as a direct dependency. Meanwhile the plan
goes on; nothing in it rests on the answer.

Measured by the reviewer of the package, the first numbers for the
spec's "How it runs": a dense sheet of 100,000 rows × 20 columns, 2,000,000
cells, half of them text, read by the package in 874 ms under node
26.8.2, the wasm's memory at 141.0 MB after it; 2,168 ms in Chromium 153
and 1,672 ms in WebKit 26.6 driven by Playwright on the owner's Mac.
Firefox would not start under Playwright there.

Not taken: the reviewer's advice to take the `# Errors` heading out of
the JSDoc wasm-bindgen copies into the declarations; the lint
`missing_errors_doc` asks for it in the Rust, and a reader of the
declarations reads it as a heading.

Issues to open once the repository is on GitHub: calamine 0.36.1 panics
on a compound file cut short (`cfb.rs` lines 306, 330 and 346), and its
comment at `cfb.rs:114` says its loop over the DIFAT may not end.
