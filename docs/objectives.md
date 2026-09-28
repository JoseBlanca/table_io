# xlsx_rs: objectives

Written on 28 September 2026, when the owner decided that the reader of
xlsx files of popnei_web, the web applications of popnei, is a project of
its own, named xlsx_rs, "following the same conventions and skills that
popnei follows". The parts of xlsx_rs and how it is released are in
`architecture.md`, and what it does, cell by cell, in its first spec,
`specs/read.md`.

xlsx_rs is a small library in Rust that reads the first sheet of an xlsx
file, the file Excel saves by default, into its cells. It is compiled to
WebAssembly, the code a browser runs beside JavaScript, and released as a
**wasm package**: the compiled library, the JavaScript that loads it and
the TypeScript declarations of the one function it exports. popnei_web
downloads that package the first time a user of its Individuals step
loads a table of individuals in xlsx, their names, populations and
traits, and a user whose table is a CSV never downloads it. The reading is
calamine's, a Rust library of spreadsheets; what xlsx_rs adds is the
choice of the sheet, the value of each kind of cell as a user of Excel
sees it, the merged cells, the refusals, and the package.

## The goals, in order

1. **The cells the user sees in Excel.** A date is a date and not the
   number Excel stores, a value merged over ten rows belongs to the ten,
   and a text keeps its spaces and its accents. Each kind of cell is
   checked by a test with the value as a literal, over files written by
   the tests and over files the owner makes by hand in Excel in Spanish
   and in English, in LibreOffice and in Google Sheets, so that what is
   checked is what those programs write and not what the writer of the
   tests thinks they write. Where the cell cannot be what Excel shows,
   because calamine does not give the format of a cell, the spec says
   what the user gets instead.

2. **A file it cannot read is refused with a reason the user can act on,
   and the tab goes on.** A CSV saved with the name `.xlsx`, a file with a
   password, an old `.xls`, a sheet whose values span more cells than
   popnei_web accepts, most often because of one stray value far from the
   table: each is a refusal that names which it is and carries what
   popnei_web needs to compose its message, the name of the sheet, the
   last row and column reached. xlsx_rs runs in popnei_web's light worker,
   the thread of the tab that reads the user's files apart from the page;
   a panic in WebAssembly ends that worker and the read with it, so the
   library does not panic, and it never holds more cells than the limit
   popnei_web gives it.

3. **Small.** The package is downloaded by a user in the middle of their
   work, so its size is measured at every release. Two crates written to
   try calamine on 27 September 2026 and thrown away after, each with
   calamine, wasm-bindgen and a few lines of their own, built as the
   package will be, gave a wasm of 0.30 MB gzipped; the package is expected
   near that, since calamine is almost all of it. popnei's own package,
   which every user of popnei_web downloads, is 0.71 MB gzipped.

4. **A contract that holds.** The declarations of the package are what
   popnei_web is written against. A change of them is a new release here
   and a change there, made together; a release is never moved once
   popnei_web names it.

Speed is not among the goals: a read runs once, on a table of a few
thousand rows, and popnei_web measures a sheet of 10,000 rows and 20
columns in its own tests. It becomes one if that measure shows the user a
wait.

## Non goals

- Not a general library of spreadsheets. It reads one sheet, the values of
  its cells and its merged ranges; not the formats, the styles, the
  formulas, the comments or the other sheets. What a second user would
  need is added when there is a second user.
- Not the rules of popnei_web. Which values are missing, the header, the
  types of the columns and the words of a refusal are popnei_web's. xlsx_rs
  gives what the file holds.
- Not a reader of `.xls`, `.xlsb` or `.ods`, although calamine reads them:
  the first release refuses them, and popnei_web asks its users for an xlsx
  or a CSV.
- Not published on crates.io or npm for now: popnei_web installs the
  package from a GitHub Release, by its URL.
- In the first release, no writing. An xlsx, and possibly the zip, of the
  report of popnei_web come with its stage 6, and where the zip goes is
  still open there.

## How the work is done

As in popnei:

- A spec before the code, a plan before the work, and the owner's approval
  of each.
- Tests fail before the change they test, with their values as literals,
  and when a change cannot have such a test, the commit says so.
- Measure before claiming, and say what was measured on: the file, the
  machine, the program.
- Findings go on the GitHub issues, once the repository is there, so that
  the reasoning survives.
- Commit messages: a lower case subject, a body with the why and the
  numbers.
