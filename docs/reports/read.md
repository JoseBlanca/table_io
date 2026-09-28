# The report of the reading of an xlsx

The work report of `docs/plans/read.md`, carried out on the branch
`plan/read` from 28 September 2026. It is written as the work goes, one
section for each work package, for the owner, who decides the merge and
the release; its last section is for whoever next revises a skill or
writes a plan.

## State

Done on 28 September 2026, on the branch `plan/read`, 50 commits after
`main`; not merged, not pushed. Every task is ticked, every deliverable
checked, and the plan's final check passes.

What exists now: the library `crates/xlsx_rs`, which reads the first
visible sheet of an xlsx into its cells as the spec gives them; the
binding `crates/xlsx_rs_js`; and the package `js/xlsx_rs`, which `npm
pack` builds, tests and packs as `xlsx_rs-0.1.0.tgz`, 291,672 bytes, whose
`.wasm` is 517,781 bytes and 284,870 gzipped with `gzip -9`, against the
295,475 of the spec's trial. 102 tests pass under `cargo test`, and 14
are ignored: the 8 of the owner's files, which wait for them, and the 6
that write the committed test files, run by hand. Under node 9 tests
pass and 2 are skipped until `excel_en.xlsx` and `encrypted.xlsx` exist.
No browser ran the package in this plan beyond what one reviewer tried
in Chromium 153 and WebKit 26.6 (work package 1): popnei_web's tests do
that.

Asked of the owner:

1. The merge of `plan/read` into `main`.
2. Whether xlsx_rs may read three parts of the file itself, with the
   `zip` and `quick-xml` crates that calamine already brings, at the
   same versions, as direct dependencies: the checksums of the zip,
   which calamine does not check, so that a damaged file is refused and
   not read with cells missing (10 of 7,120 copies with one byte changed
   were, in work package 1); the merged ranges, and the table of texts,
   which calamine reads whole with no bound, so that a file written for
   it cannot trap popnei_web's light worker (a file of 6 KB did, in work
   package 4). Meanwhile the spec says what is not bounded.
3. The eight files of the spec's "Made by the owner". Their tests fail
   until the owner's cells are written into them, so that none can pass
   by being switched on without them.
4. The repository on GitHub, and the order for the first release,
   `js-v0.1.0-dev.1`, which popnei_web's work package 9 of stage 4 waits
   for.

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

## Work package 2: every cell but the dates, the merged ranges, the limit

- 2.1, e89d69e; 2.2, da04c6f; 2.3, 5046023. A whole number past 2^53,
  which calamine's reader of an xlsx never gives, is its digits as text.
  A number that is not finite could be tested through `read_first_sheet`
  after all: rust_xlsxwriter writes a formula saved with `NaN` or `inf`
  as a number, which calamine reads as one; the plan's "What could go
  wrong" did not apply.

Deliverables, checked on d7ea962, after the review's fixes:

1. `cargo test -p xlsx_rs --test cells`: 14 passed.
2. `--test merged`: 6 passed.
3. `--test refusals`: 18 passed; the XFD1 test fails if the limit is
   checked after the read, which the reviewer of the tests confirmed by
   moving the check.

The node test: 9 pass. The `.wasm`: 515,506 bytes raw, 283,532 gzipped.

The review, four reviewers over spec, tests, errors with architecture,
and numbers with api. What mattered, fixed in fbf00c9 and d7ea962 for the
spec and d166157 to e1183d1 for the code:

- Three files a program could be written to make, small in the zip and
  large in memory: one cell written 40,000,000 times, 1.29 GB from 6.5
  MB; one long text merged over 10,000 cells, 1.15 GB from 5.7 KB; and
  40,000,000 merged ranges, 650 MB from 5.1 MB. The first two are now
  refused. The third, and the workbook's table of texts, are read whole
  by calamine, and whether xlsx_rs reads them itself is asked of the
  owner with the checksums.
- A merged range written from its last cell was left unfilled, and two
  that overlap gave cells that depended on their order in the file; the
  first is now read as the same range and the second refused.
- calamine's arithmetic on rows and columns panics in a build of the
  tests and wraps in the package; calamine is now compiled without
  overflow checks in the tests too, so that the test of damaged files of
  work package 4 tests what ships.
- The refusals `cellError` and `sheetTooLarge` were not read in the
  package's test, and no test had a rectangle refused whose first row and
  column differ: both added.

## Work package 3: the dates

- 3.1, a896b39; 3.2, 674e93f. Every value of the spec's row of dates and
  the four of the 1904 system came out as the spec gives them.
- Decided with the work and written into the spec, b36599c and 426ec8c:
  the milliseconds of a time alone and of a duration are written when
  they are not 0, as for a date and a time; the hours of a duration have
  no leading 0; a number just below 0 that rounds to 0 milliseconds is
  the time `00:00:00`; calamine is never asked for a day past 2,958,466,
  since it keeps the year in 16 bits and gave 2024 for the day
  23,981,957; a time that rounds to a whole day is the date `1900-01-01`
  where Excel shows `00:00:00`, since calamine gives no format.

Deliverables, on 5a2e243:

1. `cargo test -p xlsx_rs --lib date -- --list`: 12 tests, which pass.
2. `cargo test -p xlsx_rs --test dates`: 17 passed.

The review, two reviewers over tests with spec, and numbers with errors
and api. No wrong date: the reviewer of the numbers compared every whole
day of both date systems, 1 to 2,958,465 and 1 to 2,957,003, with the
calendar of the chrono crate, and found 0 differences. What mattered,
fixed in 1d5d07f to 5a2e243: nothing tested that the reader uses the
workbook's date system, and with it forced to 1900 a Mac workbook of the
1904 system gave every date 4 years and a day early; now a workbook
written with `date1904="1"` is read through `read_first_sheet`. Three
rules and the cap on days had no test; they have.

## Work package 4: the checks for any cell

- 4.1, 9e9faff: no file panics. 4.2, f9ffa7a: `individuals_10000.xlsx`,
  932,051 bytes, a header and 10,000 rows of 20 columns, read in 0.86 s
  in the test build, with the same bytes from two runs of its writer.
  4.3, b8f3d4a: the tests of the owner's eight files.

Deliverables, on 55375a7:

1. `cargo test -p xlsx_rs --test no_panic`: 8 passed, 19,635 reads in
   2.5 to 3.7 s; none panicked. A `panic!` put in the reader for one
   length, and one put where calamine reports broken XML, each failed it.
2. `npm test`: every cell of `written.xlsx` with its type.
3. `individuals_10000.xlsx` committed and read by `data_files.rs`.
4. `cargo test -p xlsx_rs --test owner_files -- --list --ignored`: 8
   tests; 2 node tests skipped with the name of their file.

The review, one reviewer over tests and spec. What mattered, fixed in
e34776f for the spec and 098b593 and 55375a7 for the code:

- The test of damaged files reached little of calamine: a zip keeps its
  directory at its end, so every copy cut short stopped in the reader of
  the zip, and most flipped bytes at a checksum. It now also builds the
  file from its parts, uncompressed, and damages the XML of each part
  xlsx_rs reads.
- A file of 6 KB whose table of texts says it holds 400,000,000 texts
  trapped the package under node: calamine reserves room for them
  first. It is in the spec among what xlsx_rs does not bound, and in the
  owner's question 2.
- The tests of the owner's files looked for one right cell in a column
  and would have passed a height read as text beside it; they now check
  every cell of a column, and fail until the owner's cells are in them.

## At the end

`npm pack` in `js/xlsx_rs` built, tested and packed `xlsx_rs-0.1.0.tgz`,
291,672 bytes, 7 files: `LICENSE`, `README.md`, `package.json`, and in
`wasm/` the `.wasm`, its JavaScript and two files of declarations.
Unpacked into `tmp/` and imported by its path under node 26.8.2, it read
`written.xlsx` as `Individuos`, from row 1 and column 1, 5 × 5, its first
cells `"Individuo"`, `"Población"`, `"Altura"`, `"Fecha"`, `"Afectado"`,
`"ind1"`, `"Andalucía"`, `1.75`, `"2024-05-13"`, `true`. The `.wasm` holds
no path of the owner's folders. The `.tgz` was not committed.

Issues to open once the repository is on GitHub, all of calamine 0.36.1:
its reader of compound files panics on a file cut short (`cfb.rs`, lines
306, 330 and 346) and its loop over the DIFAT may not end (`cfb.rs:114`);
it reserves room for the texts by the count a file gives
(`xlsx/mod.rs:352`); it subtracts and adds row numbers with plain
operators (`xlsx/mod.rs:2792` and `:2853`), which panic in a debug build
and wrap in a release one, a row past 2^32 putting its value in another
cell.

## For whoever next revises a skill or writes a plan

The owner can stop here. This section is for the next session that
revises a skill of xlsx_rs or writes one of its plans.

- The reviews found what the tasks did not, every time: 4 reviews, 15
  reviewers, and each work package had at least one finding of a file
  that could end popnei_web's worker or of a test that could not fail.
  What made them find it was running the case: writing the damaged file,
  building the package and reading it under node, breaking a line. The
  subagents that wrote the code had tested what the spec's table listed
  and nothing beside it. A task prompt that asks, besides the spec's
  cases, for the files a program could write to break the reader would
  have found some of these before the review.
- `isolation: "worktree"` made the reviewers' trees in popnei_web, the
  repository the session started in, so two reviewers of work package 1
  found no xlsx_rs; the code-review skill now has the orchestrator make
  them (fe5592f).
- The cost, in the tokens of the subagents as the tool reported them:
  the tasks about 640,000, the reviews about 1,020,000, the fixes about
  470,000. The reviews cost more than the building, and found what the
  building missed; the review of work package 3, the smallest, cost
  123,000 and found the untested 1904 system.
- Spec changes came from every review, 11 commits of the spec in all, each
  before the code as the skill asks. Most were cases calamine handles in
  a way the spec had not said: the ends of a text, merged ranges written
  backwards, the year kept in 16 bits.
