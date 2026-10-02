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

Work packages 1 to 4 are done, reviewed and written below; work
package 5, the export of a CSV, is next.

One question waits on the owner, and nothing of the plan rests on it.
A file of UTF-8 without the mark of its encoding, cut short in the
middle of an accented letter, as an interrupted copy leaves it, is not
valid UTF-8, so it is read as Windows-1252, and every accented letter of
the file is shown wrong: `España` becomes `EspaÃ±a`, with only the line
"Read as Windows-1252" for the user to see why. popnei_web reads it the
same way today, and `specs/text-files.md` says so. The options:

- Keep it so. table_io and popnei_web read the file alike; the user who
  sees the wrong letters can set UTF-8 and gets every letter right but
  the last, cut, one.
- Read a file whose only fault as UTF-8 is a character cut at its end
  as UTF-8, its last character not decoded and its line given as the
  line not decoded. Every letter but the cut one is right with no
  action of the user, and table_io then differs from popnei_web there,
  a change of `specs/text-files.md` and a few lines of `csv.rs`.

The recommendation is to keep it so: a file cut in the middle of a
letter is also cut in the middle of a row, which the user sees as a row
of the wrong length or a value missing at the end, and the difference
from popnei_web would be one more for its move to table_io.

## The rename of the repository on GitHub

The owner renamed the repository `github.com/JoseBlanca/table_io` on 2
October 2026. The address of `origin` in the main checkout was already
the new one, and `git ls-remote origin` answers from it, with `main` at
8bb8982 and the tag `js-v0.1.0-dev.1`. The old URL that popnei_web's
`package-lock.json` installs,
`https://github.com/JoseBlanca/xlsx_rs/releases/download/js-v0.1.0-dev.1/xlsx_rs-0.1.0.tgz`,
downloaded with `curl -L` the same day, gives 314,955 bytes whose
sha512 equals the lockfile's `integrity`, so popnei_web's `npm ci` is
not affected by the rename.

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

## Work package 4: text files

Built on 2 October 2026 by three subagents: tasks 4.1 and 4.2 by one, in
two sessions (746f18e, the spec: a line past 2^32 − 1 unreadable and the
header of a separator whose quote is never closed; e1fbf9d, left
unfinished when the first session paused; 5a17ce2), task 4.3 by another
(e544e33, b151bb9), and a spec correction of the orchestrator (970c47c).

The deliverables, checked by the orchestrator at b151bb9 and again after
the review's fixes at 87c8a86:

1. and 2. `cargo test -p table_io --test import_text`: 102 passed and 5
   ignored at b151bb9, 111 passed and 5 ignored at 87c8a86, the same with
   `csv` alone; every case of "How it is verified" of
   `specs/text-files.md`, the 256 bytes of Windows-1252 among them, and
   every row of the table of texts of `specs/import.md`, each a literal.
3. The round trip of a text file: 3,000 tables made with a fixed seed,
   each read back with its separator set, and 1,263 of them also with
   none set, those whose separator the search can find.
4. `cargo test -p table_io --test no_panic`: an xlsx of 5,649 bytes
   written by rust_xlsxwriter, 1,446,144 copies, and a CSV of 93 bytes in
   Windows-1252, 23,808 copies at b151bb9 and 121,600 at 87c8a86, when
   the same CSV in UTF-8 with its mark, in UTF-16 with its mark, and
   after `##` were added; each cut short at every length and each byte
   changed to the 255 other values. No copy panicked, in table_io or in
   calamine. It takes 52 s of the 62 s of `cargo test --workspace`.
5. The five tests of the owner's text files, `excel_es.csv`,
   `excel_es_utf8.csv`, `excel_mac.csv`, `excel_unicode.txt` and
   `libreoffice.csv`, are ignored, and with `--ignored` each fails on the
   missing file.
6. `npm test` in `js/table_io`: 38 of 38 at b151bb9; 46 passed and 1
   skipped at 87c8a86, the skipped one waiting for `excel_es.csv`.
   `written.csv` is read, and `raggedRow`, `duplicateIndividual` and
   `unclosedQuote` of a text file have their places.

`cargo test --workspace`: 452 passed, 26 ignored at 87c8a86.

The size of the package at the end of the work package, the bound of
work package 6, after `npm run build`: `wasm/table_io_bg.wasm` 651,318
bytes, 330,332 with `gzip -9`; `wasm/table_io.js` 37,489 bytes, 6,758
with `gzip -9`. At the end of work package 3 the `.wasm` was 323,876
bytes gzipped.

Changed in the plan: nothing. The orchestrator made the builds of the
tests compile the dependencies optimised, `[profile.dev.package."*"]
opt-level = 2`, which the architecture now says (section 11), since the
no-panic test took 108 s of the 126 s of the tests; it takes 52 s now.
The package is built with the release profile and does not change.

What the owner should know:

- table_io and popnei_web's own reader were run side by side under node
  over 280,000 random inputs by the implementer and 140,252 by the spec
  reviewer, the marks, UTF-16, Windows-1252, quotes, each line end and
  every option among them. They gave the same table or the same refusal
  in all of them but one kind: a file of UTF-16 that starts with three
  marks of its encoding, as one saved three times with a mark would.
  popnei_web names its first column with an invisible character before
  `id`, and table_io names it `id`, as the spec says; the spec's account
  of popnei_web was corrected (970c47c). Nothing was changed in either.
- The memory of a text file. popnei_web refuses a file of more than
  20,000,000 bytes. The import of such a file, measured as the wasm's
  memory under node 26.8.2 after `importTable`, at 87c8a86: 335.8 MB for
  a header of 100 names over rows of a name and 99 `0`; 548.0 MB for the
  same rows with their 99 cells empty; 610.5 MB, the most found, for
  `id,v` over 2,500,000 rows of `a,`, which is refused as a duplicate
  individual. Each cell is 16 bytes, and a file of only separators holds
  as many cells as bytes. Whether a browser's worker gives that much has
  not been tried; the time of a large file is measured in work package 7.

The review, at b151bb9, by all seven categories, spec, tests, numbers,
errors, api, architecture and package:

- Fixed, the time: a header of K empty names over R rows took time in
  proportion to K × R, 2.90 s for 40,000 by 40,000 and about 16,000 s
  estimated for a file of 20 MB, during which popnei_web's worker would
  not answer; the columns are now found in one pass, 0.01 s for the same
  file (numbers). popnei_web has the same loop.
- Fixed, the memory: a blank row kept a cell for each column, so a file
  of 20 MB of line breaks took 501.4 MB and now takes 21.4 MB, and a
  header of 20 MB of commas 581.4 MB, now 361.4 MB (architecture,
  numbers). The import spec's bound of the cells of a text file by its
  bytes, half of them and one, was false, an empty cell taking no byte;
  it is now their number and one (numbers).
- Fixed, the tests: fourteen rules that had code and no test that failed
  when the code was broken, among them the header of a quote never
  closed, the line of a quote opened in the second cell of a row, a lone
  carriage return as the line end of Excel for Mac, `""y` as a cell, the
  ragged row before an empty or duplicate individual, and the separator
  chosen by the header counted without its empty end; each new test was
  seen to fail under the reviewer's change of the code (spec, tests,
  errors, three reviewers on the same points). The no-panic test reached
  none of the decoding of UTF-16 or of a UTF-8 mark (errors, tests). The
  package's tests set no option of a text file, so a separator `;` read
  as `,` would have passed all 38 (package, tests, errors).
- Fixed, the code against the spec: a row of more than 4,294,967,295
  cells is unreadable, as the spec says, and not only a ragged one
  (spec). The decimal mark is counted by the table step, which the
  spec and the architecture now say (architecture). Doc comments of the
  ragged row, of the line not decoded, of the errors and of the
  borrowing of the text, and two tuples of positions made structs
  (api). `written.csv` said to be as a Spanish Excel writes it, with its
  booleans in English, which a Spanish Excel writes as `VERDADERO`; now
  said as it is (package). `.gitattributes` keeps the line ends of the
  text files of `tests/data` (package).
- Not taken, as issues: the two values that tell the table step what it
  reads, where it came from and how far it is read, can be given in
  pairs that mean nothing, which no caller builds today (api); the
  header counted for the separator and by the table step by two pieces
  of code that agree over 1,000,000 random texts and could drift
  (architecture).
- Not taken: that the decimal mark counts the cells of the columns that
  are dropped (tests); such a column holds only missing values, which
  are not counted.
- For the owner: a UTF-8 file cut short in the middle of an accented
  letter, below.

## The issues to open

When the repository's issues are used for table_io, these are opened:

- The table step is given where a table came from and how far it is
  read as two values whose pairs no type forbids: an xlsx with a decimal
  mark still to find, or a text file read as an xlsx. Each is built in
  one place today. One value that holds both would let the compiler
  refuse a wrong pair (review of work package 4, api).
- The header without its empty end is counted twice, once to find the
  separator in `csv.rs` and once in the table step; the two agree over
  1,000,000 random texts, and a change of "The header" of
  `specs/import.md` has to be made in both (review of work package 4,
  architecture).
