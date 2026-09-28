---
name: writing-plans
description: How an implementation plan is written in xlsx_rs. Use it when writing or revising a document under docs/plans/, which turns one or more settled specs into work packages with deliverables that can be checked, each made of tasks that a subagent can carry out. The following-plans skill is the one that executes it.
---

# Writing implementation plans

Adapted on 28 September 2026 from the writing-plans skill of popnei. A
plan turns a spec that is settled into the order of the work. It says
what is built first and what after, in which pieces, and how we will know
that each piece is done. It is carried out by the `following-plans`
skill: an orchestrator that sends each piece to a subagent, checks what
comes back, and goes on to the next one without stopping, so a plan has
to be something that can be run that way.

A plan decides nothing about the design. When writing it shows that the
spec left something open, the question goes back to the spec, as an open
point for the owner, and the plan waits or is written around it. When a
command that was run shows that a sentence of the spec is wrong, the
owner is told with the breakdown, and correcting the spec is the first
task.

The prose follows the `writing` skill. A plan lives in
`docs/plans/<name>.md`. The name says what it builds, in lower case with
hyphens, `read`, `write-report`, and it is also the name of the branch,
`plan/<name>`, and of the report, `docs/reports/<name>.md`.

## No filler

The number of lines is not the measure of a plan. Filler is. A plan has
as many lines as it has things the orchestrator and the subagents need,
and none besides.

The test for a sentence: would the orchestrator, or the subagent that
gets the task, do something different without it? What fails that test is
filler, and the usual kinds are these. What repeats the spec: the cells,
the cases and the signatures are there, the plan points at them, and a
copy is wrong as soon as the spec changes. What a task repeats from its
work package. A part that is there because the shape below has a place
for it: the parts are an order and not a form, a work package with
nothing known to go wrong has no such part, and a plan whose final check
is the sum of its work packages says so in a line.

## Before writing

- The specs the plan builds from exist, have been through their review
  and are approved by the owner. Everything a task builds has a spec
  behind it. When the spec is too thin to build from, the plan does not
  stand in for it, and the session that writes the plan does not write
  the missing part on the side: it tells the owner what is missing, with
  the breakdown.
- The open points of those specs are answered by the owner, or each has a
  "meanwhile" that the work can follow. For each one that is not answered
  the plan says which task the answer would change.
- Read the specs, `docs/architecture.md` and the code that exists. A plan
  written without looking at the code plans work that is already done, or
  builds on something that is not there.
- Run what the checks will run, where it can be run before the code: the
  toolchain and wasm-bindgen's command line, `which` and their versions; a
  build of calamine for `wasm32-unknown-unknown`; which of the owner's
  files are in `tests/data/`.
- Make the sketch the `writing` skill asks for. For a plan, its points
  are the breakdown: the work packages in order, each with what it gives,
  what it stands on and the number of tasks you expect, the reason for
  the order, and what is left out.
- Show the owner the breakdown before the plan is written, as a reply in
  chat. The question is whether the pieces are the right size and in the
  right order. The owner corrects ten lines faster than a finished plan.
  Write the plan when they have answered.

## Work packages

A plan is split into work packages, and a small plan can be one. A work
package is a part of the work that ends with something that exists and
can be checked: the workspace that builds the package and loads it under
node, the cells of every kind of the spec's table, the refusals. When it
is done the project is in a state that works, with every check of the
`coding` skill passing, whatever comes after.

The strongest check xlsx_rs has is the package as it is released, read
under node over a file the owner made, since that is what popnei_web
runs. So the first work package makes the whole path exist, from the
library crate to the package loaded under node, with the least in it,
and the later ones fill the library, each ending with the package built
and its test passing. A plan that writes the whole library first and
builds the package last finds a failure of the wasm build, of wasm-bindgen
or of the declarations at the end, when everything sits on it.

Each work package has:

- **What it gives**, in a sentence or two, in the words of popnei_web,
  the user of the package: what it can read, or what it is refused with,
  when the work package is done.
- **Its deliverables, each with the way to check it.** This is what makes
  it a work package. The check is something that can be run and that can
  fail: the tests of a part of the spec, named by that part, "every row
  of 'Each cell' has a cargo test with the spec's cell as a literal"; the
  package built and read under node; a build whose size is measured. "The
  dates work" is not a deliverable. The values stay in the spec. A value
  a check needs and the spec lacks is got by running the case, and
  putting it into the spec is a task, which the `spec` reviewer of the
  work package checks like the rest.
  A check fails on the commit the work starts from, and it fails because
  the thing is not there yet. `cargo test -p xlsx_rs dates` does not:
  when no test has `dates` in its name cargo runs 0 tests, prints `ok`
  and exits with 0. So a check made of cargo tests names the tests, or
  says how many have to run: `cargo test -p xlsx_rs -- --list` prints
  their names and their count. A check that is already true before the
  work checks nothing.
- **What it stands on**: the work packages before it, and what has to be
  in place outside the plan, one of the owner's files among it.
- **Its tasks.**
- **What could go wrong**, when something is known: the part of the spec
  that is thinnest, a claim about calamine no trial has seen, a file the
  owner has not made yet. The orchestrator reads this to know where to
  look.

Put first the work package that would change the plan if it failed.

## Tasks

A task is a natural part of its work package, one that a person would
also name as a unit: the types and the refusals of the first bytes, the
cell of a date and its tests, the binding crate and its struct. It is the
unit that is given to a subagent, which starts with nothing but the
skills, the spec and the plan: the orchestrator tells it which task is
its own, and it reads the whole work package of that task in the plan. So
a task is a few lines and repeats nothing of its work package. It:

- says what is built and where, the module and the files;
- names the part of the spec it is built from, by its heading;
- says which deliverables of the work package it serves, by their
  numbers;
- says what it needs from earlier tasks;
- is small enough to be done and reviewed in one go, and large enough to
  be worth a subagent of its own, which costs a prompt, a run of the
  checks and an entry in the report. Between an hour and a day of a
  person's work. A smaller one is part of its neighbour.

A task whose failure would be silent, a wrong cell and not a crash or a
failing test, is its own task with its own commit, and says which check
guards it.

Tasks that touch different files and do not need each other are marked as
able to run side by side. The rest run in order.

## The shape of the document

1. The opening: what the plan builds and from which specs, the date, and
   the state, draft, approved by the owner, under way or done.
2. In and out: what is built, and what a reader could expect and is not,
   with where it goes. The open points of the specs that are not answered
   go here, each with the task its answer would change, and so do the
   owner's files that are not made yet, each with the test that waits for
   it.
3. What has to be in place before the first task, in a form that can be
   checked, and what is not there yet, and so which of the checks of the
   `coding` skill run during this plan and which are reported as not
   there.
4. The work packages in order, each with the parts above. Work packages
   are numbered and tasks are numbered inside them, 2.3, with a box that
   the orchestrator ticks: `- [ ] 2.3 The cell of a date`.
5. How the whole plan is checked at the end, when that is more than the
   sum of its work packages: the package packed as a release would be,
   its size, and installed in popnei_web without being saved, when the
   plan changes what popnei_web reads.

A plan says what and in what order, not how. It does not hold code or
signatures, which are in the spec, and it does not repeat the spec's
reasons, which it points to.

## Before handing it over

- Every task names its spec part by its heading. A task with nothing of
  the spec behind it is a gap of the spec or work nobody asked for.
- Every deliverable has a check that can fail, and its values are in the
  spec. Every check was run on the commit the work starts from and failed
  there, or the plan says why it could not be run. A check made of cargo
  tests names them or counts them.
- Every work package ends with the package built and its test under node
  passing, or says why it stops before.
- Read it for filler, sentence by sentence, with the test above.
- The order of the plan is the breakdown the owner answered.
- Could a subagent that reads only one task, the spec and the skills do
  it? Read two tasks that way.
- Nothing in the plan decides what the spec left to the owner.
- The plan goes to the `first-reader` subagent, as any document, read as
  the orchestrator that will run it, with questions of this kind: what is
  the first thing to do, how do I know work package 2 is done, what do I
  do if the owner's file of a test is not there.
- The owner approves the plan before it is run.
