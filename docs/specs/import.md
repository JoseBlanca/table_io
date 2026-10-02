# table_io: the import of a table

Written on 2 October 2026, before any code, from `docs/architecture.md`,
sections 2, 3 and 7, which the owner approved that day, and from
popnei_web's `docs/specs/worker/individuals.md`, "The rows and the
cells", "The xlsx" and "The refusals and their words", whose rules
become table_io's here where this spec does not change them. It gives
`import_table`, the one function an application calls to read a file:
it takes the bytes of a CSV, a TSV or an xlsx and gives the table, its
columns typed, or a refusal. It finds the format, calls the module of
the format for the rows of cells (`specs/text-files.md`,
`specs/read.md`), makes the table of them by the rules both applications
share, the module `table`, and types each column by `specs/values.md`.
The library has a cargo feature for each format, `csv` and `xlsx`, both
on by default; the import is behind neither, and refuses a file of a
format whose feature a build leaves out.

## What it does

A user of either application picks a file and sees its table: the names
of the individuals down the first column, a column for each name of the
header, each with its type, and how the file was read. What goes wrong
because of this module is seen there:

- a header taken from the wrong row, a title line above the table, gives
  columns named by the title;
- an individual whose first cell is empty, or two rows of one name, would
  give a table where two rows cannot be told apart, so both are refused,
  naming the line;
- a cell `NA` read as the text `NA` makes a column of numbers text;
- an empty column that Excel leaves at the end of the header, `id;pop;;`,
  read as two columns of no name, refuses a file the user sees nothing
  wrong in;
- a refusal that names a line or a column other than the one the user
  will look at sends them to the wrong place.

### The format

The format is found from the first bytes of the file, not from its name,
as the owner decided on 2 October 2026 (`docs/architecture.md`, section
2):

1. A file that starts with the four bytes `50 4B 03 04`, `PK` and two
   bytes that open the first part of a zip, as `read_first_sheet` checks
   today, is an xlsx, read by `specs/read.md`.
2. A file that starts with the eight bytes `D0 CF 11 E0 A1 B1 1A E1`, the
   compound file of the old Office, is an old `.xls` or an xlsx saved
   with a password, which Excel encrypts inside such a file: it is of
   the format xlsx, and is refused as **old Excel** or **encrypted** by
   `specs/read.md`, "The refusals", or as a format not built in a build
   without the feature `xlsx`.
3. Any other file is a text file, read by `specs/text-files.md`, the
   empty file among them, and a CSV whose header starts with `PK`; an
   empty zip, `50 4B 05 06`, is not text there, having bytes 0.

Then a file larger than the caller's limit of bytes is refused as **too
large**, with its size and the limit, before anything more of it is
read.

So a CSV saved with the name `.xlsx`, which popnei_web refuses today with
words that ask the user to rename it, is read as the CSV it is, and the
table says it was read as a text file. A zip that holds no workbook, a
`.docx` or a `.zip` of other files, is refused as **not a workbook**, as
the owner decided on 2 October 2026, so that the application can say
that it is a zip and not an Excel workbook, which the user can act on.
It is found from the package relationships, the part `_rels/.rels` of
the zip that names its main part, which `specs/read.md` already reads:
a zip without them, or whose relationships name no workbook, is not a
workbook. The relationships name no workbook when none of them is of the
type `officeDocument` with a target, or when the folder of that target,
`xl/` in a workbook Excel saves, holds no part `workbook.xml`, its name
compared ignoring case, which is the part calamine reads as the workbook
whatever the target's own name: a `.docx` names `word/document.xml`, and
`word/` holds no `workbook.xml`. Without this refusal such a file was an unreadable one, with
calamine's message for a `.docx`, "File not found
'word/_rels/workbook.xml.rels'", and `specs/read.md`'s, "the file names
no workbook", for a zip with no relationships (tried on 2 October 2026),
which the application would have shown as a file that may be damaged:
the option not taken. A build without the feature of the format
found refuses the file as **a format not built**, naming the format, so
that the application can say which reader it lacks.

xlsx_rs's refusal of a file that is not a zip, `NotXlsx`, has no place
any more: such a file is a text file.

### The cells of an xlsx, as the table takes them

The rows of a text file come as texts, each with its line
(`specs/text-files.md`). The rows of an xlsx come as the **rectangle** of
its first visible sheet, the cells from the first row and column that
hold a value to the last ones, each cell empty, a text, a number or a boolean
(`specs/read.md`), each row with its row in the sheet and each cell with
its column in the sheet, A being 1. Before the rules below:

- a **text cell** has its spaces and tabs at the ends removed, as a cell
  of a text file outside quotes has, and is then empty when nothing is
  left;
- in the header and in the first column, whose cells are names, a number
  or a boolean is its text, `1`, `1.5`, `TRUE`, by `specs/values.md`, "The
  text of a value", so that an individual stored by Excel as the number 1
  is named `1`, and a column named by the year 2024 is `2024`.

### The rows

The same rules for the rows of every format:

- **A blank row**, one whose cells are all empty, `;;;` among them, which
  Excel writes for rows it once formatted, is skipped, wherever it is.
- **The header** is the first row that is not blank, and gives the names
  of the columns, as text: `NA` in the header is a column named `NA`.
  A file with no row that is not blank, an xlsx whose only values are
  spaces among them, which `specs/read.md` reads as a rectangle of
  texts, has no header and is refused as **empty**, as a text file with
  no line is (below, "The refusals").
- **The empty cells at the end of the header** whose columns hold no
  value in any row are dropped, as the owner decided on 25 September 2026
  for popnei_web: a header `id;pop;;` over rows of such cells is a header
  of two, which is what the user sees in Excel. The run dropped is the
  longest at the end of the header whose every cell is empty and whose
  columns hold, in every row, a missing cell, as the point on missing
  cells below defines it, or no cell at all.
- **A column with an empty name whose cells are all missing** is dropped
  too, wherever it is, since such a column has no values any more than
  one of empty cells, which Excel adds with a trailing separator. A
  column with an empty name and some value is refused, as an **unnamed
  column**, with its column. A run of empty cells at the end of the
  header that is not dropped, because one of its columns has a value in
  some row, is refused as an unnamed column before the lengths of the
  rows are checked: the user sees
  no name there, and a count of the header that took those cells in would
  be a number shown nowhere. `id;pop;;` over `a;1` and `b;2;3` is refused
  as column 3 having values and no name, and not as line 2 having 2 cells
  where the header has 3.
- **Two columns of one name** are refused, as a **duplicate column**,
  with the first name that repeats one before it, reading the header from
  left to right, its column and the column of the name it repeats, so
  that `id,a,b,b,a` is refused for `b`, columns 3 and 4, as popnei_web's
  `firstRepeated` has it, and so
  the user reads "the name 'height' is used by columns 4 and 9"
  (Vavilov Explorer's `docs/design.md`, section 5). The name of the
  first column is among the names compared, as in `firstRepeated`, so
  that `pop,pop` is refused for `pop`, columns 1 and 2. Names are
  compared exactly.
- **A row of a text file of another length than the header**, one with
  fewer cells than the header, counted as above, or with more whose cells
  past the whole header are not all empty, is refused, as a **ragged
  row**, with its line, the cells it has, the cells of the header, and
  the separator used. A row of an xlsx is always as long as the
  rectangle. The option not taken, by popnei_web, was to read a short row
  as ending in missing cells, as pandas does: a cell lost in the middle of
  a row would then move the values after it into the wrong columns, with
  no warning.
- **The first column names the individuals**, whatever its header says,
  an empty name included, as pandas' `to_csv` and R's `write.csv` write
  the column of the row names; it is never dropped, and an unnamed column
  is one of the other columns. Its cells are text, as written: `001` stays
  `001`, and `NA`, `-` and an error of Excel are the names `NA`, `-` and
  `#N/A`, since an individual is never missing. A row whose first cell is
  empty is refused, as an **empty individual**, with its line or row. An
  individual in two rows is refused, as a **duplicate individual**, with
  its name and the two rows. Names are compared exactly, `Ind_1` and
  `ind_1` being two individuals.
- **A missing cell**, in any other column, is an empty cell, `NA` or
  `-`, exactly, by `specs/values.md`, and in an xlsx a text cell that is
  one of the seven errors of Excel calamine knows, `#N/A`, `#DIV/0!`,
  `#NAME?`, `#NULL!`, `#NUM!`, `#REF!` and `#VALUE!`, exactly, once its
  spaces at the ends are removed. `specs/read.md` gives an error cell as
  its text, so a cell where the user typed the text `#N/A` is missing
  too.
  In a text file `#N/A` is a value, a text, as the owner decided for
  popnei_web on 28 September 2026, so the CSV Excel saves from a sheet
  gives the text where the xlsx gives a missing cell.
- **An error of Excel in the header**: a cell of the header of an xlsx
  that is one of the seven, once its spaces at the ends are removed,
  refuses the file, as a **header error**, with its row and column of the
  sheet and its text, as the owner decided on 29 September 2026 for
  popnei_web: a column named `#VALUE!` is a formula that failed, not a
  name the user gave. The first cell of the header, the name of the
  column of the individuals, is one of them.
- **Every other cell is a value** of its column, which `specs/values.md`
  types.

A refusal of an xlsx names its row by the row of the sheet and its column
by the column of the sheet, not of the rectangle, so that the user finds
them; the application writes a column with Excel's letters, column 4 as
D. A refusal of a text file names its line, counted as
`specs/text-files.md` counts them, and its column by its place in the
row, from 1.

### The limit of cells

The caller gives with each import the largest number of cells it accepts,
popnei_web 2,000,000 today. An xlsx is refused as **sheet too large** at
the first cell that makes the rectangle of its values larger, by
`specs/read.md`. A text file is bounded by its bytes alone, as
popnei_web has it today and as the owner decided on 2 October 2026, the
tables the two applications need being far smaller: its bytes bound its
cells to at most half of them and one, a cell and its separator taking
two bytes at least. The option not taken was the limit of cells for a
text file too, which with popnei_web's 2,000,000 would have refused a CSV
of 20 MB of short cells, `0,`, that it reads today.

### The types and the table

Each column but the first gets its type and its values by
`specs/values.md`, "The type of a column", with the decimal mark of the
read: for a text file the one set or found, for an xlsx the point. The
table is then given, with how the file was read.

## The refusals

When a file has several problems, the one reported is the first in this
order, popnei_web's with the kinds this spec adds:

1. too large;
2. a format not built;
3. the refusals of an xlsx before its rows, by `specs/read.md`: old
   Excel, encrypted, not a workbook, then the first met in the order of the file of an
   error calamine does not know and a sheet too large, and an empty first
   sheet once every cell is read; or those of a text file before its rows, by
   `specs/text-files.md`: cut short, not text, a variants file, and then,
   as the text is split, an unclosed quote;
4. a header error, an error of Excel in the header of an xlsx, the first
   by position;
5. no row below the header, **empty**, also for a text file with no line
   at all;
6. a column with no name and a value in the run of empty cells at the end
   of the header, the first by position;
7. a ragged row, the first by line;
8. a column with no name and a value elsewhere, then two columns of one
   name, the first by position;
9. then, row by row in the order of the file, an empty individual or a
   duplicate individual.

What each carries, for the words the application writes:

| refusal | carries | most likely |
|---|---|---|
| too large | the size, the limit, in bytes | the variants file picked by mistake |
| a format not built | the format | a build of the application without it |
| old Excel | none | a workbook of Excel 97–2003 |
| encrypted | none | a workbook saved with a password |
| not a workbook | none | a `.docx` or a `.zip` picked by mistake |
| empty sheet | the sheet | the table on another sheet than the first |
| cell error | the text of the error, `#GETTING_DATA` | a formula whose data never came |
| sheet too large | the sheet, the rectangle reached | a stray value far from the table |
| cut short | none | a UTF-16 file cut on its way |
| not text | none | a binary file picked by mistake |
| variants file | none | a VCF picked by mistake |
| unclosed quote | the line where the cell starts, the separator | a wrong separator |
| header error | the row and the column of the sheet, the error | a formula that failed |
| empty | none | a file of a header alone |
| unnamed column | the column | a value in a column with no name |
| ragged row | the line, its cells, the header's cells, the separator | a wrong separator, a cell lost |
| duplicate column | the name, the two columns | a name repeated by hand |
| empty individual | the line or the row | a row left without its name |
| duplicate individual | the name, the two lines or rows | a row copied twice |

## The Rust interface

In the library crate, at its root. What the caller gives:

```rust
/// What the caller accepts and sets for one import.
pub struct ImportOptions {
    /// The largest file, in bytes.
    pub max_bytes: u64,
    /// The largest number of cells of the rectangle of an xlsx; a text
    /// file is bounded by max_bytes alone.
    pub max_cells: u32,
    /// For a text file; ignored for an xlsx.
    pub text: TextOptions,
}
```

`TextOptions`, `TextRead` and `Separator` are `specs/text-files.md`'s, `ColumnValues`
and `DecimalMark` `specs/values.md`'s. What an import gives:

```rust
pub struct Table {
    /// The first column of the file, the names of the individuals.
    pub names: NameColumn,
    /// The other columns, in the order of the file.
    pub columns: Vec<Column>,
    pub read: HowRead,
}

pub struct NameColumn {
    /// Its name in the header, which may be empty.
    pub header: String,
    /// Its column in the file, from 1: in an xlsx, its column of the sheet.
    pub number: u32,
    /// One name for each row, none empty, no two the same.
    pub names: Vec<String>,
}

pub struct Column {
    /// Its name in the header, not empty, no two the same in a table.
    pub name: String,
    /// Its column in the file, from 1: in an xlsx, its column of the sheet.
    pub number: u32,
    /// One value for each row, as many as the names.
    pub values: ColumnValues,
}

pub enum HowRead {
    Text(TextRead),
    Xlsx { sheet: String },
}

pub fn import_table(bytes: &[u8], options: &ImportOptions) -> Result<Table, ImportError>;
```

What an import that gives no table gives. A refusal comes with the
format found, so that the application, which knows the format of a file
only from what the import says, writes a line or a row of the sheet, and
a column as its number or as Excel's letters, as the user will look for
it: a row of an empty or a duplicate individual is a line of a text file
and a row of the sheet of an xlsx, and a column of an unnamed or a
duplicate column the place in the row of a text file and the column of
the sheet of an xlsx.

```rust
pub enum ImportError {
    Refused { format: Format, refusal: Refusal },
    /// The zip crate's, calamine's or table_io's message, for whoever
    /// reports the problem.
    Unreadable(String),
}

pub enum Format { Text, Xlsx }

pub enum Refusal {
    TooLarge { size: u64, max_bytes: u64 },
    FormatNotBuilt,
    // An xlsx, specs/read.md:
    OldExcel,
    Encrypted,
    NotWorkbook,
    EmptySheet { sheet: String },
    CellError { error: String },
    SheetTooLarge { sheet: String, first_row: u32, first_column: u32, num_rows: u32, num_columns: u32 },
    // A text file, specs/text-files.md:
    CutShort,
    NotText,
    VariantsFile,
    UnclosedQuote { line: u32, separator: Separator },
    // Every format:
    HeaderError { row: u32, column: u32, error: String },  // of the sheet
    Empty,
    UnnamedColumn { column: u32 },
    RaggedRow { line: u32, expected: u32, found: u32, separator: Separator },
    DuplicateColumn { name: String, first_column: u32, second_column: u32 },
    EmptyIndividual { row: u32 },
    DuplicateIndividual { name: String, first_row: u32, second_row: u32 },
}
```

xlsx_rs's `read_first_sheet`, `Sheet`, `SheetCell`, its `Refusal` and its
`ReadError` become private to the library, the module `xlsx`, since an
application calls the import; its limit of cells stays a `u32`, which is
why `max_cells` is one.

## The cases

- **A file with no header, whose first row is an individual**: nothing
  can tell it, and the first individual becomes the names of the
  columns. The user sees it in the table.
- **A column all missing**: kept, with its name, a text column of
  missing values (`specs/values.md`).
- **A file of the names alone**, `only\nA\nB\n`: a table with the
  column of the names and no other column.
- **An individual named `NA`**, or `-`: the name `NA`.
- **An xlsx whose first column holds the numbers 1 and the text `1`**:
  two individuals named `1`, refused as a duplicate individual.
- **An xlsx column of dates**: text, `2024-05-13`, by `specs/read.md`. A
  year typed as a number is a number.
- **An xlsx column of heights with one text `n.d.`**: text, as in a CSV.
  With one error `#N/A` or `#DIV/0!` in its place: a float, that
  individual with no height.
- **A table whose header starts at C6 of the sheet**: the numbers of the
  columns are 3 and on, and a refusal names the rows of the sheet from 6.
- **A CSV saved with the name `.xlsx`**: the table, read as a text file.

## How it runs

An import holds the bytes the caller gave, the rows of cells of the file,
and the table, and lets each go when the next is made. For an xlsx the
rows are the rectangle of `specs/read.md`, at most the caller's limit of
cells; for a text file, the text and its cells (`specs/text-files.md`,
"How it runs").

## How it is verified

With cargo test, natively, at `import_table`, each case the literal bytes
of a text file, or an xlsx written by the test with rust_xlsxwriter, its
cells at the places said, and the literal table or refusal it gives. The
cases of popnei_web's tables at `readCsv` and at `readSheet`
(`docs/specs/worker/individuals.md`, "How it is verified"), each with the
answer of table_io, where the types are this project's and the refusals
carry what this spec adds:

| text, with `\n` for a line break | gives |
|---|---|
| `id,pop\nA,P1\nB,P2\nC,P1\n` | `,`, the point; names `A`, `B`, `C`; `pop` text `P1`, `P2`, `P1` |
| `id\tpop\nA\tP1\n` | the tab |
| `id;h\nA;1,5\nB;1,7\nC;1,9\n` | `;`, the comma; `h` float 1.5, 1.7, 1.9 |
| `id;h\nA;1.5\nB;1,7\nC;1,9\n` | `;`, the comma, two cells of it against one; `h` text `1.5`, `1,7`, `1,9` |
| `id,x\n001,1\n002,2\n003,3\n` | names `001`, `002`, `003`; `x` integer 1, 2, 3 |
| `id,st\nA,case\nB,control\nC,\nD,NA\n` | `st` text `case`, `control`, missing, missing |
| `id,s\nA,1\nB,01\nC,001\n` | `s` integer 1, 1, 1 |
| `id,n\nA,"x, y"\nB,"say ""hi"""\n` | `n` text `x, y`, `say "hi"` |
| `id,n\r\nA,1\r\n\r\nB,2\r\n`, and the same with `\r` alone | the table of `id,n\nA,1\nB,2\n`, the blank line skipped |
| `id;pop;;\nA;P1;;\n` | columns `id`, `pop` |
| `id;pop;;\nA;P1\nB;P2;NA\n` | `;`; columns `id`, `pop` |
| `id,x;pop;;\nA,1;P1\nB,2;P2;NA\n` | `;`, as `specs/text-files.md`, "The separator", finds it; the first column's header `id,x`, and `pop` |
| `id;pop;;\nA;P1\nB;P2;;x\n` | unnamed column 4: the header has four cells, the last two empty, and `x` is in the fourth |
| `id;pop;;\na;1\nb;2;3\n` | unnamed column 3, not a ragged row at line 2 |
| `id;pop;;x\nA;P1;;1\nB;P2\n` | ragged row, line 3, expected 4, found 2, `;` |
| `id,,pop\nA,NA,P1\nB,-,P2\n` | the names `A`, `B`, and `pop`, column 3 |
| `##fileformat=VCFv4.2\n#CHROM\tPOS\n` and `#CHROM\tPOS\tID\n1\t10\tx\n` | variants file |
| `\n##fileformat=VCFv4.2\n#CHROM\tPOS\n` and ` \t\r\n#CHROM\tPOS\n` | variants file |
| `id,pop\nA,P1\nB\n` | ragged row, line 3, expected 2, found 1, `,` |
| `id,pop\n,P1\n` | empty individual, line 2 |
| `id,pop\nA,P1\nA,P2\n` | duplicate individual `A`, lines 2 and 3 |
| `id,pop,pop\nA,1,2\n` | duplicate column `pop`, columns 2 and 3 |
| `id,Pop,pop\nA,1,2\n` | columns `Pop` and `pop` |
| `id,,pop\nA,1,P1\n` | unnamed column 2 |
| `id,pop\nA,"P1\nB,P2\n`, the separator set to `,` | unclosed quote, line 2, `,` |
| `id,pop\n` and the empty text | empty |
| `﻿id,pop\nA,P1\n` | the names headed `id`, not `﻿id` |
| `id;n\tx\nA;1\t2\n` | the tab, by the order of a tie |
| `id,pop\nA,P1\nB,P2,P3\n` | ragged row, line 3, expected 2, found 3, `,` |
| `id,n\nA,"x\ny"\nB,1,2\n` | ragged row, line 4, expected 2, found 3, `,` |
| `only\nA\nB\n` | `,`; the names `A`, `B` and no other column |
| `id,x\nA,1\nB,2\nC,3\n`, the limit of cells 7 | the table: a text file is bounded by its bytes alone |

| cells of the sheet, from A1 unless said | gives |
|---|---|
| `id`, `pop` / 1, `P1` / 2, `P2` / 3, `P1` | names `1`, `2`, `3`; `pop` text |
| `id`, `h` / `A`, 1.75 / `B`, `1.8` / `C`, 1.69 | `h` float 1.75, 1.8, 1.69 |
| `id`, `g` / `A`, 1 / `B`, `1` / `C`, 0 | `g` integer 1, 1, 0 |
| `id`, `h` / `A`, `1,75` / `B`, 1.8 / `C`, 1.7 | `h` text `1,75`, `1.8`, `1.7` |
| `id`, `ok` / `A`, TRUE / `B`, FALSE | `ok` boolean true, false |
| `id`, 2024 / `A`, 1 | the column `2024`, integer 1 |
| `id`, `pop` / `A`, ` P1 ` / `B`, `NA` / `C`, `  ` | `pop` text `P1`, missing, missing |
| `id`, `pop` / nothing / `A`, `P1` | the blank row skipped |
| `id`, `pop`, C1 empty / `A`, `P1`, C2 `"  "` | columns `id`, `pop`: the third column of the rectangle, with no name and a cell of spaces, dropped |
| at C5: `id`, empty, `pop` / `A`, `x`, `P1` | unnamed column 4, the column D of the sheet |
| at A5: `id`, `pop` / empty, `P1` | empty individual, sheet row 6 |
| `id`, `pop` / 1, `P1` / `1`, `P2` | duplicate individual `1`, sheet rows 2 and 3 |
| `id`, `pop` | empty |
| `id`, `#N/A` / `A`, `#N/A` | header error, row 1, column 2, `#N/A` |
| at C6: `id`, `pop`, ` #VALUE! ` / `A`, `P1`, 1 | header error, row 6, column 5, `#VALUE!` |
| `#REF!`, `pop` | header error, row 1, column 1, before empty |
| `id`, `pop` / `A`, `#VALUE!` / `B`, `P1` | `pop` missing, text `P1` |
| `id`, `h` / `A`, `#DIV/0!` / `B`, 1.5 / `C`, 1.6 / `D`, 1.7 | `h` float missing, 1.5, 1.6, 1.7 |
| `id`, `h` / `A` to `E` with `#NAME?`, `#NULL!`, `#NUM!`, `#REF!`, `#VALUE!` / `F`, 1.5 | `h` float, five missing and 1.5 |
| `id`, `h` / `#N/A`, 1.5 / `#REF!`, 2.5 | the names `#N/A` and `#REF!` |
| `id`, `ok` / TRUE, 1 | the name `TRUE` |

And, of this spec alone: a text file's `read` gives its encoding,
separator, decimal mark and line not decoded, an xlsx's its sheet; the
bytes of an xlsx with the limit of bytes one less than their length, too
large, with the size and the limit; a text file of `id,pop\nA,P1\n` read
by a build of the feature `xlsx` alone, `cargo test -p table_io
--no-default-features --features xlsx`, a format not built, text, and an
xlsx and `excel97.xls` by a build of `csv` alone, `cargo test -p table_io
--no-default-features --features csv`, a format not built, xlsx; at the
root of the workspace without `-p` the binding crate turns both features
on again; the compound files of `specs/read.md`'s tests, old Excel and
encrypted; a `.docx` written by the test as a zip of its parts, a zip
holding one CSV, and an xlsx without its part `xl/workbook.xml`, not a
workbook; a CSV whose header starts with `PK`, a
table; and the order of the refusals, each pair of the list
above that one file can hold, `#REF!` in the header of a sheet of one
row among them, giving the one first in the list.

The no-panic test of `docs/architecture.md`, section 12, at
`import_table`: a small xlsx written by rust_xlsxwriter and a small CSV
in Windows-1252 with a quoted cell over two lines, each cut short at
every length and with each of their bytes changed in turn to every one of
the 256, every copy giving a table, a refusal or an error, under
`std::panic::catch_unwind`.

The owner's files of `specs/read.md` and `specs/text-files.md`, each
imported and asserted as the table the owner says it shows.

## Open points

None. The owner decided the two points of the draft on 2 October 2026:
a zip that holds no workbook is refused as not a workbook, and a text
file is bounded by its bytes alone; each is written above where it
applies.

## Not in this spec

- How the cells of an xlsx and of a text file are read: `specs/read.md`
  and `specs/text-files.md`.
- What a value means and the types: `specs/values.md`.
- The words of the refusals: each application's.
- The export: `specs/export.md`.
- What the package gives JavaScript of a table: `specs/package.md`.
