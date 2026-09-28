---
name: coding
description: How code is written in xlsx_rs, in the Rust library crate, in the wasm-bindgen binding crate and in the package of js/xlsx_rs. Use it before writing or changing any code or test of xlsx_rs. It covers the order of the work, the integers and the dates, errors without panics, the types and the names, what the architecture asks of the code, the binding crate and the package, the dependencies, the tests and the checks to run before the work is called done.
---

# Coding

Adapted on 28 September 2026 from the coding skill of popnei, whose lints
and rules xlsx_rs takes. xlsx_rs has three layers, and
`docs/architecture.md`, section 1, describes them: the library crate
`crates/xlsx_rs`, plain Rust with no wasm-bindgen, where everything the
read does is; the binding crate `crates/xlsx_rs_js`, which exports one
function and does nothing else; and the package `js/xlsx_rs`, which is
what wasm-bindgen generates, with a test under node. The goals are in
`docs/objectives.md`, in order: the cells the user sees in Excel, a
refusal the user can act on and never a panic, a small download, and a
contract with popnei_web that holds. When two of them pull against each
other the earlier one wins.

Most of what follows is enforced by the lint table in `lints.toml`, beside
this file, which goes into the `Cargo.toml` of the workspace. A rule that a
lint enforces is given here with its reason and not repeated in detail.
The prose is for what no lint catches.

## Before the code

Code is written from a spec, `docs/specs/<module>.md`, and usually from a
task of an implementation plan. Read the part of the spec, the functions
of calamine it names, in `~/.cargo/registry/src/` at the version
`Cargo.toml` pins, and the part of `docs/architecture.md` it stands on.

When the spec can be read in two ways that give different cells, write
the file of the case with rust_xlsxwriter and see what calamine gives,
before choosing. It takes a minute, and the choice made without it is a
guess that the tests will not catch, because they were written from the
same reading.

When the spec does not say what should happen in a case, the choice is not
made in silence. A choice that changes a cell a user sees, a refusal or
the declarations of the package goes to the owner as an open point of the
spec. A smaller one is made and written in the spec, in a commit of its
own that comes before the commit of the code and the tests, as the
`writing-specs` skill says, and it is named in the commit message of the
code.

## The order of the work

1. The test first. It asserts the cells or the refusal the spec gives, as
   literals, and it fails before the change. To make it compile, the new
   function gets a body that returns a wrong value, an empty sheet, and
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
types truncates in both. The numbers xlsx_rs handles reach those limits:
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
with it. So library code does not panic: `unwrap`, `expect`, `panic!`,
`todo!`, `unimplemented!` and indexing with `[]` are denied outside the
tests. Slices are walked with iterators and `get`.

- A read ends in a sheet, a refusal or an error, as `docs/architecture.md`
  says, section 3, and the spec gives the types: `Refusal`, a value with
  the fields its words need, and `ReadError`, which holds a refusal or the
  message of a file calamine cannot read. Each refusal of the spec is one
  case, named for what the user did, `NotXlsx`, `Encrypted`, and not for
  what the code checked.
- An error never passes silently. A file cut short or damaged is an error
  and not a sheet with fewer cells: an error of calamine while the cells
  are read ends the read, it does not end the sheet early.
- calamine's errors are turned into those of xlsx_rs in one place, one
  function of the library, which keeps calamine's message as text. No type
  of calamine is in the public interface, so that the crate's users do not
  depend on its version. A new kind of error of calamine that should be a
  refusal is a point for the spec.
- A `Result` is never dropped. `let _ =` on one needs a reason in a
  comment.
- The binding crate turns a refusal into the fields of what it returns and
  the message into a `JsError`, which JavaScript receives as an `Error`,
  in one function. It adds nothing to the message.

## Types, names and defaults

- A name says what the value is: `first_row`, `max_cells`, never `n`,
  `data`, `tmp`, `val`. The names of the things are those of the spec, a
  sheet, a cell, the rectangle, a refusal, and one thing has one name in
  the Rust, the binding and the declarations, the last in camelCase.
- Where the spec gives a signature, a field or a code of refusal, that is
  it. When it looks wrong, that is a point for the owner and not a silent
  change: the declarations are the contract with popnei_web.
- No `bool` parameters, an enum with two named variants. No value of a
  finite set passed as a string inside Rust; the codes of refusal are
  strings only in the struct the binding crate returns, made in one
  `match`.
- xlsx_rs has no default that changes what a user sees: the limit of cells
  is the caller's, given with each read. A constant the code needs is a
  named `const` with a doc comment that says where its value comes from.
- A `match` on an enum of xlsx_rs or of calamine names every variant, so
  that a new one does not fall into a `_` arm. calamine's `DataRef` has
  variants its reader of an xlsx never gives, and they are named too, as
  the spec says.
- Private by default, `pub(crate)` between modules, `pub` for what the
  binding crate calls. Every `pub` item has a doc comment as the `writing`
  skill describes it, with `# Errors` when it returns a `Result`.
- No `unsafe`: the library crate has `#![forbid(unsafe_code)]`, and the
  binding crate has none of its own.
- A lint is silenced with `#[expect(lint, reason = "...")]` on the
  smallest item, never with a bare `#[allow]`.

## What the architecture asks of the code

- Never the whole sheet at once. The cells come one by one from
  calamine's `worksheet_cells_reader`, and the rectangle is checked
  against the limit as each arrives. `worksheet_range`, which builds the
  sheet before xlsx_rs can look at its size, is not called.
- Nothing is kept for a cell with no value; the cells of the rectangle
  are laid out once, at the end.
- One thread, no clock, no file system, no network: nothing of `std::fs`,
  `std::time`, `std::thread` or `std::env` in the library. It reads the
  bytes it is given.
- Nothing of popnei_web's rules: no value is made missing, no space is
  trimmed, no header is found. What the file holds is given.

## The binding crate and the package

The binding crate translates and holds no logic. If a function there has
an `if` about a cell, it is in the wrong crate: put it in the library,
where `cargo test` reaches it.

- Its struct is `#[wasm_bindgen(getter_with_clone)]` with each field
  `readonly` and named in camelCase with `js_name`, as the spec gives it.
  The crate has `crate-type = ["cdylib"]`, `test = false` and `doctest =
  false`: the functions wasm-bindgen generates are stubs that panic
  natively, and what it adds is tested under node.
- The package is what wasm-bindgen generates, with no TypeScript of its
  own. After a change of the binding crate, `npm run build` and compare
  `wasm/xlsx_rs.d.ts` with the declarations of the spec, line by line. A
  difference is a change of the contract: the spec first, then a new
  release, and popnei_web told what it changes.

## Dependencies

A dependency is pure Rust, builds for `wasm32-unknown-unknown`, and is
approved by the owner before it is added, with what it adds to the
package, raw and gzipped, measured. A version is pinned with `=`, since
a new version of calamine can give a cell differently; an upgrade is a
commit of its own, with every test and the size before and after, and the
spec corrected where calamine changed.

## Tests

- The values of a test are literals, from the spec. A test never computes
  its expected cells with the code under test, nor with a second copy of
  it. When the spec names a case and gives no value, the value is got by
  running the case, looked at against what Excel shows, and added to the
  spec with how it was got.
- The files of the cases are written in memory by the tests, with
  rust_xlsxwriter's `Workbook::save_to_buffer`, each case a few lines that
  say what the file holds. The files the owner makes are read from
  `tests/data/`, and a test whose file is not there yet is marked
  `#[ignore = "waits for tests/data/<file>, made by the owner"]`.
- Every field takes, in some test, a value that differs from the others:
  a rectangle that starts at row 1 and column 1 cannot tell the two
  apart, nor tell a row counted from 0 from one counted from 1.
- A test has to be able to fail. When in doubt, break the code on purpose
  and see the test fail.
- The malformed inputs are cases of their own: no bytes, a CSV, a zip cut
  short, a compound file of the old Office.
- The name of a test says the behaviour and the outcome:
  `a_date_a_hundredth_of_a_millisecond_before_midnight_is_the_next_day`.

## Before the work is called done

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo wasm-check
cd js/xlsx_rs && npm run build && npm test
```

The four cargo commands run for every change, and the last line from the
moment the package exists. `cargo test` prints how many tests were
ignored: the report says which files they wait for. `cargo wasm-check`,
an alias of `.cargo/config.toml`, compiles both crates for
`wasm32-unknown-unknown` with the warnings denied, since nothing else
builds them for wasm before the package is built. A layer that does not
exist yet is reported as not there, not as passed.

When the change touches a dependency, the profile or the binding crate,
the size of `wasm/xlsx_rs_bg.wasm` and `wasm/xlsx_rs.js`, raw and with
`gzip -9 -c | wc -c`, goes into the commit message, before and after.

Report what each command printed when it failed and that it passed when it
passed.

## When this skill is wrong

No code existed when this was written, on 28 September 2026. The lint
table and the commands above are to be tried on the first work package. A
lint that proves too noisy is changed in `lints.toml` with the count that
showed it, and a command that is not the right one is corrected here.
