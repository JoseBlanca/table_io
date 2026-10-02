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
