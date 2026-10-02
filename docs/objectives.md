# table_io: objectives

Revised on 2 October 2026, when the owner decided that xlsx_rs, the
reader of xlsx files of popnei_web, becomes **table_io**, a library that
reads and writes tables for two applications: popnei_web, the web
applications of popnei, and Vavilov Explorer, a desktop application of the
owner (Vavilov Explorer's `docs/design.md`, section 7). xlsx_rs had been
written on 28 September 2026 as a project of its own, "following the same
conventions and skills that popnei follows", and its first release,
`js-v0.1.0-dev.1`, is what popnei_web reads an xlsx with today. What
Vavilov Explorer needs of table_io is in its `docs/table_io-needs.md`, and
is taken here as the requirements of that side. The parts of table_io and
how it is released are in `architecture.md`, and what each module does in
its spec under `specs/`. The owner approves these goals before any spec
of table_io is written, and confirms or changes the target of speed of
goal 7.

table_io is a library in Rust that reads a table from a CSV, a TSV or the
first sheet of an xlsx file, and writes one as a CSV or an xlsx. A table
here is what both applications hold of the user's file: a row of names,
the header, over rows of values, the first column naming the
individuals, the plants or animals the table is about. Reading a file
goes in three steps. The **cells** come out of the file by the rules of
its format: the decoding, the separator and the quotes of a CSV, and the
sheet chosen, the dates and the merged cells of an xlsx. The **table**
comes out of the cells by rules that are the same for every format: the
blank rows skipped, the header, which cells are missing values, the first
column, and the refusals. The **types** of the columns come last: which
columns hold numbers, whole numbers, booleans, categories or text. The
first two steps are what both applications share. The third is not:
popnei_web gives its columns types of its own, identifier, binary,
continuous and categorical, which its analyses need, and Vavilov
Explorer gives them numeric, integer, text, boolean and categorical.

The two applications use the library in two ways. **Vavilov Explorer**,
whose backend is Rust, depends on the library by git, at a revision it
pins, and imports a file the user picks into a new project: it gives
table_io the bytes and gets the columns, typed, or a refusal; it calls
table_io again when the user changes the type of a column, so that the
values are read by the rules of the import; and it exports the table of a
project to a CSV or an xlsx. **popnei_web** runs in a browser, and takes
from table_io a **wasm package**: the library compiled to WebAssembly,
the code a browser runs beside JavaScript, with the JavaScript that loads
it and the TypeScript declarations of what it exports. Its light worker,
the thread of the tab that reads the user's files, downloads the package
the first time a user loads an xlsx, and reads only the cells of an xlsx
with it: popnei_web reads a CSV and makes the table in TypeScript, by the
rules of its `docs/specs/worker/individuals.md`, which become table_io's.
That stays so after this work: popnei_web keeps the release it has, and
whether it ever reads its CSV through table_io, which would make every
one of its users download the package, the owner has set aside. Each
format is a **cargo feature** of the library, a part of it that a user
of the crate can leave out when it is compiled, so that a package built
for popnei_web holds only the formats popnei_web reads with it.

## The goals, in order

1. **The table the user sees in their file.** In an xlsx, the cells as
   Excel shows them: a date is a date and not the number Excel stores, a
   value merged over ten rows belongs to the ten, and a text keeps its
   accents. In a CSV, the text as the program that saved it meant it: a
   file of Excel in Spanish, in Windows-1252 with `;` between its cells
   and a comma in its decimals, reads `España` and `1,75` without the
   user setting anything, and each guess, the encoding, the separator
   and the decimal mark, is reported so that the user can see it and set
   it. Each kind of cell is checked by a test with its value as a
   literal, over files written by the tests and over files the owner
   makes in Excel in Spanish and in English, in LibreOffice and in Google
   Sheets, so that what is checked is what those programs write.

2. **One reading in both applications.** A file gives the same table,
   the same missing values and the same refusal in popnei_web and in
   Vavilov Explorer, and an xlsx gives the table of the CSV Excel saves
   from it, but where a format holds more than text: an error of Excel
   such as `#N/A` is missing in an xlsx and a text in a CSV, and a
   number or a boolean cell of an xlsx is a number or a boolean where the
   CSV has its text. Until
   popnei_web reads its CSV with table_io, its rules exist twice, in its
   TypeScript and here, so the cases of popnei_web's spec are tests of
   table_io, with the same literals, and a difference between the two is
   a finding for both.

3. **A file it cannot read is refused with a reason the user can act
   on, and the application goes on.** A variants file picked by mistake,
   a row with a cell too few, a CSV saved with the name `.xlsx`, a file
   with a password, a table larger than the caller accepts: each is a
   refusal that names which it is and carries what the application needs
   to word its message, the line, or the row and the column as Excel
   names them, the separator used, the names involved. The words are each
   application's. table_io does not panic on any input: in WebAssembly a
   panic ends popnei_web's light worker and the read with it, and in
   Vavilov Explorer, whose release build stops the program at a panic, it
   ends the application and the user's unsaved work.

4. **What it writes reads back.** A table exported as a CSV, with any
   separator, decimal mark, encoding and text of a missing value the
   user chooses, or as an xlsx, and imported again, gives the same names,
   values and types, but where the owner's rule that tells a categorical
   column from a text one decides otherwise: a column is guessed
   categorical when some value appears in more than one row and it has
   fewer than 20 distinct values, so a categorical column of 20
   populations comes back as text, and a text column of a few repeated
   values comes back categorical. A value that could not read back, a
   character Windows-1252 does not have, or a text that the import would
   take for missing, empty, `NA` or `-`, whatever the user chose to write
   for a missing value, is a refusal that names its column and row, never
   a character replaced or a value changed without a word.

5. **Small, for the web.** The package is downloaded by a user of
   popnei_web in the middle of their work, so its size is measured at
   every release. The `.wasm` of the first release of xlsx_rs,
   `js-v0.1.0-dev.1`, is 300,646 bytes gzipped with `gzip -9`, measured
   on the owner's Mac on 29 September 2026, 0.30 MB, and the `.wasm` of
   popnei, which every user of popnei_web downloads, 0.71 MB gzipped.
   There is no limit; a release whose package grows names its size before
   and after in its notes. What
   table_io adds for Vavilov Explorer is not in the package unless
   popnei_web asks for it: the reading of a CSV, the types and the
   writers are left out of its build.

6. **Contracts that hold.** Two programs are written against table_io:
   popnei_web against the declarations of the package, Vavilov Explorer
   against the public Rust interface of the library. A change of either
   is a change in that application too, made together, and a release of
   the package is never moved once popnei_web names it.

7. **Fast enough for a desktop table.** Vavilov Explorer's tables have
   tens of thousands of rows. The target, proposed in
   `table_io-needs.md` and to be confirmed by the owner, is a CSV of
   100,000 rows and 50 columns imported in less than one second by a
   release build on the owner's Mac, the measurement giving the model of
   the Mac. Nothing has been measured. Faster than the target, speed is
   not worked on, and no goal before this one is given up to reach it.

## Non goals

- Not a general library of spreadsheets. It reads one sheet, the values
  of its cells and its merged ranges, and writes one sheet of values; not
  the formats, the styles, the formulas, the comments, the other sheets,
  nor anything of a workbook that is not the table. Vavilov Explorer
  writes to a new file, so nothing of another workbook has to be kept.
- Not each application's choices: the words of a message, the types
  popnei_web gives its columns, the largest file and table each accepts,
  which table_io takes from the caller with each read, and the defaults
  of an export, which Vavilov Explorer's export dialog chooses.
- Not a reader of `.xls`, `.xlsb` or `.ods`, although calamine, the Rust
  library of spreadsheets that reads the xlsx, reads them: they are
  refused, and popnei_web asks its users for an xlsx or a CSV. Not
  Parquet, which Vavilov Explorer writes into its project file itself,
  and no type for dates, which are text such as `2024-05-13` until a view
  of Vavilov Explorer needs one.
- Not published on crates.io or npm for now: Vavilov Explorer takes the
  library by git and popnei_web installs the package from a GitHub
  Release, by its URL.
- Not the wiring into the two applications. What table_io needs of them
  is written down here and given to the owner; their repositories are
  changed from their own sessions.

## How the work is done

As in popnei:

- A spec before the code, a plan before the work, and the owner's approval
  of each.
- Tests fail before the change they test, with their values as literals,
  and when a change cannot have such a test, the commit says so.
- Measure before claiming, and say what was measured on: the file, the
  machine, the program.
- Findings go on the GitHub issues, so that the reasoning survives.
- Commit messages: a lower case subject, a body with the why and the
  numbers.
