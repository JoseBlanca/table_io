# The report of table_io

The work report of `docs/plans/table-io.md`, carried out on the branch
`plan/table-io` on 2 October 2026: what is done and what is asked of the
owner, then a section for each work package, with its deliverables,
what its review found and what changed, and the issues to open. The
last section is for whoever next revises a skill or writes a plan.

## State

The plan is carried out, on the branch `plan/table-io`, with one of
its deliverables not met: the package's `.wasm`, the compiled program
the browser loads, is 138 bytes larger than the plan allowed, for a
reason only the owner can settle, question 1 below. Nothing is merged
into `main`, nothing is pushed, and no release is made. Every check of
the coding skill, the list of commands the project runs before work is
called done, passes at the last commit of the branch: `cargo test
--workspace` 538 tests passed and 27 ignored, `npm test` in
`js/table_io` 46 passed and 1 skipped; the ignored and skipped tests
wait for files the owner makes.

Nothing a user of popnei_web sees has changed: popnei_web still installs
xlsx_rs's release `js-v0.1.0-dev.1` by its old address, which still
downloads the same file after the rename of the repository, its hash
equal to popnei_web's lockfile. table_io reaches popnei_web's users only
when popnei_web installs it, from its own session.

What exists now that did not:

- The library table_io, in `crates/table_io`, which imports a CSV, a TSV
  or an xlsx as a table of typed columns, integer, float, boolean or
  text, or refuses it with the reason and the place, by the rules of
  popnei_web's reader; and which exports such a table as a CSV, with any
  separator, decimal mark, encoding and text of a missing value, or as an
  xlsx of one sheet, every value reading back as itself or refused with
  its place. Vavilov Explorer can take it as a crate.
- The package `table_io` 0.2.0, in `js/table_io`, for popnei_web: the
  import, the conversion of a column and the rules of a value, without
  the export. `npm pack` gives `table_io-0.2.0.tgz`, 360,468 bytes, which
  under node reads `tests/data/written.csv` and `written.xlsx`. Its
  `.wasm` is 651,456 bytes, 330,388 with `gzip -9`, against 565,045 and
  300,646 for xlsx_rs's release.
- The speed, on the owner's Mac, an Apple M5 Pro, of a CSV of 100,000
  rows and 50 columns, 36,197,015 bytes in Windows-1252 as a Spanish
  Excel saves it: 0.505 s natively, the median of 7 runs. Under node,
  through the package, as popnei_web's worker would run it, the first
  import after the package is loaded takes 1.04 to 1.07 s and the later
  ones 0.519 s, since the engine first runs the program compiled quickly
  and then compiled well. So the goal of less than one second of
  `docs/objectives.md` is met natively and for every import but the
  first, which misses it by 0.04 to 0.07 s; a browser has not been
  timed. As the plan says, this is reported and not worked on.
  popnei_web refuses a file of more than 20,000,000 bytes, so it would
  not take this one.
- The memory: the import of a CSV of 20,000,000 bytes takes from 335.8
  MB of the program's memory, for rows of numbers, to 610.5 MB, the most
  found, for 2,500,000 short rows (work package 4). A program for the
  web can hold up to 4 GiB; whether a browser's worker gives 610 MB has
  not been tried. An export of an xlsx of 20,000 rows and 100 columns,
  a table of 61 MB, took 456 MB natively (work package 6).

What is asked of the owner. The merge does not wait on the questions:
each answer is a small change that can be made on `main` after it.

1. The 138 bytes, and the clock of rust_xlsxwriter, the crate that
   writes an xlsx. Linking it into the package makes the `.wasm` 651,456
   bytes where it was 651,318, 330,388 gzipped where it was 330,332, a
   second copy of one function of Rust's standard library that no
   setting of the build tried removes; the package holds none of the
   export's code and does nothing differently, so the bytes matter only
   as the plan's bound. Separately, rust_xlsxwriter reads the computer's
   clock to date each file it makes, and in a program for the web that
   read stops the program, so the export of an xlsx could not go into
   the package as it is. The options:
   - (a) Accept the 138 bytes, 0.02% of the `.wasm`, and settle the
     clock when the export enters the package, with rust_xlsxwriter's
     setting for the web.
   - (b) Leave the writer of an xlsx out of every build for the web: the
     `.wasm` goes back to 651,318 bytes, measured, and an export of an
     xlsx in such a build is refused as a format not built, a refusal
     the caller words for the user, where today it would stop the
     program. Vavilov Explorer is not built for the web and loses
     nothing; section 5 of `docs/architecture.md` gains the rule.
   - (c) A feature of its own for the export, which the package leaves
     off; a change of section 5 too, and of every build that exports.

   The recommendation is (b): it meets the plan's bound, and it turns a
   stop of the program into a refusal in the one kind of build where the
   writer cannot work.
2. An integer past 2^53 in an export of an xlsx. The owner decided that
   such an integer is refused, since a cell of Excel would hold it as the
   nearest float. But the import makes integers of a column of whole
   numbers of an xlsx up to 2^63, and a number such as 2^60, which a cell
   holds exactly, then cannot be saved again as an xlsx: the user opens
   a file, saves it, and is refused for a value that came from it. The
   options:
   - Keep the refusal of every integer past 2^53. The rule is one bound
     the user can be told, and a table read from an xlsx with such
     numbers is saved as a CSV or not at all.
   - Refuse only the integers a cell cannot hold exactly: 2^53 + 1 is
     refused and 2^60 written. Every table read from an xlsx can be
     saved as an xlsx; the rule the user is told is "a number too
     precise for Excel", and some large integers are refused while
     larger ones are written. A change of `specs/export.md` and of one
     function.

   The recommendation is the second.
3. A text that holds `_x` and four hexadecimal digits followed by a
   control character, such as a carriage return, is written by
   rust_xlsxwriter so that it reads back as another text: `_x0041` and
   a carriage return become `Ax000D_`, a wrong value or a wrong name of
   an individual with no warning. table_io refuses such a text, as a
   character the file cannot carry, which the spec says is the default
   until the owner decides. No way was found to write it so that it
   reads back; the only other option is to let it through, wrong. The
   recommendation is to keep the refusal, and to report the fault to
   rust_xlsxwriter's repository on GitHub, which is the owner's to do,
   since it publishes outside this project; what the issue says is under
   "The issues to open".
4. Excel's "CSV UTF-8" starts a file with three bytes, `EF BB BF`,
   that mark it as UTF-8; other programs write UTF-8 without them. A
   file of UTF-8 without those three bytes, cut short in the middle of an
   accented letter, as an interrupted copy leaves it, is not valid
   UTF-8, so it is read as Windows-1252, and every accented letter of
   the file is shown wrong: `España` becomes `EspaÃ±a`, with only the
   line "Read as Windows-1252" for the user to see why. popnei_web reads
   it the same way today, and `specs/text-files.md` says so. The
   options:
   - Keep it so. table_io and popnei_web read the file alike; the user
     who sees the wrong letters can set UTF-8 and gets every letter
     right but the last, cut, one.
   - Read a file whose only fault as UTF-8 is a character cut at its end
     as UTF-8, its last character not decoded and its line reported as
     the line not decoded. Every letter but the cut one is right with no
     action of the user, and table_io then differs from popnei_web
     there; a change of `specs/text-files.md` and a few lines of code.

   The recommendation is to keep it so: a file cut in the middle of a
   letter is also cut in the middle of a row, which the user sees as a
   row of the wrong length or its last value cut short, and the
   difference from popnei_web would be one more for its move to
   table_io.
5. The import looks at the first bytes of a file to know what it is,
   and the export of a CSV writes the first name of the header there.
   Five first names, written as they are, make the file read back as
   something else: in Windows-1252, a name that starts with `ÿþ` or
   `þÿ` makes the file read as UTF-16, an encoding of two bytes a
   character, so every pair of its letters becomes one character of
   another script, and the file is refused as empty; one that starts with `ï»¿` loses those three
   characters, read as the mark of UTF-8; in any encoding, `PK` and the
   control characters 3 and 4 make the file read as an xlsx, which then
   cannot be read, and the eight characters that start an old Excel
   file make it refused as one. The spec already refuses two cases of
   the kind, a header that starts as a VCF file of variants does,
   `#CHROM`, which the import refuses, and a first name
   that starts with the mark of UTF-8, and refuses the latter even when
   the name is in quotes, where it would read back as itself. The
   options:
   - Refuse the five, as a header that reads back as another format, a
     new refusal of `specs/export.md`. The user who names a column so is
     told to rename it.
   - Put the first name in quotes when it starts with any of these, so
     that the file starts with `"` and reads back as itself, and so too
     a first name that starts with the mark, which is then no longer
     refused. No refusal is added; the file holds one pair of quotes,
     which Excel does not show.

   The recommendation is the quotes: every such table is then exported
   and reads back, in table_io and in Excel, and the rule is one line of
   the writer and one of the spec. No file of the owner's has a name
   that starts so, so either choice changes nothing a user does today;
   until the owner decides, these names are written as they are.

The owner ordered the merge into `main` and the push on 2 October 2026,
with the five questions above still open. The owner's text files, the
look in Excel of the files of an export, and the three issues under "The
issues to open" are deferred by the owner; the tests of the files stay
ignored until then. When popnei_web and Vavilov Explorer move to
table_io is the owner's to decide; what each changes is in
`docs/architecture.md`, sections 8 and 9, and is done from their own
sessions.

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

## Work package 5: the export of a CSV

Built on 2 October 2026 by one subagent, both tasks: b815f5b (the spec:
a row, a column or a count of a refusal past 4,294,967,295 given as
4,294,967,295), 7721dca (task 5.1) and fa60d95 (task 5.2).

The deliverables, checked by the orchestrator at fa60d95 and again after
the review's fixes at 096f900:

1. `cargo test -p table_io --test export_csv`: 43 passed at fa60d95, 52
   at 096f900; every case of `specs/export.md`, "How it is verified",
   for a CSV, as its literal bytes or its literal refusal, each refusal
   with a column and a row that differ.
2. The round trip of "What reads back": 2,000 tables made with a fixed
   seed, each exported with the 36 combinations of separator, decimal
   mark, encoding and text of a missing value, 72,000 exports. 65,916
   read back as themselves; 35,584 of those were made again after a
   refusal, with a value made missing or a name changed, and each
   refusal is now checked to be due by its rule; 6,084 tables were
   refused as having no individual; 25,134 columns read back as the
   narrower type the spec allows, a text of whole numbers as integers.

`cargo test --workspace`: 508 passed, 26 ignored at 096f900. `npm test`:
46 passed, 1 skipped; the package does not change, the export not being
in it.

Changed in the plan: nothing. The export of an xlsx, asked of any build
before work package 6, is refused as a format not built.

What the owner should know:

- The floats: 306,312 floats, the smallest and the largest among them,
  and 100,006 integers were exported with each of the 12 combinations
  of separator, decimal mark and text of a missing value, and read back
  bit for bit, −0 as 0, as the spec says (numbers reviewer).
- A CSV exported with the decimal comma and read back without the mark
  set may lose its float columns: the import guesses the mark by
  counting the cells written with each, and two text columns of numbers
  with the point, `1.5`, outnumber one float column of `1,5`. The spec
  promises the round trip only when the import is given the mark the
  export used, so Vavilov Explorer sets the mark when it opens a CSV it
  wrote; the same holds of the encoding.
- Five first names make the file the export writes read back as
  something else, the question below.

The review, at fa60d95, by the categories spec, tests, numbers, errors,
api and architecture; the package was not touched:

- Fixed, the tests: the round trip made every refused table valid and
  exported it again, so a refusal given for no reason passed it; three
  such changes of the code, a subnormal float refused as not finite, the
  character U+FFFE refused in UTF-8, and every header that starts with
  `#` taken as a variants file, passed all 43 tests (tests). The order of
  "no individual" before "a variants file", and of a variants file
  before a mark at the start of the header, had no test (tests, spec).
  Eight literal tests added, each seen to fail under its change.
- Fixed, for work package 6: the writer of the cells could give a
  refusal and not the failure of rust_xlsxwriter, which the spec gives
  as an export that failed; the check of the shape of the table, a
  column of the wrong length, was inside the CSV's path, and an xlsx
  writer that missed it would have filled the short column with missing
  values (architecture, errors).
- Fixed, the interface: `Display` and `std::error::Error` for
  `ExportError`, as the import has them, so that Vavilov Explorer passes
  it on with `?` (api, errors, spec: three reviewers; a sentence added
  to the spec). The errors of `export_table` listed in order; how a
  column of a refusal is counted against the columns given (api).
- Fixed, the code: the characters removed at the ends of a cell and the
  quoting of a cell written once, shared with the import; the bytes of
  the file given back at their length, 16,911,682 bytes where the
  buffer had grown to 33,554,432 (architecture). The architecture and
  the coding skill name the modules of the export (architecture).
- Not taken: building the header line once (architecture); the check of
  a variants file must come before the refusals of the header's cells,
  which are found as the header is written, so the line is built twice
  by one function.

## Work package 6: the export of an xlsx

Built on 2 October 2026 by one subagent, both tasks: e214f5c (the spec:
what an xlsx writes for an empty first name, a float and the characters
of a text), 5654294 (task 6.1) and 44e39f8 (task 6.2).

The deliverables, checked by the orchestrator at 44e39f8 and again after
the review's fixes at abf9d61:

1. `cargo test -p table_io --test export_xlsx`: 26 passed at 44e39f8, 28
   at abf9d61, each file read back by `import_table` and, since the
   review, its cells by calamine too; rust_xlsxwriter 0.99.1 is a
   dependency of the feature `xlsx`, and `cargo tree` for `csv` alone
   names none of the crates of the xlsx, the coding skill's check now
   naming rust_xlsxwriter.
2. The round trip of an xlsx, over 2,000 tables made with a fixed seed:
   1,831 read back as themselves, 1,552 of them after a refusal, each
   refusal checked to be due; 169 had no individual; 794 columns read
   back as a narrower type. Each refusal only an xlsx gives is met: spaces
   at the ends 1,376 times, an integer past 2^53 1,207, U+FFFE or U+FFFF
   1,094, an error of Excel as a name 233 and as a value 115, a control
   character after `_x` and four hex digits 259.
3. Not met, and with the owner: the package's `.wasm` is 651,456 bytes,
   330,388 with `gzip -9`, against 651,318 and 330,332 at the end of work
   package 4, 138 bytes more; `wasm/table_io.js` is unchanged, 37,489 and
   6,758. The question is in "State".

`cargo test --workspace`: 538 passed, 26 ignored at abf9d61. `npm test`:
46 passed, 1 skipped.

Changed in the plan: nothing; deliverable 3 is ticked with its task, and
left open here until the owner answers.

What the owner should know:

- The memory of an export of an xlsx: 456 MB at its peak for a table of
  20,000 rows and 100 columns, half floats and half texts, which takes
  61 MB itself, natively, in a release build on the owner's Mac, an Apple
  M5 Pro; 0.73 s, a file of 9.2 MB. rust_xlsxwriter holds every cell and
  the whole XML of the sheet before it zips it, and its mode of low
  memory writes to temporary files, which the library does not use. Now
  in `specs/export.md`, "How it runs".
- rust_xlsxwriter reads the clock to date the file it makes, and natively
  writes the parts of the file on a second thread. The date makes two
  exports of one table differ in their bytes, in the properties of the
  file, as two files saved by Excel do. In a build for the web, reading
  the clock stops the program: an export of an xlsx in the package would
  end popnei_web's worker, which the architecture reviewer and the
  errors reviewer each saw under node. The package does not export
  today; the question is in "State".
- A file larger than 4 GiB inside the xlsx, which a table of 1,048,575
  rows and 100 columns of floats reaches after about 20 GB of memory,
  fails as an export that failed, with a message about an option of
  zip, and not as a refusal. The option would make every file need a
  reader of large zips, which rust_xlsxwriter's documentation says Excel
  is and other programs may not be; it is left off, and the spec says so.

The review, at 44e39f8, by all seven categories:

- Fixed, a wrong cell: rust_xlsxwriter writes a control character as
  `_x000D_` after it has escaped the text's own `_x` sequences, so a text
  holding `_x0041` followed by a carriage return read back as `Ax000D_`,
  a value or the name of an individual, with no refusal (spec). No way
  was found through rust_xlsxwriter to write it so that it reads back,
  so such a text is refused as a character the file cannot carry, the
  spec saying this is the default until the owner decides; the round
  trip now makes such texts. The fault is rust_xlsxwriter's, and goes to
  the issues.
- Fixed, the tests: eight literal tests read the file back by the import
  only, which gives the same integer for a text cell `001` and a number
  cell 1, so a writer that put numbers and booleans where the spec says
  texts passed them; they now check calamine's cells (tests). The round
  trip never met an error of Excel; it does now, and asserts that each
  refusal only an xlsx gives is met (tests). Two tests of a cell past the
  sheet checked only that the export failed (errors).
- Fixed, the documents: the order of two refusals of one cell (spec);
  the doc comments of the error of Excel as a name, which concerns the
  header alone, of a text read as missing and of a failed export (api);
  the memory and the option of large files (architecture, errors). The
  test of a space or a tab at the ends of a text is one function, used by
  the import and the writer (architecture).
- With the owner: the clock in a build for the web, with the 138 bytes;
  an integer past 2^53 that a cell holds exactly (numbers); whether Excel
  shows the smallest and the largest floats as written, added to the file
  the owner opens in Excel when it is made (numbers).

## Work package 7: the end

Built on 2 October 2026 by one subagent: 27950be (task 7.1), 4b45bf5
(the notice of dlmalloc), and after the review bf97208, d6efd34 and
1d39ed4.

The deliverables, checked by the orchestrator at 1d39ed4:

1. The sizes after `npm run build`: `wasm/table_io_bg.wasm` 651,456
   bytes, 330,388 with `gzip -9`; `wasm/table_io.js` 37,489 and 6,758.
   xlsx_rs's `.wasm` of `js-v0.1.0-dev.1` was 565,045 and 300,646, so
   table_io's is 86,411 bytes larger, 29,742 gzipped: the table step, the
   types, the reader of a text file and the new binding.
2. `cargo test -p table_io --release --test speed -- --ignored`, on a
   Mac17,9, Apple M5 Pro: a CSV written by the test, 100,000 rows of a
   name, 20 integers, 20 floats with the decimal comma and 9 texts such
   as `España7`, in Windows-1252 with `;`, 36,197,015 bytes, every
   option left to be found; 7 runs, median 0.505 s, from 0.492 to 0.529.
   Under node 26.8.2 through the package: the first import in a fresh
   process 1.038, 1.065 and 1.067 s in 3 processes; 7 later imports in
   one process, median 0.519 s, from 0.512 to 0.526. The test asserts
   every column's name, type and length and the first and last rows.
3. `THIRD_PARTY_LICENSES.md` made again: the 30 crates of `cargo tree -p
   table_io_js -e normal,no-proc-macro --target wasm32-unknown-unknown`,
   rust_xlsxwriter 0.99.1 now among them, and what Rust's standard
   library compiles into the `.wasm`, core, alloc and std, dlmalloc
   0.2.13, compiler_builtins 0.1.160 with libm 0.2.16, which the file
   had never listed. The README of the package says what it exports,
   each refusal with its fields; the crate's description says it imports
   and exports a table.
4. `npm pack` gives `table_io-0.2.0.tgz`, 360,468 bytes, of 8 files,
   `LICENSE`, `README.md`, `THIRD_PARTY_LICENSES.md`, `package.json` and
   in `wasm/` the `.wasm`, its JavaScript and two files of declarations;
   unpacked in a scratch folder and imported under node, it reads
   `written.csv` as text in Windows-1252 with `;`, 4 columns, and
   `written.xlsx`, 4 columns.

The review, at 4b45bf5, by the categories spec, tests and package, and
the README read by the first-reader as a session of popnei_web that has
never seen table_io:

- Fixed: the README sent its reader to the spec for the fields of each
  refusal, how a column is counted and what `convertColumn` accepts, and
  said that a text file cut short is unreadable, where it is `cutShort`
  (first-reader, spec); the reader, sent again, answered its five
  questions from the README alone. The speed test checked three columns
  of one row, and passed an import that cut every column to one value,
  in 0.413 s; it now checks every column, and its file is in
  Windows-1252, as its comment says a Spanish Excel writes it, which
  costs about 10% more than the ASCII file it was (tests). The time
  under node was the first import alone (tests). The notices of
  compiler_builtins and libm (package).
- Not taken: none.

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
- rust_xlsxwriter 0.99.1 escapes the `_x` sequences of a text before it
  writes its control characters as `_xHHHH_`, so `_x0041` followed by a
  carriage return is read back as `Ax000D_` by calamine, and by the rule
  of the format would be by Excel too, which was not tried; table_io
  refuses such a text meanwhile. For
  rust_xlsxwriter's repository (review of work package 6, spec).

## How the work went, for whoever next revises a skill or writes a plan

This section is not for the owner, who can stop here. The tokens below
are those the `Agent` tool reported for each subagent, a resumed one's
being its total.

- The review cost more than the building in work packages 4 to 6, and
  about as much in work package 7, and found what the building missed:

  | work package | building and fixes | the review | reviewers |
  |---|---|---|---|
  | 4, text files | 395,000 tokens, two subagents | 695,000 | 7 |
  | 5, export of a CSV | 263,000 | 417,000 | 6 |
  | 6, export of an xlsx | 303,000 | 513,000 | 7 |
  | 7, the end | 183,000 | 152,000 | 3 and the first-reader |

  The reviewer of tests found the most in each, by changing the code and
  seeing which test failed: 7 of 42 changes passed every test in work
  package 4, 6 of 30 in work package 5, and 3 of 38 in work package 6.
  A plan of this kind can count a review at about twice its building.
- A subagent that wrote a work package fixed its own review's findings,
  given each with its evidence and the orchestrator's verdict, in one
  message, and none had to be sent twice; the fixes took 60,000 to
  98,000 tokens in work packages 4 to 6, about a third of the building.
- The deliverable "no larger than" the size of the `.wasm` at an
  earlier commit, with no margin, could not be held for a reason that
  had nothing to do with what it guards: linking a crate changed how
  the linker kept one function of Rust's standard library, 138 bytes.
  What the deliverable meant to keep out, the code of the export, was
  checked otherwise, by the names of the functions in the `.wasm`. A
  bound of size in a plan names what it guards and checks that, with
  the size in bytes as a number to report.
- Three rules of the coding skill were missing and were added on 2
  October 2026, each with the review that showed it: an xlsx exported
  is read back by calamine as well as by the import; a property that
  repairs a refused table checks that each refusal is due; a test of
  speed checks its whole result.
- The reviewers of architecture and of errors found that a dependency,
  rust_xlsxwriter, reads the clock, by running a build for the web under
  node: the check `cargo wasm-check` compiles for the web and runs
  nothing.
  A plan that adds a dependency to a crate compiled for the web asks for
  one call of it run under node, not only compiled.
