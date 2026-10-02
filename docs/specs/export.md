# table_io: the export of a table

Written on 2 October 2026, before any code, from `docs/architecture.md`,
section 4, which the owner approved that day, and from Vavilov Explorer's
`docs/table_io-needs.md`, section 6. It gives `export_table`, which takes
a table in the model of `specs/import.md`, the names of the individuals
and the typed columns, with the format and the choices of the user, and
gives the bytes of a new CSV or xlsx file, or a refusal. The CSV is
written by the module `csv`, behind the feature `csv`, and the xlsx by
the module `xlsx` with rust_xlsxwriter, behind the feature `xlsx`, which
the owner approved as a dependency of the library on 2 October 2026.
popnei_web does not export a table yet, so the package does not export
this function (`docs/architecture.md`, section 5); Vavilov Explorer is
its user.

## What it does

A user of Vavilov Explorer exports the table of a project, to open it in
Excel or to give it to a colleague, choosing in the application's dialog
a CSV with `;` and a decimal comma for a Spanish Excel, or an xlsx. What
goes wrong because of this module is seen when the file is opened, often
much later and by someone else:

- a value written so that it reads back as another, a text `NA` that
  every reader takes for a missing value, a float `1.75` written `1.75`
  in a file whose decimal mark is the comma;
- a character the encoding does not have, written as `?` by a writer
  that replaces it, a name that no longer matches;
- a cell holding the separator and not quoted, which shifts every value
  after it one column to the right;
- a file Excel cannot open, a text longer than a cell of Excel holds.

So the export writes nothing it cannot write so that table_io's import
reads it back as itself, and refuses instead, naming the column and the
row, as `objectives.md`, goal 4, asks.

### The table it takes

The column of the names and the other columns, each with its name and
its values, as an import gives them (`specs/import.md`, "The Rust
interface"); their numbers in the file are not used. A categorical
column of the application is given as text. The file written has the
header, the name of the names' column and then each column's, and a row
for each individual, its name and then its values. A row is counted from
1 among the rows of the table, the first below the header being row 1,
and the header is row 0; a column from 1, the names' being column 1, as
the user will find them in the file.

### A CSV

The caller sets each of these, and every combination is accepted, a
comma as both separator and decimal mark among them; the defaults are the
application's (`table_io-needs.md`, section 8):

- the separator, the tab, `;` or `,`;
- the decimal mark, the point or the comma;
- the encoding: UTF-8, UTF-8 with the byte order mark Excel writes in
  "CSV UTF-8", or Windows-1252, what Excel on Windows writes for "CSV";
- the text of a missing value: empty or `NA`.

A value is written:

- **an integer** as its digits, `-3`;
- **a float** as `specs/values.md`, "The text of a value", writes it, the
  shortest digits that read back as the same float, with the decimal mark
  chosen, and with the mark and a `0` after it when that form has neither
  a mark nor an exponent: 1 is `1.0` or `1,0`, 1.5 `1,5`, 10^21 `1e+21`.
  The owner decided on 2 October 2026 that a number written `1.0` is a
  float, so a float column of whole values reads back as a float;
- **a boolean** as `TRUE` or `FALSE`;
- **a text** as it is;
- **a missing value** as its text, empty or `NA`.

A cell is put in quotes, by RFC 4180, the standard of CSV, as Excel
writes them, when it holds the separator, a `"`, a line break, `\r` or
`\n`, or a space at its start or its end, or a tab there when the
separator is not the tab, since the import removes those outside quotes;
a `"` inside is written `""`. So with the comma as both separator and
decimal mark every float with decimals is quoted, `"1,5"`. Every line
ends with `\r\n`, as Excel writes them, the last one included.

### An xlsx

One sheet, named `Sheet1`, rust_xlsxwriter's name for a first sheet; the
names in the first row, from A1; a row for each individual below; a
float as a number cell; an integer as a number cell, when it is from
−2^53 to 2^53, which a number of Excel, a float, holds exactly, and
refused beyond (below); a boolean as a boolean cell; a text as a text
cell; a missing value as no cell. A float written as a number cell 1 is
read back as an integer, since an xlsx keeps the number and not how it
was written (`specs/values.md`, "The type of a column"). The name of the
sheet is `Sheet1`, as the owner decided on 2 October 2026; the option
not taken was a name the application gives, which would have needed a
refusal of the names Excel does not take.

## The refusals

Five are about the whole table and are checked first, in this order: a
format not built, a column of the wrong length, a table with no
individual, a table larger than a sheet of Excel, and a header that reads
back as a variants file. The others name a cell and are
found in the order of the file: the header first, then row by row, and
in a row from left to right; the first met is the one given.

- **A format not built**: the xlsx asked of a build without the feature
  `xlsx`, or a CSV of one without `csv`, as the import refuses such a
  file.
- **A table with no individual**: a header alone, which the import
  refuses as empty.
- **A column of the wrong length**: a column with another number of
  values than there are names, with its column, the names' count and its
  own. A defect of the caller, refused and not panicked at
  (`docs/architecture.md`, section 6).
- **An empty name** of a column other than the names': read back, the
  column would be refused as an unnamed column. The name of the names'
  column may be empty, as an import gives it, but for a table with no
  other column, whose header would then be a blank row, skipped, so that
  the first individual would become the header.
- **Two columns of one name**, the names' column among them, with the
  name and the two columns: read back, refused as a duplicate column.
- **An empty individual**, or **an individual in two rows**, with the
  rows: read back, refused as the import refuses them.
- **A text that reads back as missing**: a text value that is empty,
  `NA` or `-`, whatever the text of a missing value is, as
  `table_io-needs.md`, section 6, asks; and in an xlsx a text value that
  is one of the seven errors of Excel, `#N/A`, `#DIV/0!`, `#NAME?`,
  `#NULL!`, `#NUM!`, `#REF!` and `#VALUE!`, which the import of an xlsx
  takes for missing.
- **An error of Excel as a name**, in an xlsx: a name that is one of the
  seven, which the import refuses as a header error.
- **A text with spaces at its ends**, in an xlsx: the import removes the
  spaces and tabs at the ends of a text cell of an xlsx, and an xlsx has
  no quotes to keep them. A name or a value, with its column and row. A
  CSV quotes such a text and keeps it.
- **An integer too large for a number of Excel**, in an xlsx: an integer
  beyond −2^53 to 2^53, which a number cell would hold as the nearest
  float, 9,007,199,254,740,993 as 9,007,199,254,740,992, with its column
  and row, as the owner decided on 2 October 2026. The options not taken
  were a text cell of its digits, which Excel marks as a number stored as
  text, and the nearest float.
- **A float that is not finite**, infinite or not a number, which no
  import gives and a caller can make: no file holds it as a number.
- **A character the file cannot carry**, with its column, its row and the
  character, never replaced: in a CSV in Windows-1252, a character
  Windows-1252 does not have; in any CSV, U+0000, whose byte 0 makes the
  import refuse the file as not text, and a U+FEFF at the start of the
  first name of the header, which the import removes as a mark of
  UTF-8; in an xlsx, U+FFFE and U+FFFF, which rust_xlsxwriter writes as
  `_xFFFE_` and calamine reads back as those seven characters (tried with
  rust_xlsxwriter 0.99.1 and calamine 0.36.1 on 2 October 2026).
- **A text longer than a cell of Excel holds**, in an xlsx, 32,767
  characters, with its column, its row and its length. rust_xlsxwriter
  counts the characters of Unicode, and accepted 32,767 emoji in the
  review's trial; Excel counts, it is believed and has not been checked,
  the units of UTF-16, of which an emoji is two. So the export counts the
  units of UTF-16, the stricter, and refuses past 32,767.
- **A table larger than a sheet of Excel**, in an xlsx: more than
  1,048,575 rows below the header, Excel's 1,048,576 rows less the
  header's, or more than 16,384 columns, the
  names' among them, with the numbers of rows and of columns.
- **A header that reads back as a variants file**, in a CSV, a variants
  file being the VCF of the genotypes, which popnei_web's users may pick
  by mistake and the import refuses: the line of the header as written,
  after its quotes, its spaces and tabs at the start left out, starting
  with `##fileformat=VCF` or `#CHROM`, as the import of a text file reads
  it (`specs/text-files.md`). So an empty name of the names' column and a
  second name `#CHROM` with the tab as separator, `\t#CHROM`, is
  refused, and a first name ` #CHROM`, quoted, is not.

The export makes these checks itself before it writes, so that the
refusal names the place; an error of rust_xlsxwriter that comes all the
same, which the checks should leave none of, is given as the message of
an export that failed, for whoever reports the problem, and never as a
panic.

## The Rust interface

In the library crate, at its root:

```rust
pub enum ExportFormat {
    Csv(CsvExport),
    Xlsx,
}

pub struct CsvExport {
    pub separator: Separator,
    pub decimal: DecimalMark,
    pub encoding: CsvEncoding,
    pub missing: MissingText,
}

pub enum CsvEncoding { Utf8, Utf8WithMark, Windows1252 }

/// How a missing value is written in a CSV.
pub enum MissingText { Empty, Na }

/// A cell of the file written: the header is row 0, the first
/// individual row 1; the names' column is column 1.
pub struct CellPlace { pub column: u32, pub row: u32 }

pub enum ExportRefusal {
    FormatNotBuilt,
    NoIndividual,
    WrongLength { column: u32, expected: u32, found: u32 },
    EmptyName { column: u32 },
    DuplicateName { name: String, first_column: u32, second_column: u32 },
    EmptyIndividual { row: u32 },
    DuplicateIndividual { name: String, first_row: u32, second_row: u32 },
    ReadsAsMissing { place: CellPlace },
    ErrorAsName { place: CellPlace },
    SpacesAtEnds { place: CellPlace },
    IntegerTooLarge { place: CellPlace },
    NotFinite { place: CellPlace },
    CannotCarry { place: CellPlace, character: char },
    TextTooLong { place: CellPlace, length: u32 },
    TooLargeForSheet { rows: u32, columns: u32 },
    ReadsAsVariantsFile,
}

pub enum ExportError {
    Refused(ExportRefusal),
    /// rust_xlsxwriter's message, for whoever reports the problem.
    Failed(String),
}

/// The bytes of a new file holding the table.
pub fn export_table(names: &NameColumn, columns: &[Column], format: &ExportFormat)
    -> Result<Vec<u8>, ExportError>;
```

`Separator` and `DecimalMark` are `specs/text-files.md`'s and
`specs/values.md`'s, `NameColumn` and `Column` `specs/import.md`'s.

## What reads back

A table exported and imported again, the import of a CSV given the
separator, the decimal mark and the encoding the export used, gives the same names, in
the same order, and the same values, and each column the type it had,
but where its values read as a narrower type than its own
(`specs/values.md`, "The type of a column"):

- a text column whose every value is a whole number comes back integer,
  whose every value is a number float, whose every value is `TRUE` or
  `FALSE` in any case boolean, each value converted as
  `convert_column` converts it;
- a column whose every value is missing comes back text;
- from an xlsx, a float column whose every value is whole and from
  −2^63 to 2^63 − 1, the range of an integer, comes back integer; one
  with a whole value beyond that range, 10^20, comes back float, since no
  integer holds it.

These are the only changes: for each column, the values read back are
those `convert_column` gives of the values exported, to the type read
back, with the decimal mark of the CSV, and with the point for an xlsx.
A text holding U+FFFD, the replacement character, reads back as itself,
and the import reports the line of the first as a character not decoded,
which an application may show as a warning of a damaged file. The categorical column of an application, given as text, comes
back as text, and the application makes it categorical again by its own
rule.

## The cases

- **A text column of `001` and `002`**, kept as text in Vavilov Explorer:
  comes back integer 1 and 2, by the rule above. The user keeps the
  zeros by writing the codes so that they are not numbers, `P001`, or
  converts the column back to text in the application, which gives `1`
  and `2`.
- **A float column with the comma as separator and decimal mark**: every
  value with decimals in quotes, `"1,75"`, and a whole one `"1,0"`.
- **A name `height ` with a space at its end**: a CSV writes `"height "`
  and reads back `height `; an xlsx refuses it.
- **A text value `#N/A`**: a CSV writes it, and reads it back as the text
  `#N/A`; an xlsx refuses it.
- **A text with `€` in Windows-1252**: written as the byte `80`. A text
  with `ő`, which Windows-1252 does not have: refused.
- **An integer 2^60 in an xlsx**: refused, with its place; in a CSV
  written `1152921504606846976` and read back as that integer.

## How it is verified

With cargo test, natively, at `export_table`, each case a literal table,
the format and its choices, and the literal bytes of the CSV or the
refusal; for an xlsx, what rust_xlsxwriter wrote is read back by
`import_table` and by calamine, and checked as the cells of the sheet:
A1 the name, a number cell where a float was, a number cell 2^53 exact, a
boolean cell, no cell for a missing value.

- The CSV of the table `id`, `h`, `n`, `ok`, `pop` over two rows, `A`,
  1.5, 3, true, `P1` and `B`, missing, −2, false, `x;y`, with `;`, the
  comma, UTF-8 and the empty text of a missing value:
  `id;h;n;ok;pop\r\nA;1,5;3;TRUE;P1\r\nB;;-2;FALSE;"x;y"\r\n`; with `,`,
  the point, UTF-8 with the mark and `NA`: `EF BB BF` then
  `id,h,n,ok,pop\r\nA,1.5,3,TRUE,P1\r\nB,NA,-2,FALSE,x;y\r\n`; with `,`
  and the comma: `"1,5"` for 1.5.
- The floats 1, 0.1 + 0.2, 10^21, 1.5 × 10^−7 and −0 with the point:
  `1.0`, `0.30000000000000004`, `1e+21`, `1.5e-7`, `0.0`. −0 reads back
  as 0, the same number, and the round trip compares floats as numbers,
  −0 equal to 0.
- The quoting: `say "hi"` as `"say ""hi"""`; `a\nb` and `a\rb` quoted;
  ` x` and `x ` quoted; `a\tb` quoted only with the tab as separator, and
  `\tx` with every separator, holding the tab or starting with it.
- Windows-1252: `España` as the bytes of `Espa`, `F1`, `a`; `€` as `80`;
  `ő` refused, its column and row.
- Each refusal of "The refusals", once, with the place it names, and two
  in one table, giving the first in the order of the file.

The round trip of "What reads back", as a property over tables made by a
generator of the tests with a fixed seed, no dependency added: names
distinct and not empty, with accents, an emoji, a quote, the three
separators and spaces inside; columns of each of the four types, with
missing values, whole floats, integers beyond 2^53, refused in an xlsx, texts that are
numbers and texts that are not, U+0000, a U+FEFF at the start, U+FFFE,
an empty name of the names' column before `#CHROM`, and tables of no
individual; each exported as a CSV with every
combination of the separator, the decimal mark, the encoding, and the
text of a missing value, 36 in all, and as an xlsx, a table the export
refuses being made again without the value it names; each imported with
the options the export used, and compared column by column with what
`convert_column` gives of the values exported, to the type read back.

Made by the owner: one table exported as a CSV with `;` and the comma in
Windows-1252, and as an xlsx, opened in Excel in Spanish on the owner's
Mac, and what Excel shows in each column, its accents, its decimals and
its booleans, written in the report of the plan.

## Open points

None. The owner decided the two points of the draft on 2 October 2026:
the sheet is named `Sheet1`, and an integer beyond 2^53 is refused in an
xlsx; each is written above where it applies.

## Not in this spec

- The defaults of the dialog of an export, and a categorical column made
  text: the application's.
- The export from the package: when popnei_web exports a table
  (`docs/architecture.md`, section 5).
- Writing more than one sheet, the formats, widths or styles of a sheet:
  not needed (`objectives.md`, "Non goals").
