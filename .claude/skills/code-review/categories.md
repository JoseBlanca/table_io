# The categories of a code review

A reviewer reads the section for its category and the parts of
`.claude/skills/coding/SKILL.md` it names. The rules are there with their
reasons and are not repeated here. What is here is what to look for and
how to look. Adapted on 28 September 2026 from popnei's, with rules of
pop_var_caller's `rust-code-review` checklists, reliability, naming,
refactor safety and errors, that fit a small library.

## spec

Does the code do what the spec says, and does calamine give what the
spec says it gives?

- Go through the part of the spec the change builds: what the user of
  popnei_web sees, each row of a table of cases, the refusals in their
  order, the open points and their "meanwhile". For each statement find
  the code that makes it true and the test that would fail if it stopped
  being true. A statement with no code, or with code and no test, is a
  finding.
- Run the cases. Write the file of a case with rust_xlsxwriter, in a crate
  of trial under `tmp/`, and see what the library gives; add two or three
  of your own the spec does not name: a sheet with one cell, a value in the
  last row or the last column of Excel, a merged range that runs outside
  the rectangle, a text of only spaces.
- Where a claim of the spec rests on calamine, open calamine's source at
  the pinned version, in `~/.cargo/registry/src/`, and check it.
- Look for what the code does that the spec does not say: a value made
  missing, a text trimmed, a cell skipped, an early return. Each is either
  a gap of the spec or a defect of the code, and the finding says which
  you think it is.
- When the code and the spec disagree and the code looks right, the
  finding is about the spec.

## tests

Can each test fail, and are the numbers the change claims true?

- For every test of the change, break the code it guards and run it: flip
  a comparison, drop the rounding of a date, return early, swap the row
  and the column. A test that still passes guards nothing, and that is a
  finding. Before reporting one, show that your change did alter the
  behaviour on some input, because a change that alters nothing proves
  nothing about the test. Report three numbers: the changes made, those
  no test caught, and those that changed no behaviour; only the second
  are findings.
- For every test that exists, name what in its file makes the failure it
  asserts reachable, and ask which wrong implementations also pass it. In
  pop_var_caller, ten of sixteen rounds of review had a blocking finding
  that was a test unable to fail, not wrong code.
- The ways a fixture hides a defect: a rectangle that starts at A1, so a
  row counted from 0 and one counted from 1 cannot be told apart; a date
  with no time; a sheet with no merged range; a limit no fixture reaches.
- The expected values are literals from the spec. A test that computes its
  expectation with the code under test, or with a second copy of it, is a
  finding.
- A test of one of the owner's files that is ignored says which file it
  waits for; one that runs asserts what the spec says the file shows.
- Every number that a doc comment, a test comment or the commit message
  gives about this change, a count of cells, a size in bytes, the row at
  which a limit is passed, is computed again, not read again. Report each
  as right or as wrong with the right value.
- Restore the code after each experiment, and check the restore by the
  content of `git diff`, not by its count of lines: in pop_var_caller two
  changes of a script of experiments reached a commit whose message quoted
  a suite that had passed on the clean tree.

## numbers

The sections "Integers" and "Numbers and dates" of the coding skill.

- Every `#[expect(clippy::arithmetic_side_effects)]` and every comment
  that gives a bound: is the bound true, and is it established where the
  reason says, for every caller?
- The area of a rectangle, a count of cells, in anything narrower than
  `u64`, and `usize` in anything that must be the same under wasm.
- `as` between integer types, and from float to integer without the check
  for NaN and range.
- The dates: the rounding to the millisecond done once, the split into
  days and milliseconds in whole numbers, a number below 0 or after 9999,
  the 1904 system, a duration past 24 hours and a negative one.
- Floats compared with `==`, or a number of a cell changed by arithmetic
  of ours when the spec gives it as calamine gives it.

## errors

The section "Errors, and no panics" of the coding skill.

- Every path to a panic outside the tests: `unwrap`, `expect`, `[]`,
  `panic!`, a division by zero, an `#[expect]` of one of those lints whose
  reason does not hold. A panic is a trap that ends popnei_web's light
  worker.
- For each way the input can be wrong, what does popnei_web receive? The
  refusal the spec gives, with its fields filled; or an `Error` with
  calamine's message; never a sheet with fewer cells.
- An error of calamine turned into a refusal or a sheet, an error dropped,
  a type of calamine in the public interface.
- A test of a malformed input that asserts only that it fails, and not
  which refusal or error it gives. The test that cuts short and changes
  each byte of a file: does it run, and over a file with every kind of
  cell?

## api

The section "Types, names and defaults" of the coding skill, and the doc
comments as `.claude/skills/writing/SKILL.md` asks for them.

- Read every new name as someone who has not seen the code: does it say
  what the value is? A generic noun alone, `data`, `value`, `result`, an
  adjective without its noun, `last`, a function that is not a verb, a
  `bool` that is not a question. Is the same thing called the same in the library, in
  the binding and in the declarations, and as the spec calls it?
- Signatures and fields against the spec, the declarations above all.
- `bool` parameters, two `bool` fields whose truth table has rows that
  mean nothing, two `u32` of different meaning that can be swapped in one
  call, strings for a finite set inside Rust, a `_` arm on an enum, a
  `pub` that could be `pub(crate)`.
- What the compiler would not flag when the code next changes: a struct
  literal with `..Default::default()`, a destructure with `..` in a trait
  impl, a slice checked for its length and then indexed.
- Doc comments: what the item is, in the words of the spec, the units
  and the numbering, from 0 or from 1, and `# Errors`.
- A function longer than a screen or with many branches, the same logic
  in two places, dead code, a `TODO` with no issue.

## architecture

The section "What the architecture asks of the code" of the coding skill,
against `docs/architecture.md`.

- The whole sheet built at once, `worksheet_range`, or cells kept past the
  limit before it is checked.
- Anything kept for a cell with no value, a copy of the cells where one
  would do.
- `std::fs`, `std::time`, `std::thread` or `std::env` in the library.
- A rule of popnei_web in xlsx_rs: a value made missing, a space trimmed.
- A new dependency, a feature turned on, a change of the profile: is it
  pure Rust, does it build for `wasm32-unknown-unknown`, what does it add
  to the package, and did the owner approve it?

## package

The section "The binding crate and the package" of the coding skill.

- Logic in the binding crate: an `if` about a cell, a value it works out.
- The struct: `getter_with_clone`, each field `readonly`, the names in
  camelCase as the spec gives them.
- Build the package and compare `wasm/xlsx_rs.d.ts` with the declarations
  of the spec, line by line. Any difference is a finding.
- The test under node: does it load the package as it is released, with
  the bytes of the `.wasm` given to `init`, read one of the owner's files
  and one refusal, and free what it gets?
- `package.json`: the name, `"type": "module"`, the `exports`, the `files`,
  the scripts of the build, against `docs/architecture.md`, section 5.
