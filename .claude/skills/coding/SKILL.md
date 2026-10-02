---
name: coding
description: How code is written in table_io, in the Rust library crate, in the wasm-bindgen binding crate and in the package of js/table_io. Use it before writing or changing any code or test of table_io. It covers the order of the work, the integers, the numbers and the dates, errors without panics, the types and the names, what the architecture asks of the code, the binding crate and the package, the dependencies, the tests and the checks to run before the work is called done.
---

# Coding

Adapted on 28 September 2026 for xlsx_rs from the coding skill of
popnei, whose lints and rules it takes, with the rules of Rust of
pop_var_caller's `rust-feature-implementation` and `rust-code-review`
skills (`/Users/jose/devel/pop_var_caller/ai/skills/`) that fit a small
library, and revised on 2 October 2026 when xlsx_rs became table_io.
table_io has three layers, and `docs/architecture.md`, section 1,
describes them: the library crate `crates/table_io`, plain Rust with no
wasm-bindgen, where everything an import and an export do is, and which
Vavilov Explorer depends on; the binding crate `crates/table_io_js`,
which exports the functions of the package and does nothing else; and
the package `js/table_io`, which is what wasm-bindgen generates, with a
test under node. The goals are in `docs/objectives.md`, in order: the
table the user sees in their file, one reading in both applications, a
refusal the user can act on and never a panic, what is written reads
back, a small download, contracts that hold, and fast enough for a
desktop table. When two of them pull against each other the earlier one
wins.

Most of what follows is enforced by the lint table in `lints.toml`, beside
this file, which goes into the `Cargo.toml` of the workspace. A rule that a
lint enforces is given here with its reason and not repeated in detail.
The prose is for what no lint catches.

## Before the code

Code is written from a spec, `docs/specs/<module>.md`, and usually from a
task of an implementation plan. Read the part of the spec, the functions
of calamine or rust_xlsxwriter it names, in `~/.cargo/registry/src/` at
the version `Cargo.toml` pins, and the part of `docs/architecture.md` it
stands on. Where the spec moves a rule of popnei_web into table_io, read
the rule in popnei_web's `docs/specs/worker/individuals.md` too.

When the spec can be read in two ways that give different values, write
the file of the case, an xlsx with rust_xlsxwriter or a text file as its
bytes, and see what calamine or the library gives, before choosing. It
takes a minute, and the choice made without it is a guess that the tests
will not catch, because they were written from the same reading.

When the spec does not say what should happen in a case, the choice is
not made in silence. A choice that changes a value a user sees, a
refusal, the declarations of the package or the public Rust interface
goes to the owner as an open point of the spec. A smaller one is made
and written in the spec, in a commit of its own that comes before the
commit of the code and the tests, as the `writing-specs` skill says, and
it is named in the commit message of the code.

## The order of the work

1. The test first. It asserts the table, the cells or the refusal the
   spec gives, as literals, and it fails before the change. To make it
   compile, the new function gets a body that returns a wrong value, an
   empty table, and
   never `todo!()`, which the lints deny. Run the test and see it fail on
   the assertion. When a change cannot have such a test, a rename, a move,
   say so in the commit message.
2. The code, as small as the task asks for.
3. The checks at the end of this file, all of them, with their real
   output. A check that could not be run is reported as not run.
4. The commit, with the message the `writing` skill describes.

## Integers: no operator that can be silently wrong

In a release build `+`, `-`, `*` on integers wrap when they overflow, and
the program goes on with a wrong number; in a debug build the same line
panics, and `/` and `%` by zero panic in every build. `as` between integer
types truncates in both. The numbers table_io handles reach those limits:
a sheet has 1,048,576 rows and 16,384 columns, so the area of its
rectangle, rows times columns, does not fit in a `u32`, nor in a `usize`
under wasm, where it is 32 bits.

- The rows and the columns are `u32`, as calamine gives them; a row or a
  column as Excel numbers it, from 1, is the one calamine gives plus 1, a
  checked addition. An area, a count of cells, is a `u64`. A `usize` is for
  indexing memory and for nothing that has to be the same in wasm and
  natively.
- Integer arithmetic uses the methods that say what happens on overflow:
  `checked_add` and its family, with the `None` turned into an error, as
  the default; `saturating_` and `wrapping_` only when that is the meaning
  wanted. The lint `arithmetic_side_effects` is denied, so a plain
  operator on integers does not compile.
- Where a bound makes overflow impossible, the plain operator is allowed
  with the bound written down: `#[expect(clippy::arithmetic_side_effects,
  reason = "...")]` on the smallest item, with the bound and where it is
  established.
- To widen, `u64::from(x)`. To narrow, `u32::try_from(x)?`. No `as`
  between integer types. From a float to an integer, check for NaN and for
  the range first, because `f64::NAN as i64` is 0.

## Numbers and dates

- A number of a cell is given as calamine gives it, an `f64`, and is not
  rounded: `0.1 + 0.2` stays `0.30000000000000004`. A number that is not
  finite becomes the text JavaScript writes for it, as the spec says,
  since the cell that crosses to JavaScript holds a finite number only.
- A date is worked out from a whole number of milliseconds, as `read.md`
  gives it, "Each cell": the `f64` of calamine rounded once, checked for
  its range, turned into an `i64`, and split into days and milliseconds
  with the arithmetic of whole numbers. The parts of the day come from
  calamine and from no library of dates.
- Floats are compared with `total_cmp`, or as exact literals in a test
  when the value is one calamine gives without arithmetic of ours. The
  lint `float_cmp` is denied.

## Errors, and no panics

A panic in WebAssembly is a trap, which ends popnei_web's light worker,
the thread of its tab that reads the user's files, and the user's read
with it; in Vavilov Explorer, whose release build stops the program at a
panic, it ends the application and the user's unsaved work. So library
code does not panic: `unwrap`, `expect`, `panic!`, `todo!`,
`unimplemented!` and indexing with `[]` are denied outside the tests.
Slices are walked with iterators and `get`.

- An import ends in a table, a refusal or an error, an export in the
  bytes or a refusal, as `docs/architecture.md` says, section 7, and the
  specs give the types: `Refusal`, a value with the fields its words
  need, and `ImportError`, which holds a refusal or the message of a file
  that cannot be read. Each refusal of the spec is one case, named for
  what the user did, `OldExcel`, `Encrypted`, `RaggedRow`, and not for
  what the code checked.
- An error never passes silently. A file cut short or damaged is an error
  and not a table with fewer rows: an error of calamine while the cells
  are read ends the import, it does not end the sheet early.
- The errors of calamine, of the zip crate and of rust_xlsxwriter are
  turned into those of table_io in one place for each, one function of
  the library, which keeps their message as text. No type of theirs is
  in the public interface, so that the crate's users do not depend on
  their versions. A new kind of error of calamine that should be a
  refusal is a point for the spec.
- A `Result` is never dropped. `let _ =` on one needs a reason in a
  comment.
- The binding crate turns a refusal and a file that cannot be read into
  the fields of what it returns, the message as text, in one function,
  and adds nothing to the message. It throws a `JsError`, which
  JavaScript receives as an `Error`, only for a defect of the caller, an
  option that is not one of the package's, as `specs/package.md` says.

## Types, names and defaults

- A name says what the value is: `first_row`, `max_cells`, never `n`,
  `data`, `tmp`, `val`, `item`, `value`, `result`, nor an adjective
  alone, `current`, `last`, `next`, which needs its noun: `last_row`. A
  function is a verb, `import_table`, `cell_of_date`; a type is a noun
  of the domain, `Table`, `Refusal`. A `bool` reads as a question,
  `is_hidden`. A binding that holds a value of calamine before table_io
  has turned it into its own is named for calamine, `calamine_cell`, so
  that the layer shows at the use. The names of the things are those of
  the spec, a table, a column, a value, a sheet, a cell, the rectangle,
  a refusal, and one thing has one name in the Rust, the binding and the
  declarations, the last in camelCase.
- Where the spec gives a signature, a field or a code of refusal, that is
  it. When it looks wrong, that is a point for the owner and not a silent
  change: the declarations are the contract with popnei_web, and the
  public Rust interface the contract with Vavilov Explorer.
- Illegal states are not representable. No `bool` parameters, an enum
  with two named variants; two `bool` fields of which some combinations
  mean nothing are an enum of the combinations that do; a row and a
  column, or a number of rows and one of columns, that meet in one
  signature get newtypes when they could be swapped unseen. No value of a
  finite set passed as a string inside Rust; the codes of refusal are
  strings only in the struct the binding crate returns, made in one
  `match`.
- table_io has no default that changes what a user sees: the limits of
  bytes and of cells are the caller's, given with each import, and the
  choices of an export are the caller's too. A constant the code needs
  is a named `const` with a doc comment that says where its value comes
  from.
- A `match` on an enum of table_io or of calamine names every variant, so
  that a new one does not fall into a `_` arm. calamine's `DataRef` has
  variants its reader of an xlsx never gives, and they are named too, as
  the spec says.
- Private by default, `pub(crate)` between modules, `pub` for what an
  application calls, Vavilov Explorer or the binding crate, and for
  nothing else, since every `pub` item is part of the contract with
  Vavilov Explorer. Every `pub` item has a doc comment as the `writing`
  skill describes it, with `# Errors` when it returns a `Result`.
- No `unsafe`: the library crate has `#![forbid(unsafe_code)]`, and the
  binding crate has none of its own.
- A lint is silenced with `#[expect(lint, reason = "...")]` on the
  smallest item, never with a bare `#[allow]`.

## Rust that the compiler can check

Taken from pop_var_caller, where each rule was paid for:

- A struct literal names every field, never `..Default::default()`, so
  that a field added later stops the build at every place that has to
  think about it; a destructure in a trait impl names every field too,
  with no `..`.
- A slice of a known length is matched, `[first, second] => ...`, and not
  checked for its length and then indexed.
- `mut` lives only while a value is built: `let rectangle = { let mut
  ... ; ... };`.
- Borrow before cloning. A clone is questioned where it runs once for
  each cell of a sheet; one at the end of a read, or in a test, is left
  alone.
- Paths more than one module up are written from the crate,
  `crate::cell::Cell`, not `super::super::`. A module is one file,
  `src/date.rs`, until it has modules of its own. A file is renamed with
  `git mv`.
- A type or field's doc comment says first what the value is, its shape
  and its numbering, and then why only when that would surprise.
- A number of a cell goes through no function whose rounding depends on
  the platform, `ln`, `exp`, `powf` and their kin, which the operating
  system's library computes and which macOS and Linux were measured to
  round differently in pop_var_caller. table_io needs none: `round`, the
  four operations and the conversions to integers are exact everywhere.

## What the architecture asks of the code

- Never the whole sheet at once. The cells come one by one from
  calamine's `worksheet_cells_reader`, and the rectangle is checked
  against the limit as each arrives. `worksheet_range`, which builds the
  sheet before table_io can look at its size, is not called.
- Nothing is kept for a cell with no value; the cells of the rectangle
  are laid out once, at the end.
- One thread, no clock, no file system, no network: nothing of `std::fs`,
  `std::time`, `std::thread` or `std::env` in the library. It reads the
  bytes it is given.
- The step of the cells gives what the file holds, and the rules both
  applications share, the blank rows, the header, the missing values and
  the first column, are in the module `table` alone. The module `xlsx`,
  and the module `csv` where it decodes a text file and splits it into
  lines and cells, hold none of them: no value is made missing, no space
  is trimmed, no header is found. The one exception is a rule of the
  format, the spaces at the ends of a cell of a text file, which
  `specs/text-files.md` removes there. What a text means, missing or a
  number, is asked of the module `value`, and never worked out a second
  time elsewhere.
- Each format behind its feature: the modules `csv` and `xlsx` behind
  `csv` and `xlsx`, calamine, zip, quick-xml and rust_xlsxwriter used
  only behind `xlsx`, and the modules `table`, `value`, `types`,
  `export` and `export_cells` behind neither, so that a build with one
  feature compiles and refuses a file of the other
  (`docs/architecture.md`, section 5).
- The rules every export shares, the order of the cells and the
  refusals that do not depend on the format, are in the module
  `export_cells` alone, as those of the import are in `table`; the
  writer of a format refuses only what its format cannot hold, and the
  shape of the table is checked once, in `export_table`, before any
  writer.

## The binding crate and the package

The binding crate translates and holds no logic. If a function there has
an `if` about a cell, it is in the wrong crate: put it in the library,
where `cargo test` reaches it.

- Its structs are `#[wasm_bindgen(getter_with_clone)]` with each field
  `readonly` and named in camelCase with `js_name`, as the spec gives
  them.
  The crate has `crate-type = ["cdylib"]`, `test = false` and `doctest =
  false`: the functions wasm-bindgen generates are stubs that panic
  natively, and what it adds is tested under node.
- The package is what wasm-bindgen generates, with no TypeScript of its
  own. The declarations it generates, `wasm/table_io.d.ts`, are also
  kept in git as `js/table_io/test/table_io.d.ts`, the lines the spec
  gives, and `npm test` fails when the two differ. So a change of the
  contract cannot pass unseen: it is the spec first, then that file,
  then a new release, and popnei_web told what it changes.

## Dependencies

A dependency is pure Rust, builds for `wasm32-unknown-unknown`, and is
approved by the owner before it is added, with what it adds to the
package, raw and gzipped, measured. A version is pinned with `=`, since
a new version of calamine or rust_xlsxwriter can read or write a cell
differently; an upgrade is a
commit of its own, with every test and the size before and after, and the
spec corrected where the crate changed.

## Tests

- The values of a test are literals, from the spec. A test never computes
  its expected cells with the code under test, nor with a second copy of
  it. When the spec names a case and gives no value, the value is got by
  running the case, looked at against what Excel shows, and added to the
  spec with how it was got.
- The files of the cases are written in memory by the tests, an xlsx
  with rust_xlsxwriter's `Workbook::save_to_buffer` and a text file as
  its bytes, `b"id;h\r\nA;1,75"`, so that its encoding is the one the
  test says, each case a few lines that say what the file holds. The
  files the owner makes are read from `tests/data/`, and a test whose
  file is not there yet is marked `#[ignore = "waits for
  tests/data/<file>, made by the owner"]`.
- The cases of popnei_web's `docs/specs/worker/individuals.md` are tests
  here, with its literals where table_io keeps its rule, and with the new
  value, from the spec, where it changes it.
- Every field takes, in some test, a value that differs from the others:
  a rectangle that starts at row 1 and column 1 cannot tell the two
  apart, nor tell a row counted from 0 from one counted from 1.
- A test has to be able to fail. When in doubt, break the code on purpose
  and see the test fail.
- The malformed inputs are cases of their own: no bytes, a zip cut
  short, a compound file of the old Office, a text with a byte 0, a quote
  never closed, each asserting the refusal or
  the error it gives and not only that it fails.
- No file makes the library panic. One test takes a small xlsx written by
  rust_xlsxwriter and a small CSV, cuts each short at every length and
  changes each of its bytes in turn, and asserts, with
  `std::panic::catch_unwind`, that every copy gives a table, a refusal or
  an error. It needs no dependency, and it
  is how popnei found, by the same means, a damaged file its reader of VCF
  took for an empty one. A panic it finds inside calamine is a finding for
  the spec and an issue for calamine.
- Every kind of text the user types is in some test: accents, `Población`,
  a character outside the first plane of Unicode, an emoji, and a line
  break; and in a text file, each encoding the import reads and the
  export writes.
- An export is tested with the import: a table exported with each
  choice of the CSV and as an xlsx, and imported again, gives its names,
  values and types back, as `specs/export.md` lists.
- The name of a test says the behaviour and the outcome:
  `a_date_a_hundredth_of_a_millisecond_before_midnight_is_the_next_day`.

## Before the work is called done

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo wasm-check
cargo clippy -p table_io --no-default-features --features csv --all-targets -- -D warnings
cargo clippy -p table_io --no-default-features --features xlsx --all-targets -- -D warnings
! cargo tree -p table_io -e normal --no-default-features --features csv | grep -E 'calamine|zip|quick-xml'
cd js/table_io && npm run build && npm test
```

The seven cargo commands run for every change, and the last line from
the moment the package exists. The two runs of clippy with one feature
each check that a format left out takes nothing with it that the other
needs, and that no code is left that only the other format uses, which
a `cargo build` would pass with a warning; a run with the defaults sees
neither. The line of `cargo tree` fails when a build of `csv` alone
compiles a crate of the xlsx, a dependency that lost its `optional`,
which the clippy of `csv` alone passes; without `-e normal` it would
show the zip that rust_xlsxwriter, a dev-dependency, brings to the
tests. `cargo doc` fails on a broken
link or a malformed doc comment, which clippy does not read. `cargo
test` prints how many tests were ignored: the report says which files
they wait for. `cargo wasm-check`, an alias of `.cargo/config.toml`,
compiles both crates for `wasm32-unknown-unknown` with the warnings
denied, since nothing else builds them for wasm before the package is
built. A layer that does not exist yet is reported as not there, not as
passed.

When the change touches a dependency, the profile or the binding crate,
the size of `wasm/table_io_bg.wasm` and `wasm/table_io.js`, raw and with
`gzip -9 -c | wc -c`, goes into the commit message, before and after.

Report what each command printed when it failed and that it passed when it
passed.

## When this skill is wrong

No code existed when this was written, on 28 September 2026. The lint
table and the commands above are to be tried on the first work package.
The rules of the import, the export and the features were added on 2
October 2026, before their code, and are tried on the plan of table_io. A
lint that proves too noisy is changed in `lints.toml` with the count that
showed it, and a command that is not the right one is corrected here.
