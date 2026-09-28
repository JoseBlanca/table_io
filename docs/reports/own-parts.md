# The report of the parts xlsx_rs reads before calamine

The work report of `docs/plans/own-parts.md`, carried out on the branch
`plan/own-parts` on 28 September 2026: a section for each work package,
with its deliverables, what its review found and what changed. The last
section is for whoever next revises a skill or writes a plan.

## State

Under way. Work package 1 is being built.

## Work packages 1 and 2, built

- 1.1, 5797372: the zip opened before calamine, every part read to its
  end, `MAX_UNZIPPED_BYTES`. 1.2, bb40625: the date system from the
  `workbookPr` of the workbook's root; the owner's `excel_1904.xlsx` now
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

## The review of work packages 1 and 2: to act on

Three reviewers on 56a6903, 28 September 2026: spec, tests, and errors
with numbers, api and architecture. Nothing below is fixed yet. The
first three findings are one defect seen three ways, and decide the fix:
xlsx_rs tried to read the parts as calamine reads them, and every small
difference between the two readers lets a file past the bounds.

1. **Attributes split differently.** calamine reads attributes with its
   own `RawAttrIter` (`calamine-0.36.1/src/attrs.rs:22-99`), which takes a
   form feed, `\x0C`, for a space and trims it from a key; xlsx_rs used
   quick-xml's `attributes()`, which keeps it in the key. Files of about
   2.5 KB trapped the package under node: `\x0CuniqueCount=` in the
   table of texts; `\x0CType=` in `_rels/.rels`, where xlsx_rs then finds
   no workbook and runs no check; and `\x0Cname=` on the sheet, which let
   36,000,000 merged ranges through, 583 MB natively.
2. **A table of texts cut short.** At the end of the part before
   `</sst>`, `parts.rs:477, 499, 507` return before comparing
   `uniqueCount` with the texts counted; calamine has already reserved
   room for `uniqueCount` texts when it met `<sst>`. A file of 2,506
   bytes with `uniqueCount="400000000"` and no `</sst>` trapped the
   package.
3. **The workbook listed by another rule.** calamine reads the content
   of a `definedName` as text and stops at the end tag named `workbook`;
   xlsx_rs lists a `sheet` inside a `definedName` and stops when its
   count of open elements reaches 0, which a stray end tag, `</x>`, or
   an element closed before the root reaches. Then `check_merged_ranges`
   counts a decoy part, or finds nothing and lets the file be
   (`parts.rs:97-124`, 570-610). 50 merged ranges with `max_cells` 10
   were read.

   The fix the orchestrator had chosen, to be written into the spec
   first: stop copying calamine's reading and bound by counting more
   than calamine could hold. Count every element of local name `si`,
   and read every `uniqueCount`, in every part whose name, matched as
   calamine matches it, ends in `sharedStrings.xml`, to the end of the
   part or to its first error, comparing then; count the `mergeCell`
   elements of every part of the zip, refusing if any part holds more
   than `max_cells`; read every attribute xlsx_rs reads with a port of
   calamine's `RawAttrIter` (calamine is MIT); and refuse a file when
   xlsx_rs cannot find a part calamine will read, the workbook from
   `_rels/.rels` above all, since then the two readers disagree.
4. **Tests missing** (the tests reviewer made 45 changes to the code, 19
   not caught, all 19 changing behaviour): the parts found through a
   workbook in another folder are tested only for the date system, not
   for the table of texts, the settings bound or the merged ranges; the
   real bounds `MAX_UNZIPPED_BYTES` and `MAX_TEXT_TABLE_BYTES` and the
   fields of `PART_BOUNDS` (lib.rs:201-204) have no test at their real
   values; `MAX_TEXTS` has no test of exactly 10,000,000 texts read; and
   the rules taken from calamine (the local name of `workbookPr`,
   `date1904="0"`, the last `workbookPr`, a nested `si`, `uniqueCount`
   with `+`, the last `officeDocument` relationship, the last relationship
   of an Id, the first sheet of a name, the first `mergeCells`) have
   none.
5. **API.** `read_first_sheet_within_unzipped_bytes` and
   `read_first_sheet_within_text_table_bytes` are public for the tests
   alone; move those tests into a `#[cfg(test)]` module of the crate,
   taking `tests/hand_written/` by `#[path]`, and keep the functions
   private. `raw_sheet_of` returns two `Vec<u8>` that can be swapped:
   name them.
6. **Accepted, to write in the spec:** the list of parts is held twice
   while calamine reads, 261 MB against 166 MB for a zip of 20 MB with
   220,000 empty parts; a part in a compression the zip crate does not
   read, which calamine never opens, now refuses the file; a `mergeCell`
   with no `ref` is counted, and a `uniqueCount` past 2^32 refused, both
   stricter than calamine in wasm.
7. **A number:** commit 5797372 says 4,276 compressed bytes of the
   sheet; the test's file has 4,249, every one of whose changed copies
   the zip crate refuses. The spec's 145 of 4,072 copies is the spec
   reviewer's own file, of another size, and stands.
