# The report of the reading of an xlsx

The work report of `docs/plans/read.md`, carried out on the branch
`plan/read` from 28 September 2026. It is written as the work goes, one
section for each work package, for the owner, who decides the merge and
the release; its last section is for whoever next revises a skill or
writes a plan.

## State

Under way. Work package 1 is being built.

## Work package 1, as it goes

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
