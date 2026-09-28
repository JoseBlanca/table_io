---
name: writing-specs
description: How a spec is written in xlsx_rs. Use it when writing or revising a document under docs/specs/, which says what one module of xlsx_rs has to do, what calamine gives it and what the files of Excel, LibreOffice and Google Sheets hold, its Rust interface and the declarations of its package, how the result is verified and what is left for the owner to decide. The prose follows the writing skill, which is read first.
---

# Writing specs

Adapted on 28 September 2026 from the writing-specs skill of popnei. A
spec says what one module of xlsx_rs has to do and how we will know that
it does it. It is written before the code of the module and it is what the
implementation plan and the tests are made from. It does not give the
order of the work, which is the plan's, and it does not repeat
`docs/architecture.md`, which it points to.

The prose follows the `writing` skill. Read that first. This skill says
what goes into a spec, in which order, and what stays out.

## The example

`docs/specs/read.md`, the reading of an xlsx into cells, written in
popnei_web on 27 and 28 September 2026, reviewed twice against the source
of calamine 0.36.1 and two crates of trial, approved by the owner on 28
September 2026, and moved here whole. Read it before writing a spec, for
its order, for how it says what the user of popnei_web sees before the
rule, for how it gives each claim about calamine with the function it was
read in or the trial that saw it, for how it marks what the owner decided
with the date and the option not taken, and for how its verification
gives each case as a file and the literal cells it gives.

## The two readers

The owner reads a spec to check that the right thing is going to be built
and to decide the points that are theirs. The implementer, a person or a
session of the assistant, reads it to build the module without having to
work out again what calamine gives or what Excel writes. Every paragraph
is for one of the two. A paragraph that is for neither, that defends a
choice nobody questioned or shows how carefully the writer worked, goes
out.

## No filler

The number of lines is not the measure of a spec. Filler is. A spec has
as many lines as it has things its two readers need, and none besides. A
spec that the owner does not read to the end has failed, however correct
it is, and what stops them is not its size but thoroughness for its own
sake: every edge named, every part filled.

The test for a sentence is this: would an implementer who has read
calamine's function get this wrong without it? Would the owner decide
differently without it? When both answers are no, it goes out. That
calamine's reader of an xlsx never gives a whole number, `Int`, is in a
spec, because it tells the implementer which arm of the `match` no file
can reach. What a `u32` does at its maximum is not.

The parts below are the order of a spec and not a form to fill. A part
with nothing to say is a line or is left out.

A spec that describes more than one person can build in one go is split
into two specs along a line the code will also have.

## Before writing

1. Read the source of calamine, or of the other crate the module stands
   on, at the version `Cargo.toml` pins, in `~/.cargo/registry/src/`: the
   functions the module will call and the ones they call. What calamine
   gives is the specification of what xlsx_rs receives, and what it gives
   for a case is known from its code, or from running it, not from its
   documentation.
2. Try it: a crate of trial under `tmp/`, which git ignores, that calls
   calamine on the files of the cases, written with rust_xlsxwriter or
   made by the owner, and prints what comes back. It costs minutes, and it
   is what found, for `read.md`, that calamine's parts of a number a
   hundredth of a millisecond before midnight are 13 May 2024 at hour 24.
3. Write down what you find that a reader of the signature would not
   expect: a value calamine gives for two things, an error that refuses the
   whole sheet, a format taken for a date. These are what the spec is most
   needed for.
4. Make the sketch the writing skill asks for. The names of the things are
   those of `read.md`: a sheet, a cell, the rectangle of the values, a
   refusal, and Excel's own word where the user sees one.
5. Ask the owner, in a reply in chat, what would rewrite the spec if it
   were answered after the writing. Two things. The first is what the
   module adds that others will call: the Rust functions and types, the
   exported function and what its package declares, each in a line, with
   the one at which each check of "How it is verified" will be made. The
   second is any open point whose other answer would change the parts
   around it and not one sentence. Each is asked as the writing skill asks
   for a decision. An open point that moves one value or one paragraph is
   not asked now: it stays in the spec with its "meanwhile". What the
   owner answered is in the spec as decided, with the date and the option
   not taken. When there is nothing of either kind to ask, say so in a line
   and go on.

A claim about what calamine does comes from code that was read, and names
the function, or from a trial that was run, and says so. A claim about
what Excel, LibreOffice or Google Sheets write comes from a file one of
them wrote, which only the owner can make here: until it exists, the
claim is marked as unconfirmed and says which file will settle it. A
number the writer worked out, the row at which a rectangle passes the
limit, comes with how it was obtained, so that the next person can get it
again.

## The parts of a spec

One spec for each module, in `docs/specs/<module>.md`: `read.md` for the
reading, and `write.md` for the writing when it comes.

**The opening.** What the module gives to its user, popnei_web today, in a
paragraph. The date, that there is no code yet or what there is, and the
specs and documents it depends on.

**What it does**, first as what a user of popnei_web sees, the table of
their first sheet as in Excel, and what goes wrong when the module is
wrong; then the rules, each under a `###` heading that names its subject,
with a table where the cases are parallel, as "Each cell" of `read.md`
has one row for each value of calamine.

**The refusals**, when the module can refuse: each, in the order the
code meets them, with what it carries for its words and what the user is
most likely to have done.

**The Rust interface.** The types and the signatures of the library crate
that the binding crate calls, in a code block, with a sentence before each
one. Nothing private.

**The exported function and the declarations**, when the module adds to
what the package exports: the Rust of the binding crate, and what
wasm-bindgen declares of it, taken from a build and not written by hand.
These declarations are the contract with popnei_web
(`docs/architecture.md`, section 4), and a spec that changes them says
what popnei_web changes with them.

**Its size**, when the module adds to the package: what it adds, raw and
gzipped, measured in a crate of trial or in the build.

**The cases** that a reader would not guess, each with what comes out.

**How it runs**, when there is something to say: the memory at its
largest, what is read once and what for each cell.

**How it is verified.** The tests at the library's public function, each a
file and the literal cells or refusal it gives, and whether the file is
written by the test or made by the owner, with what it must hold; the test
of the built package under node; and what popnei_web checks on its side.
Each check names the function it is made at, the highest at which the
value can be seen: a check at a private helper keeps that helper from
changing. A helper is checked alone only for what no file can hold, as
`read.md` checks the dates of the 1904 system, which rust_xlsxwriter does
not write.

**Open points.** What the owner has to decide, as described below.

**Not in this spec.** What a reader could expect to find here and is
somewhere else or will not be built, with where it goes. A few lines.

## Decided, inherited and open

Every statement in a spec is one of three kinds, and the reader has to be
able to tell which.

Decided: the spec states it, with the reason when there was a real choice.
When there was no choice there is no reason to give, and one is not made
up.

Inherited: calamine gives it this way and xlsx_rs passes it on. "calamine
0.36.1 knows seven errors and refuses the sheet at any other" is a
complete entry, because it tells the reader what may move with the next
version of calamine.

Open: the owner decides. The writer does not decide it for them and does
not leave it as a sentence that can be read both ways. Where it comes up,
the text gives the fact and marks it, "(**Open 2**, below)", and says no
more. The list at the end has each point once, as a request for a decision
in the form the writing skill gives: the options, what each gives and
takes, the recommendation, and what the implementer does meanwhile. The
points are numbered through the whole spec, and the list opens with a line
that says the owner decides them and the implementer follows the
"meanwhile" of each until they do.

What the writer can decide alone, a name, the layout of a private struct,
is decided and not listed. Few open points, each one worth the owner's
time: what changes a cell a user sees, a refusal, or the declarations of
the package.

While nothing has been built from a spec, a changed decision is changed in
the text and git keeps the history. Once code has been built from it the
spec still changes first: a changed decision, an open point the owner
answered, a case the code or one of the owner's files found that the spec
did not have, goes into the spec in a commit of its own, and the commit
that changes the code and the tests comes after it. The spec is where the
origin of every literal is written down, so it is never behind the code.

## Before handing it over

The spec, or the part of it that was written, goes to the `first-reader`
subagent as the writing skill says, once as the owner and once as the
implementer, with questions for each. For the implementer: what comes out
for a date of the 1904 system, which function of calamine says which
sheets are hidden, what the first test asserts. For the owner: what is
asked of me, and what does each answer lead to.

Then read the list of open points against the text. Every **Open** in the
text is in the list, and nothing in the list is already decided somewhere
in the text.

Read it for filler as the owner would, who has a day of other work. For
each part, and for each sentence that looks like diligence, ask what
would be lost without it.

Last, the review. The code is made from the spec, so a wrong sentence in
it becomes wrong code, and the writer who misread calamine will not find
the misreading by reading the spec again. The spec goes to the
`spec-reviewer` subagent, with the path of the spec and the functions of
calamine it is about. It checks the claims against calamine's source and
by running them, recomputes the numbers, looks for what is missing, for
points that should be open or should not be, for conflicts with the
architecture and with what popnei_web reads, and for what could go.

A change made to a spec after its review is checked against every other
place that speaks of the same thing: search the spec for the name and for
the number. A sentence added to one part can contradict a worked figure in
another, and a spec that gives two answers is read as the one the
implementer finds first.

Evaluate each finding before acting on it. The reviewer can be wrong, and
it has not read everything the writer read. Check its evidence, take what
makes the spec more right or easier to build from, and leave the rest.
When the spec is handed to the owner, the message says which findings were
not taken and why, a line for each.

## When a spec is sent back

As in the writing skill: the spec is corrected, the principle here that
allowed the failure is revised, and the case is saved under `cases/`
beside this file.
