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
