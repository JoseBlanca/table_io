# table_io: the values and their types

Written on 2 October 2026, before any code, from `docs/architecture.md`,
sections 2 and 3, which the owner approved that day, and from the rules
of popnei_web's `docs/specs/worker/individuals.md`, "The decimal mark and
the numbers" and "The types of the columns", which become table_io's
where this spec does not change them. It gives the module `value` and the
module `types` of the library crate: what a text means, which of the
four types a column of an import gets, and how a column is converted to
another type when the user asks. `specs/import.md` calls this module for
every column of a table, and an application calls it directly to read a
value or to convert a column. It is behind neither of the cargo features
of the library, `csv` and `xlsx`, so that every build has it. An xlsx is
read by calamine, the Rust library of spreadsheets, which gives each cell
of a sheet as empty, a text, a number or a boolean, a **number cell**, a
**text cell** and a **boolean cell** below (`specs/read.md`).

## What it does

A user of Vavilov Explorer imports `plants.csv` and sees each column with
a type beside its name: `height` a float, `seeds` an integer, `fertile`
a boolean, `region` text. What goes wrong because of this module is seen
there, or later:

- a column of heights written `1,75` in a Spanish CSV taken as text,
  where a decimal mark read wrong makes every value fail to be a number;
- a column of counts taken as floats, so that the user's `12` shows as a
  float and the integer column a later view needs is not there;
- a text `n.d.` in a column of numbers, which makes the column text, as
  it must, and which the user finds through the conversion's report,
  "1 value is not a number, 'n.d.' in row 40";
- a conversion that changes a value without a word, an integer
  9,007,199,254,740,993 made the float 9,007,199,254,740,992.

### The four types

table_io gives each column but the first one of four types, taken from
Arrow, the format of columns in memory that Parquet and polars read, as
the owner decided on 2 October 2026:

| type | holds | Arrow's |
|---|---|---|
| integer | a whole number from −2^63 to 2^63 − 1 | `Int64` |
| float | a finite number of 64 bits | `Float64` |
| boolean | true or false | `Boolean` |
| text | a text of Unicode | `Utf8` |

Each value of a column is one of its type or **missing**, which is kept
apart from the values. What a column means beyond its type, a
classification of populations, a code, a binary trait, is the
application's (`docs/architecture.md`, section 2).

### What a text holds

The rules a text is read by, the same at the import and at a conversion,
and public so that an application reads a value as the import did. Each
takes the text as it is: the import removes the spaces at the ends of a
cell before it asks, and a caller with a text typed by the user does the
same if it wants that.

- **Missing**: the empty text, `NA` and `-`, exactly. `na`, `N/A`,
  `NaN` and ` NA` are not.
- **A whole number**: an optional sign, `+` or `-`, then one digit or
  more, and nothing else, whose value is from −2^63 to 2^63 − 1. `12`,
  `-3`, `+5`, `007` and `-0` are, the last two 7 and 0; `12.0`, `1e3`,
  `1 000` and `9223372036854775808` are not. No decimal mark is involved.
- **A number**, with a decimal mark, the point or the comma: an optional
  sign, then digits with at most one decimal mark among or around them,
  at least one digit in all, `12`, `1,75`, `,5`, `5,` with the comma;
  then optionally an exponent, `e` or `E`, an optional sign and one digit
  or more, `1,2E-03`; and a value that is finite. Nothing else is: no
  thousands separator, `1.234,5` is not a number with either mark; no
  space, no `%`, no `Inf`, no `NaN`; `1e999` is not, being infinite; `.`
  and `e5` are not. A whole number is a number too. The value is the
  float nearest to the decimal number written, as Rust's `str::parse` for
  `f64` gives it once the comma is made a point; that parser also takes
  `inf`, `NaN` and `infinity`, which is why the text is checked against
  this rule before it is parsed (tried with Rust 1.98.0 on 2 October
  2026).
- **A boolean**: `TRUE` or `FALSE` in any case of their ASCII letters,
  `true`, `False`. Not `yes`, `1`, nor `VERDADERO` and `FALSO`, which a
  Spanish Excel writes into a CSV, as the owner decided
  (`table_io-needs.md`, section 3).

### The text of a value

A value that is not a text is written as text in three places: a name of
the header or of the first column that an xlsx holds as a number or a
boolean (`specs/import.md`), a value of an xlsx in a column that is text,
and a conversion to text.

- **An integer**: its digits, with `-` before a negative one, `12`,
  `-3`.
- **A float**: as JavaScript's `String` writes a number, `1`, `1.5`,
  `0.30000000000000004`, `1e+21`, `1e-7`, the decimal mark then made the
  one of the read or the conversion, `1,5` with the comma. JavaScript
  writes the shortest digits that read back as the same float, and lays
  them out plainly from 10^−6 to below 10^21 and with an exponent outside
  (ECMAScript's `Number::toString`); `-0` is `0`, and a negative float is
  `-` before the text of its absolute value. Rust's `{:e}` gives the
  shortest digits too, `1.2345678901234568e20`, and its `{}` lays every
  digit out plainly, `1000000000000000000000` for 10^21 (tried with Rust
  1.98.0 on 2 October 2026), so table_io takes the digits and the
  exponent of `{:e}` and lays them out as ECMAScript does. The two
  differ when the float lies exactly halfway between two strings of
  shortest digits: ECMAScript takes the one whose last digit is even, and
  Rust's `{:e}` the one above, so that 100000000000000.125 is
  `100000000000000.12` in JavaScript and `…13` from `{:e}`; 712 of
  1,209,654 floats tried in the review of 2 October 2026 differed so,
  each in its last digit, both texts reading back as the same float.
  table_io takes JavaScript's: when the exact decimal value of the float,
  which `{:.767e}` writes in full, is the halfway point between the
  digits of `{:e}` and the same digits less one in the last place, and
  the last digit of `{:e}` is odd, it takes the digits less one. The form is
  JavaScript's because popnei_web wrote it so, and a name a user saw in
  popnei_web stays the same. The layout, from the shortest digits `d`, k
  of them, and the exponent n of `{:e}` plus 1, so that the number is
  0.d × 10^n:
  - when k ≤ n ≤ 21, the digits and n − k zeros, `100`;
  - when 0 < n ≤ 21, the first n digits, the mark, the rest, `1.5`;
  - when −6 < n ≤ 0, `0`, the mark, −n zeros, the digits, `0.000001`;
  - otherwise the first digit, then the mark and the rest when k > 1,
    then `e`, `+` or `-`, and the absolute value of n − 1, `1e+21`,
    `1.5e-7`.
- **A boolean**: `TRUE` or `FALSE`, as Excel shows it, where popnei_web
  wrote `true` and `false`, as JavaScript's `String` writes them
  as the owner decided on 2 October 2026. The option not taken was
  `true`, which a name of popnei_web that is a boolean cell of an xlsx
  had been.

A float a caller made itself may be infinite or not a number, which no
import gives: its text is `Infinity`, `-Infinity` or `NaN`, as
JavaScript writes them, and the export refuses such a value
(`specs/export.md`).

### The type of a column

The import gives each column but the first the narrowest of the four
types that holds every value it has, its missing cells left out, in this
order:

1. **Integer**, when every value is a whole number by the rule above,
   and from an xlsx, a number cell whose value is whole and from −2^63 to
   2^63 − 1.
2. **Float**, when every value is a number, with the decimal mark of the
   read, or a number cell of an xlsx.
3. **Boolean**, when every value is a boolean by the rule above, or a
   boolean cell of an xlsx.
4. **Text**, otherwise, and for a column with no value, every cell
   missing, as the owner decided on 2 October 2026: no value says which
   type it is, and a text column converts to any other with no value
   failing.

The values are then made of that type: a whole number read from its
text, a number from its text with the decimal mark, a boolean from its
text; a number cell as it is, converted exactly to an integer when the
column is integer; and in a text column each value as its text, a cell
of an xlsx that is a number or a boolean written as above.

In a text file every value is a text, and a whole number is one written
with no decimal mark and no exponent, as the owner decided on 2 October
2026: `1.0` is a float, and a column of `1.0`, `2.5` is float, of `1`,
`2` integer. An xlsx holds a number and not how it was written, and
calamine gives no format of a cell, so a number cell 1, whether Excel
shows it `1` or `1.0`, is whole, and a column of number cells 1 and 2 is
integer, as the owner decided on 2 October 2026: "the column will be
integer only if all numbers in the column cells might be integers. So
1.1 is clearly not an integer, but 1.0 might be." The option not taken
was every number cell a float, which would have made a column of codes 1
to 12 float in an xlsx and integer in the CSV saved from it. A float
column of whole values exported as an xlsx comes back integer
(`specs/export.md`).

A text cell of an xlsx is read with the point, since the numbers of an
xlsx need no decimal mark and Excel takes `1,5` typed in a sheet in
Spanish as a number, so a text `1,5` there is one Excel did not take. A
column of an xlsx can mix cells: a number cell 1 and a text cell `2` are
an integer column of 1 and 2; a number cell 1.5 and a text cell `2,5` are
a text column of `1.5` and `2,5`; a boolean cell and a number cell are a
text column of `TRUE` and `1`, since the boolean is not a number and the
number not a boolean.

The guess depends on the values and not on their order, and is the same
for a table and the same table with its rows in another order.

### The conversion of a column

An application converts a column to another of the four types when the
user changes it (Vavilov Explorer's `docs/design.md`, section 6). The
conversion reads each value by the rules above, with the decimal mark it
is given, which for a column of an import is the one the import used, and
either gives every value converted or, when one value or more does not
convert, gives no column and says how many do not, and the first of
them, with its row, counted from 1 among the rows of the table, the first
below the header being row 1, and its text. That row is the table's and
not the file's: an application that shows a line or a row of the sheet
keeps that number itself. So the user reads "12
values are not numbers, such as 'n.d.' in row 40", in the application's
words. A missing value stays missing and never fails. The row is a
number of 32 bits, as a row of a sheet is, so that a first value past
row 4,294,967,295, in a column of more rows than that which a caller
made and no import gives, is named as row 4,294,967,295, a choice made
with the code on 2 October 2026.

| from \ to | integer | float | boolean | text |
|---|---|---|---|---|
| integer | itself | always, each value the nearest float | no value converts | always |
| float | each value whole and from −2^63 to 2^63 − 1 | itself | no value converts | always |
| boolean | no value converts | no value converts | itself | always |
| text | each a whole number | each a number with the decimal mark | each a boolean | itself |

Where the table says that no value converts, the conversion succeeds
only for a column whose every cell is missing, and fails for any
other. A boolean is not a number, and a number is not a
boolean, as the owner's rule of the booleans has it. A conversion to text
writes each value as "The text of a value" says, with the decimal mark of
the conversion.

From integer to float, a whole number beyond 2^53 that a float cannot
hold, 2^53 + 1 but not 2^53 + 2, becomes the nearest float, as a text
`9007199254740993` does when it is read as a float, and as Vavilov
Explorer's design has it, "from integer to numeric, always", as the
owner decided on 2 October 2026, such integers being beyond what the
tables need; the same holds where a text is read as a float and where the
guess makes a column of such numbers float because a value in it is not
whole. The option not taken was a value that is not exactly a float
failing to convert, and such a column made text. From float to integer, the range is checked against 2^63 itself
and not against `i64::MAX as f64`, which Rust rounds to 2^63, so that a
float of 2^63 does not convert and is never made the integer 2^63 − 1
by a cast that saturates.

## The Rust interface

In the library crate, `table_io`, at its root. The type of a column and
its values:

```rust
/// The four types a column of a table can have.
pub enum ColumnType { Integer, Float, Boolean, Text }

/// The values of a column, one for each row of the table, None for a
/// missing one.
pub enum ColumnValues {
    Integer(Vec<Option<i64>>),
    Float(Vec<Option<f64>>),   // finite in what an import gives
    Boolean(Vec<Option<bool>>),
    Text(Vec<Option<String>>),
}

impl ColumnValues {
    pub fn column_type(&self) -> ColumnType;
    /// The number of rows.
    pub fn len(&self) -> usize;
}

/// The mark between the whole part and the decimals of a number.
pub enum DecimalMark { Point, Comma }
```

The rules of a text and the text of a float, for an application that
reads or writes a value as the import does:

```rust
/// Whether a text is a missing value: "", "NA" or "-", exactly.
pub fn is_missing(text: &str) -> bool;
/// The whole number a text holds, or None.
pub fn parse_integer(text: &str) -> Option<i64>;
/// The finite number a text holds with the decimal mark, or None.
pub fn parse_float(text: &str, decimal: DecimalMark) -> Option<f64>;
/// The boolean a text holds, TRUE or FALSE in any case, or None.
pub fn parse_boolean(text: &str) -> Option<bool>;
/// A float as JavaScript writes it, with the decimal mark.
pub fn float_text(number: f64, decimal: DecimalMark) -> String;
```

The conversion, and what it says when a value does not convert:

```rust
pub struct ConversionFailure {
    /// How many values do not convert, 1 or more.
    pub num_failed: u64,
    /// The row of the first, among the rows of the table, from 1.
    pub first_row: u32,
    /// Its text, as "The text of a value" writes it.
    pub first_text: String,
}

/// The values converted to `to`, read with `decimal`, or how many do
/// not convert.
pub fn convert_column(values: &ColumnValues, to: ColumnType, decimal: DecimalMark)
    -> Result<ColumnValues, ConversionFailure>;
```

The guess of the type is called by the import and is not public: an
application asks for a type by converting to it.

## The cases

- **A column of `001`, `002`, `010`** in a CSV, a code the user wrote
  with its zeros: integer, 1, 2 and 10, the zeros lost. A conversion to
  text gives `1`, `2`, `10`, since the integer holds no zeros. A user who
  needs the zeros writes the codes so that they are not numbers, `P001`.
  The first column, the names, keeps `001` as written
  (`specs/import.md`).
- **A column of `1` and `1.0`** in a CSV: float, 1 and 1.
- **A column of `0` and `1`**: integer, not boolean.
- **A column of `TRUE`, `false` and `NA`**: boolean, true, false and
  missing.
- **A column of `1,5` and `2` with the decimal mark the point**, a
  Spanish file read with the point: text. The import finds the comma for
  such a file (`specs/text-files.md`), and the user can set it.
- **A column of `9223372036854775807` and `9223372036854775808`**: float,
  since the second is past the integers; their floats are both 2^63.
- **A float column converted to text and back**: `1.50` is read as 1.5,
  written `1.5`, and read back as 1.5; the text is not the one of the
  file.
- **A column of number cells of an xlsx, 1 and 2.5**: float. The text of
  2.5 written with the comma, in a conversion to text with the comma, is
  `2,5`.

## How it is verified

With cargo test, natively, at the public functions above, each case a
literal input and the literal output.

At `is_missing`: `""`, `"NA"`, `"-"` are missing; `"na"`, `"N/A"`,
`"NaN"`, `" NA"`, `"--"` are not.

At `parse_integer`: `"12"` 12, `"-3"` −3, `"+5"` 5, `"007"` 7, `"-0"` 0,
`"9223372036854775807"` 9,223,372,036,854,775,807,
`"-9223372036854775808"` −9,223,372,036,854,775,808; `"12.0"`, `"1e3"`,
`"1 000"`, `"9223372036854775808"`, `""`, `"+"`, `"١٢"` (Arabic digits)
None.

At `parse_float`, with the comma: `"12"` 12, `"-1,75"` −1.75, `",5"`
0.5, `"5,"` 5, `"1,2E-03"` 0.0012, `"+,5e+2"` 50; `"1.234,5"`, `"1,5 "`,
`"Inf"`, `"inf"`, `"NaN"`, `"infinity"`, `"1e999"`, `"-"`, `"NA"`, `","`,
`"e5"`, `"1e"`, `"1.5"` None; `"1e-999"` 0. With the point: `"1.5"` 1.5, `"1,5"` None,
`"0.1"` the float 0.1.

At `parse_boolean`: `"TRUE"`, `"true"`, `"True"` true, `"FALSE"`,
`"false"` false; `"yes"`, `"1"`, `"VERDADERO"`, `"T"` None.

At `float_text`, with the point: 1 `"1"`, 1.5 `"1.5"`, 0.1 + 0.2
`"0.30000000000000004"`, 10^20 `"100000000000000000000"`, 10^21
`"1e+21"`, 1.2345678901234568 × 10^21 `"1.2345678901234568e+21"`,
10^−6 `"0.000001"`, 10^−7 `"1e-7"`, 1.5 × 10^−7 `"1.5e-7"`, −0 `"0"`,
5 × 10^−324 `"5e-324"`, the largest float `"1.7976931348623157e+308"`,
infinity `"Infinity"`, NaN `"NaN"`; with the comma, 1.5 `"1,5"` and
1.5 × 10^−7 `"1,5e-7"`; −1.5 `"-1.5"`, −1.5 × 10^−7 `"-1.5e-7"`; the
ties 100000000000000.125 `"100000000000000.12"` and 12345678901234.0625
`"12345678901234.062"`. Each of these is what node 26.8.2 prints for
`String(x)` on the owner's Mac, which the test of the package checks
again (`specs/package.md`).

At `convert_column`, each a column literal and the target:

| values | to | gives |
|---|---|---|
| text `"1"`, `"2"`, missing | integer | 1, 2, missing |
| text `"1"`, `"n.d."`, `"2"`, `"x"` | float | 2 failed, row 2, `"n.d."` |
| text `"1,5"`, `"2"` with the comma | float | 1.5, 2 |
| text `"TRUE"`, `"no"` | boolean | 1 failed, row 2, `"no"` |
| integer 3, missing | float | 3, missing |
| integer 9,007,199,254,740,993 | float | 9,007,199,254,740,992 |
| integer 9,007,199,254,740,994 | float | 9,007,199,254,740,994 |
| float 2, 3.5, with the point | integer | 1 failed, row 2, `"3.5"` |
| float 2^63 | integer | 1 failed, row 1, `"9223372036854775808"` |
| float −2^63 | integer | −9,223,372,036,854,775,808 |
| float 2, 3 | integer | 2, 3 |
| float 1.5 with the comma | text | `"1,5"` |
| boolean true, false | text | `"TRUE"`, `"FALSE"` |
| boolean true | integer | 1 failed, row 1, `"TRUE"` |
| integer 1 | boolean | 1 failed, row 1, `"1"` |
| missing, missing as integer | boolean | missing, missing |

The guess of the type is checked at the import, over whole tables
(`specs/import.md`, "How it is verified"), since it is not public: the
cases of "The type of a column" and of "The cases" above, each as a CSV
and as an xlsx written by the test where the format can hold it; and one
property, that a table and the same table with its rows shuffled give the
same types, over tables made by a generator of the tests with a fixed
seed, no dependency added.

## Open points

None. The owner decided the three points of the draft on 2 October 2026:
a whole number cell of an xlsx is an integer, a whole number beyond 2^53
becomes the nearest float, and a boolean is written as text `TRUE`; each
is written above where it applies.

## Not in this spec

- Which column is categorical, binary or a code, and the warning of a
  column of few whole numbers: the application's
  (`docs/architecture.md`, section 8).
- Which cells are missing in an xlsx beyond these rules, the seven errors
  of Excel, and the names of the first column, which are never missing:
  `specs/import.md`.
- How the decimal mark of a text file is found: `specs/text-files.md`.
- Dates, which are text, `2024-05-13`, as `specs/read.md` gives them.
