# The plan of the parts xlsx_rs reads before calamine

Written on 28 September 2026. It builds "What xlsx_rs reads before
calamine" of `docs/specs/read.md`: xlsx_rs opens the zip itself, checks
every part's checksum, takes the date system from the workbook, and
bounds what calamine would otherwise read whole, before it gives the
bytes to calamine. State: under way again, work package 3 added on 29 September 2026 by the owner's decision of Open 3; approved by the owner on 28 September 2026. Branch
`plan/own-parts`, report `docs/reports/own-parts.md`. It comes before the
first release, `js-v0.1.0-dev.1`, since without it every date of a
workbook of the 1904 system saved by Excel 365 is four years and a day
early.

## In and out

In: the four points of that section, their tests, and the measurements
of its "What it costs".

Out: the release, which is the owner's order; the owner's
`libreoffice.xlsx`, not made, whose test stays ignored. No issue is
opened in calamine's repository: the owner does not post in public
repositories that are not theirs, and calamine's defects stay recorded in
`docs/reports/read.md` and in the spec.

## What has to be in place

On `owner-files`, 5 commits after `main` at 84262f4, with the owner's
files and this spec: 108 tests pass, 8 are ignored, among them the
owner's 1904 file, which gives `2020-05-12` today; node gives 11 pass.
zip 8.6.0 and quick-xml 0.41.0 are in `Cargo.lock` through calamine and
in the cargo registry. The checks below fail on that commit: the 1904
test is ignored and gives the wrong date, and the named test files
below do not exist.

`cargo wasm-check`, the shortcut of `.cargo/config.toml` that compiles
both crates for the browser, runs in a worktree only once `main` has it
written as a string, as 405f9f6 of `owner-files`; until the branch is merged, the plan runs
the command it stands for by hand, `cargo check --workspace --all-targets
--target wasm32-unknown-unknown --config 'build.rustflags=["-D","warnings"]'`.

## 1. The zip and the date system

**What it gives:** popnei_web reads the dates of a Mac workbook of the
1904 system right, and a file damaged inside its compressed bytes is
refused instead of read with cells missing.

**Deliverables:**

1. `cargo test -p xlsx_rs --test parts -- --list` lists at least 5
   tests, which pass: a file whose sheet's compressed bytes are changed
   one at a time, each changed copy either refused or read with exactly
   the cells of the unchanged file; the bound of unzipped bytes, tested
   through a function that takes the bound as an argument, with a bound
   of a few KB; a table of texts named in
   capitals found; a Strict file of the 1904 system; the first 500 bytes
   of an xlsx refused with the zip crate's message.
2. `cargo test -p xlsx_rs --test owner_files`: the 1904 test runs,
   no longer ignored, and passes with `2024-05-13` and `14:30:00`.
3. The checks of the coding skill pass.

**Tasks:**

- [x] 1.1 The dependencies `zip` and `quick-xml` as the spec's "Its
  dependencies" gives them; a module of the library that opens the zip,
  finds each part as calamine finds it, and reads every part to its end,
  counting the bytes ("What xlsx_rs reads before calamine", the opening
  and point 1). Serves 1 and 3.
- [x] 1.2 The date system from the `workbookPr` that is a direct child of
  the workbook's root, used in the place of `has_1904_epoch` (point 2).
  A task of its own: a wrong date system is every date wrong and no
  failure. Serves 1, 2 and 3. Needs 1.1.

**What could go wrong:** calamine's rule for finding the parts,
`read_package_relationships` and `cached_zip_path`, has to be read and
followed exactly, or a file passes xlsx_rs's checks under another name.

## 2. The bounds

**What it gives:** no file of a few MB makes calamine hold GBs before
the first cell, which ends popnei_web's light worker.

**Deliverables:**

1. `cargo test -p xlsx_rs --test parts -- --list` lists at least 11
   tests, the 5 of work package 1 and one for each bound of points 3 and
   4: the settings parts past 10,000,000 bytes, the table of texts past
   400,000,000 bytes through a small bound and past 10,000,000 texts,
   `uniqueCount` larger than its texts, a missing `uniqueCount` let be,
   more merged ranges than `max_cells`. Changed with the spec after the
   review of the work package, 28 September 2026: the settings parts
   past 50,000,000 bytes, a `uniqueCount` past 10,000,000 refused and one
   larger than its texts read, and each bound tested at its real value
   (`docs/reports/own-parts.md`, "The review of work packages 1 and 2,
   and the fix").
2. `npm test`: the owner's 1904 file read with its dates, and
   `tests/data/unique_count.xlsx`, the file of 6 KB that trapped the
   package, refused and not a trap.
3. Measured and in the report, under node 26.8.2 on the owner's Mac,
   before and after: the read of `individuals_10000.xlsx` and of a sheet
   of 2,000,000 cells; the `.wasm` raw and gzipped.

**Stands on:** work package 1.

**Tasks:**

- [x] 2.1 The bounds of point 3: the settings parts, the table of texts
  and its count. Serves 1.
- [x] 2.2 The count of the merged ranges of point 4, after calamine has
  opened the file. Serves 1. Needs 2.1, since both change the same
  module.
- [x] 2.3 The node tests, `unique_count.xlsx` written by
  `crates/xlsx_rs/tests/hand_written/`, which builds an xlsx from the XML
  of its parts, and committed, and the measurements. Serves 2 and 3.

**What could go wrong:** the time. If the read of 2,000,000 cells more
than doubles, the owner is told before the plan ends, with where the
time goes.

## 3. Every part bounded

Added on 29 September 2026, when the owner decided Open 3 of the spec:
the trap of one cell fixed before the first release.

**What it gives:** no cell of a sheet, however large, ends popnei_web's
light worker; a file whose largest part holds more than 300,000,000
bytes, or is in an encoding other than UTF-8, is refused with a message.

**Deliverables:**

1. The tests of "What xlsx_rs reads before calamine", point 5, listed at
   the end of that section, pass; the review's file of one cell of
   999,000,000 bytes is refused, natively and under node.
2. Under node, a sheet of one cell of 300,000,000 bytes takes the wasm
   to no more than about 2 GB and does not trap.
3. The checks of the coding skill pass; the times of work package 2
   measured again.

**Stands on:** work packages 1 and 2.

**Tasks:**

- [ ] 3.1 `MAX_PART_BYTES` and the rule of UTF-8 by the bytes of every
  part, replacing the check of point 3 through quick-xml's decoder, and
  their tests. Serves 1 and 2.
- [ ] 3.2 The measures under node. Serves 2 and 3.

**What could go wrong:** the rule of UTF-8 refusing a file a user saves;
every file of `tests/data/` is read as before, or the task stops.

## At the end

The package packed as a release would be, and read under node from its
`.tgz`, as the plan of the reading did; the report says what the owner
is asked: the merge, and the order for the release.
