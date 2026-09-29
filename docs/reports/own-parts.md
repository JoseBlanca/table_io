# The report of the parts xlsx_rs reads before calamine

The work report of `docs/plans/own-parts.md`, carried out on the branch
`plan/own-parts` on 28 September 2026: a section for each work package,
with its deliverables, what its review found and what changed. The last
section is for whoever next revises a skill or writes a plan.

## State

Done on 29 September 2026. Work packages 1 and 2 were merged into
`main` on 29 September 2026 by the owner's order, at b6a2fbe; work
package 3, added the same day when the owner decided Open 3 of the spec,
is on the branch `plan/own-parts`, 15 commits after that; nothing is
pushed.

What a user of popnei_web gets from it, once it is released: the dates of
a workbook of the 1904 system saved by Excel 365, the owner's
`excel_1904.xlsx` among them, are right, `2024-05-13` where the release
before this plan would have given `2020-05-12`; a file damaged inside its
compressed bytes is refused instead of read with cells missing; and no
file found in four rounds of review, as small as 2.5 KB, ends the light
worker, the thread of popnei_web's tab that reads the user's files: each
is refused with a message. The largest memory a read was made to take
is 2.25 GB of the wasm's 4.29 GB. A table of 10,000 individuals is read
in about 1.2 times the time of the package before this plan, and the
package is 300,646 bytes gzipped, 15,776 more than before it.

What the owner decided on 29 September 2026: the merge of work packages
1 and 2; Open 3, one cell whose text could end the light worker, fixed
before the first release, which work package 3 did; and the first
release once the significant work of the plan is done. What is asked
now: the merge of work package 3, and the push and the release, which
cannot be undone once others have downloaded them.

## Work packages 1 and 2, built

- 1.1, 5797372: the zip opened before calamine, every part read to its
  end, and refused past 1,000,000,000 bytes unzipped,
  `MAX_UNZIPPED_BYTES`. 1.2, bb40625: the date system from the setting
  of the workbook that holds it, `workbookPr`, at the workbook's root; the owner's `excel_1904.xlsx` now
  gives `2024-05-13`. 2.1, 540af16: the settings parts, the table of
  texts and its `uniqueCount` bounded. 2.2, b1a0909: the merged ranges
  counted. 2.3, bd4af18: `tests/data/unique_count.xlsx`, 2,588 bytes, and
  the node tests; the package built from `main` trapped on it, this one
  refuses it.
- On 56a6903: `cargo test` 129 passed, 8 ignored; node 13 pass; `cargo
  tree -d` prints nothing.
- Measured under node 26.8.2 on the owner's Apple M5 Pro, best of 3,
  against the package built from `main` at f0e4307: `individuals_10000.xlsx`
  107 ms before, 155 to 166 ms after; a sheet of 100,000 rows × 20
  columns, 9.6 MB zipped, 1,367 to 1,399 ms before, 1,811 to 1,926 ms
  after, about 1.35 times. The `.wasm` from 284,870 to 298,447 bytes
  gzipped.
- The message of 5797372 says the sheet of its test has 4,276 compressed
  bytes; it has 4,249, and the zip crate refuses every one of their
  changed copies. The spec's 145 of 4,072 copies is from the spec
  reviewer's own file, of another size, and stands.

## The review of work packages 1 and 2, and the fix

Three reviewers read 56a6903 on 28 September 2026 and found that the
bounds could be passed: xlsx_rs read the parts as calamine reads them,
and every small difference between the two readings let a file past a
bound. Files of about 2.5 KB trapped the package under node, the
JavaScript runtime outside the browser, in three ways: a form feed, an
invisible character that calamine reads as a space and xlsx_rs did not,
before the name of a setting in the XML; a table of texts cut short
before its end; and a list of sheets that xlsx_rs read differently from
calamine, with a decoy sheet inside a defined name, the name Excel lets a
user give a range, or after a stray closing tag.

The fix, written into the spec first (1e44906) and built after: each
bound counts at least what calamine could hold, in every part calamine
could read for it, found by the end of the part's name, and to the end
of the part; the attributes are read with a copy of calamine's own reader
of them, under its MIT license; and the merged ranges are counted as the
bytes `mergeCell`, the name of a merged range in the XML, in every part,
so that xlsx_rs no longer looks for the sheet's part. The spec's review of that change found two more ways past
a bound before any code was written, and both are in the spec and the
code:

- calamine keeps a path for every sheet the workbook lists, and 2,000
  sheets naming one target of 500 KB, a zip of 8,586 bytes, took 1.0 GB:
  now refused, "the workbook lists too many sheets";
- calamine decodes a part in the encoding it declares, and a byte of an
  older encoding of Windows, `windows-1252`, can become 3 bytes of UTF-8,
  the encoding Excel writes: a part calamine holds whole in another
  encoding than UTF-8 is now refused.

It also led to three changes of what is refused, each of files no
program writes except the second: a `uniqueCount`, the number of texts a
table says it holds, larger than its texts is read, as calamine reads
it, and only one past 10,000,000 is refused; the parts of settings, the
workbook and the styles among them, are refused past 50,000,000 bytes
instead of 10,000,000, since a workbook that gathered tens of thousands
of styles can pass 10 MB; and a file whose `_rels/.rels`, the part of the
zip that says where the workbook is, names no workbook is refused by xlsx_rs rather than by calamine. The files
xlsx_rs now refuses that calamine would read are listed in the spec, at
the end of "What xlsx_rs reads before calamine".

The code of the fix, 4d59e4e..fd31a69, was reviewed in six categories,
spec, tests, errors, numbers, api and architecture. What was fixed after
it, 242fcf9..54d138c:

- the tests reviewer changed the code 56 times, and 14 of the changes
  that alter behaviour were not caught. Three would have given a wrong
  date unseen: the test of a form feed in `_rels/.rels` passed with the
  old reader of attributes, since its file was refused for another
  reason; and a workbook named in capitals, or with `&amp;` in its path,
  gave the dates of 1900. Each now has a test that fails under the
  change;
- doc comments that listed only some of the messages of an error, one
  that was false about 20 digits, and a message written in two places.

Not taken, each for a reason: the six bounds stay public constants, since
the doc comment of `read_first_sheet` links to them; the trap of one
cell of the sheet goes to the owner as Open 3 rather than into this plan,
since its fix refuses files of a size no one has measured yet.

## Measured on the branch

At 247df4d, on the owner's Mac, 28 September 2026:

- `cargo test` 184 passed, 14 ignored: 6 tests at the real values of the
  bounds, which take up to 70 s each in a build of the tests and pass in
  2.7 s together with `cargo test --release -p xlsx_rs --test parts --
  --ignored`; 7 that write the files of the tests; and the owner's
  `libreoffice.xlsx`, which does not exist. clippy, `cargo doc` and
  `cargo wasm-check` clean; node 13 pass.
- Under node 26.8.2, best of 3 rounds of 3 reads, the package of `main`
  at f0e4307 and this one read in turns, the machine busy with a load of
  13 to 17: `individuals_10000.xlsx` 133 ms and 159 ms, 1.20 times; a
  sheet of 100,000 rows × 20 columns, 9.6 MB zipped, 1,808 ms and
  2,226 ms, 1.23 times. The plan asked to be told if either doubled.
- Memory, measured by the architecture reviewer: no file it tried raised
  the largest memory of a read under node; a zip of 220,000 empty parts
  takes 175 MB natively, compiled for the Mac and not for the browser, against 166 MB for calamine alone and 261 MB before
  the fix, since xlsx_rs now lets go of its zip before calamine opens
  the file.
- The `.wasm` is 564,714 bytes, 300,145 gzipped with `gzip -9`, against
  284,870 gzipped on `main`.

## Work package 3, every part bounded

Added on 29 September 2026 when the owner decided Open 3: one cell of a
sheet could hold a text of up to 1,000,000,000 bytes, which calamine
builds at 2 to 6.6 times its size before xlsx_rs sees it, and a zip of
730,563 bytes ended the light worker so.

- Measured first, on the owner's Mac under node 26.8.2, 29 September
  2026: the largest sheet found in a zip of 20,000,000 bytes, popnei_web's
  limit, is about 248 MB, of blank cells each with a fill, compressed as
  Excel compresses; one cell of 300,000,000 bytes took the wasm to 1.71
  GB. So every part is now refused past 300,000,000 bytes,
  `MAX_PART_BYTES`, the table of texts among them, which had 400,000,000.
  Whether Excel writes a formatted range of a million rows cell by cell
  is not confirmed; such a sheet near 250 MB would be close to the bound.
- quick-xml decodes a part in any encoding a declaration names, even one
  in the middle of the part after a first declaration with none. So the
  bytes of every part are scanned for such a declaration: a part
  calamine holds whole, the workbook, its relationships, the styles, the
  table of texts, is refused if one is found, "a part of the file is not
  in UTF-8"; any other part is counted three times against the bound,
  since decoding makes at most 3 bytes of each byte, so that an image in
  SVG of an older program in `iso-8859-1` does not refuse its workbook.
- 68aa0e7 and 9a320db built it; the spec's review before the code found
  a declaration hidden behind a decoy attribute or a million bytes,
  which took the wasm to 4.17 GB, and the review of the code, in three
  categories, spec, tests and errors, found a sheet named as a table of
  texts that had the larger bound, a test that could not fail for its
  reason, a line no test guarded and a comment that said quick-xml takes
  a later declaration always. 0a15197..f35ebf3 fixed them.
- At f35ebf3: `cargo test` 202 passed, 19 ignored, the 11 at the real
  bounds passing in release in 3.1 s; clippy, `cargo doc`, `cargo
  wasm-check` clean; node 13 pass. Under node, the largest memory any
  file within the bounds took is 2.25 GB, a sheet and a table of texts
  of 300,000,000 bytes each made of runs of CDATA; no file of the four
  reviews trapped. The times, the package of `main` at b6a2fbe and this
  one read in turns with a load of 6: `individuals_10000.xlsx` 91 ms and
  94 ms, the sheet of 100,000 rows × 20 columns 1,249 ms and 1,303 ms.
  The `.wasm` is 565,045 bytes, 300,646 gzipped.

## At the end

`npm pack` in `js/xlsx_rs` built, tested and packed `xlsx_rs-0.1.0.tgz`,
314,949 bytes at f35ebf3, 8 files: `LICENSE`, `README.md`, `package.json`,
`THIRD_PARTY_LICENSES.md`, and in `wasm/` the `.wasm`, its JavaScript
and two files of declarations. `THIRD_PARTY_LICENSES.md` is new: the
notices of the 29 crates compiled into the `.wasm`, all permissive,
which their licenses ask to go with every copy. Unpacked and imported
under node 26.8.2, it read `written.xlsx` with its first cells
`"Individuo"`, `"Población"`, `"Altura"`, `"Fecha"`, `"Afectado"`,
`"ind1"`; `excel_1904.xlsx` as `"Fecha"`, `"Hora"`, `"2024-05-13"`,
`"14:30:00"`; `unique_count.xlsx` refused, "too many texts"; and
`encrypted.xlsx` as the refusal `encrypted`. The `.wasm` holds no path of
the owner's folders. The `.tgz` was not committed.

The package's comment on `readXlsx` now says that its error carries the
message of the zip crate, of calamine or of xlsx_rs; only the comment of
the declarations popnei_web reads changed.

## Risks seen and not acted on

For the issues of xlsx_rs, once its repository is on GitHub:

- The zip crate reserves room for the number of parts a zip says it has
  (`zip-8.6.0`, `read/zip_archive.rs`, lines 184 to 200), and a zip that
  says 8,000,000 could ask for about 2 GB in the wasm; suspected by the
  errors reviewer, no such file built. calamine opened such a file the
  same way before this plan.
- A styles part of 5 MB of one format and 250,000 styles using it is
  about 1.25 × 10^12 characters of calamine's reading, 20 minutes at a
  thousand million characters a second; computed, not measured. It is slow and not a trap.
- xlsx_rs's own reading of a table of texts holds its longest text, up
  to 400 MB, also for a table in a folder calamine never opens; the
  memory calamine takes for the table it reads is the same.

## For whoever next revises a skill or writes a plan

The owner can stop reading here.

- **A bound in front of another reader counts what that reader could
  hold; it does not copy its reading.** The first design followed
  calamine's rules for finding and reading each part, and three
  differences each let a file of 2.5 KB through. The fix that held
  counts a superset, by name suffix and by bytes, independent of which
  part calamine picks. The `writing-specs` skill could say so where it
  asks for claims about calamine, with this case.
- **Asking the spec reviewer to build a file that passes the bound
  worked.** Its first review found a trap of 1.0 GB from 8.6 KB, its
  second one of 300 MB through an encoding, both before any code; the
  reviewer of the code against the spec, asked the same, found the trap
  of one cell. The `spec-reviewer` and the `spec` category could ask it of
  every bound.
- **A test file that carries two hostile things cannot fail for the
  first.** The form feed of `_rels/.rels` was tested with a file that
  also held a huge `uniqueCount`, so it passed with the defect put back.
  The `coding` skill could ask for one hostile thing per file.
- **A mutation undone with `git checkout` took an uncommitted test with
  it**, which the subagent caught and wrote again. Mutations are safer
  with `git stash` or on a committed tree.
- **Costs, in the tokens the `Agent` tool reported:** the spec's change,
  first reader 72,000 and spec reviewer 140,000 over two rounds; the fix
  of nine findings, 375,000; the review of that code in six categories,
  771,000, the tests reviewer alone 199,000; the fixes of that review and
  the two last tasks, about 59,000 more. The review cost twice the code
  it reviewed, and found one defect that gives a wrong cell, three
  untested, and the trap of Open 3.
- **Work package 3, in the same tokens:** the measures before the spec,
  106,000; the spec's review, 145,000; the code and its six fixes,
  281,000; the review of the code in three categories, 352,000. The
  measures first, then a spec reviewer asked to build a file that passes
  the new bound, found the one trap that remained before any code.
- **This session ran from popnei_web**, whose subagents are its own; the
  subagents of xlsx_rs were run as `general-purpose` agents told to follow
  their definition in `.claude/agents/`. It worked, and a session of
  xlsx_rs started in its own repository would not need it.
