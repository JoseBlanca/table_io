# table_io: how the assistant works here

table_io is a Rust library that imports a table from a CSV, a TSV or the
first visible sheet of an xlsx file as typed columns, and exports typed
columns to a CSV or an xlsx. It was xlsx_rs, the reader of the cells of
an xlsx for popnei_web, until the owner widened it on 2 October 2026. It
has two applications: popnei_web, the web applications of popnei, which
takes it compiled to WebAssembly and released as a wasm package, and
Vavilov Explorer, a desktop application of the owner, whose backend
depends on the Rust library by git. What it is for and what it is not is
in `docs/objectives.md`, its parts, its two contracts and how it is
released in `docs/architecture.md`, and what each module does in its
spec under `docs/specs/`. popnei_web is at `/Users/jose/devel/popnei_web`,
Vavilov Explorer at `/Users/jose/devel/vavilov-explorer`, whose
`docs/table_io-needs.md` says what it needs of table_io, and popnei,
whose conventions this project follows, at `/Users/jose/devel/popnei`.

The owner is a population geneticist who programs in Python and Rust. They
use Excel as its users do, and have not had to know what is inside an
xlsx, a zip of XML files, nor the encodings a CSV can be saved in, nor
how a wasm package is loaded by a page. So a problem of the file format
or of the web is for the assistant to catch, and when one reaches the
owner it is said as what a user of popnei_web or of Vavilov Explorer
would see or be unable to do.

The skills are under `.claude/skills/` and the subagents under
`.claude/agents/`. The `writing` skill is read before anything a person
will read is written, and a document, a spec or a GitHub issue goes to
the `first-reader` subagent before it is handed over. The work goes in
this order: a spec, under `docs/specs/`, as the `writing-specs` skill
says; a plan, as `writing-plans` says; the code, as `coding` says. A
change to what `docs/objectives.md` or `docs/architecture.md` decide is
shown to the owner before the spec that needs it.

A session that writes anything in the repository, a spec, a plan, a skill
or code, works in a git worktree and a branch of its own, under
`.claude/worktrees/`, which it makes before its first edit: two sessions
that edit the main checkout at once leave their changes mixed in the same
files, as happened in popnei on 21 September 2026. An implementation plan
is carried out the same way, as the `following-plans` skill says. Nothing
is merged into `main`, nothing is pushed, and no release is made, without
the owner's order.

## The two contracts

Two programs are written against table_io, each against a part of it
that does not change unseen (`docs/architecture.md`, section 8).

- popnei_web's light worker is written against what the package
  declares: the functions it exports, among them `importTable` and
  `convertColumn`, what they return, and the `init` that loads the wasm.
  A change to it is a change in both projects: a new release here, and
  in popnei_web a change of the code that reads the package and a new URL
  in its `package.json`, in the same piece of work.
- Vavilov Explorer is written against the public Rust interface of the
  library, at a revision of `main` it pins. A change to it is a change in
  Vavilov Explorer in the same piece of work, made from its own session
  when it moves its pin, and the commit here says what changed for a
  caller; a value read differently is named in the commit message too.

The rules by which the cells become a table, the header, the missing
values, the first column, the types and the refusals, are table_io's,
taken from popnei_web's `docs/specs/worker/individuals.md`, and the same
for both applications. What each application makes of the table is its
own: what a column means, a categorical column or the roles of
popnei_web's analyses, how a refusal is worded, the largest file it
accepts, the defaults of an export. A session here that needs one of
those changed says so and does not change it here, nor in the other
repository.

## Replies in chat

The owner reads a reply to decide something or to learn something they
need for their own work. They did not see the session. The principles of
the writing skill, `.claude/skills/writing/SKILL.md`, hold in chat, and
these are the ones that fail most often there:

- A reply opens with a sentence that says what it is about, also when it
  follows a question: what was asked or what was being worked on, with the
  options or the things named by what they are. The owner may come back to
  it after hours of other work. The answer comes next, in a full sentence.
  When several things are true, the one that changes what the owner does
  goes first.
- A longer reply is sketched before it is written, as the writing skill
  says, so that no term is used before the sentence that explains it.
- A reply holds the decisions that are needed from the owner and what they
  need to know. How the work went stays out, unless it changes what they
  do next.
- A reply carries the numbers the answer turns on and leaves the rest of
  what was measured for a document, with a line that says where it is or
  that it can be written. A reply that needs headings is usually a
  document.
- A request for a decision has the options, what each one costs, and a
  recommendation. What can be decided without the owner is decided, and is
  mentioned only when they need to know of it.
- No name before its explanation. Labels from the session, the names of
  files, types and functions, and the words of the file format and of the
  web, are not words the owner has, unless the owner used them first.
- A number where an adjective would go, with what it was measured on. A
  comparison has both sides in the same units and says which is better.
- Nothing about the reply itself, and nothing about how interesting or
  important a thing is.
- What failed, or was not done, is said as plainly as what worked.
