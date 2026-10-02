# table_io, the wasm package

The WebAssembly build of table_io, which reads the bytes of a CSV, a TSV
or an xlsx into a table of typed columns. It is written for popnei_web's
light worker, the Web Worker that reads the user's files away from the
thread of the page, and needs nothing of popnei_web. It is what
wasm-bindgen, the tool that makes Rust functions callable from
JavaScript, generates from the Rust crate `crates/table_io_js`, with no
code of its own around it. This file says all a caller needs; the rules
in full are in the repository, github.com/JoseBlanca/table_io, in
`docs/specs/import.md`, `docs/specs/text-files.md`,
`docs/specs/values.md` and `docs/specs/package.md`.

## Loading it

`init`, the default export, downloads and compiles `table_io_bg.wasm`; it
is awaited once, before anything else is called. Called with no argument
it fetches the `.wasm` from beside `table_io.js`, as a browser or a
bundler resolves it. Under node, whose `fetch` does not read a file, give
it the bytes: `init({ module_or_path: bytes })`.

## What a table is

The first row of the file that is not blank, every cell empty, is the
header, and gives the names of the columns. The first column is the
**column of the names**, which names the individuals, one for each row
below the header, none empty and no two the same; it has no type and is
kept apart from the other columns. Each of the other columns has a name
in the header and one of four types, the first of these, in this order,
that holds every value it has, its missing values left out:

- `"integer"`, a whole number from −2^63 to 2^63 − 1, written with no
  decimal mark and no exponent in a text file: `12`, `-3`, `007`;
- `"float"`, a finite number, with the decimal mark of the file: `1,75`,
  `1.2E-03`;
- `"boolean"`, `TRUE` or `FALSE` in any case: `true`, `False`;
- `"text"`, anything else, and a column whose every cell is missing.

A **missing** value is an empty cell, `NA` or `-`, exactly; `na` and
`N/A` are texts. The spaces at the ends of a cell are removed before it
is read. Blank rows are skipped wherever they are. In an xlsx, a number
cell whose value is whole is an integer, `1` shown as `1.0` among them,
and the cells are read from the first visible sheet.

## importTable

`importTable(bytes, max_bytes, max_cells, encoding, separator, decimal)`
reads a file and returns a `TableRead`, which holds a table or a
refusal. Its format, a text file or an xlsx, is found from its first
bytes, never from the name of the file.

- `bytes`: the file, a `Uint8Array`, as `new Uint8Array(await
  file.arrayBuffer())` gives it. An `ArrayBuffer` or a `DataView` given in
  its place is read as no bytes, an empty file, with no `Error`;
  TypeScript refuses it from the declarations, JavaScript does not.
- `max_bytes`: the largest file accepted, in bytes, a whole number from
  0 to 2^53; a larger one is the refusal `tooLarge`. The bytes are
  already in the memory of the wasm by then, which never shrinks, so a
  caller with a `File` checks its `size` before reading it.
- `max_cells`: for an xlsx, the largest number of cells of the
  **rectangle** of its sheet, from the first row and column that hold a
  value to the last ones, a whole number from 0 to 4,294,967,295; a
  larger one is the refusal `sheetTooLarge`, given as soon as the
  rectangle passes it. A text file is bounded by `max_bytes` alone.
- `encoding`: `""` to find it, `"utf-8"` or `"windows-1252"`. Found, it is
  UTF-16 when the file starts with the byte order mark of UTF-16, the
  bytes `FF FE` or `FE FF`, which Excel's "Unicode Text" writes, whatever
  is set; UTF-8 when the file starts with the mark of UTF-8, `EF BB BF`,
  which Excel's "CSV UTF-8" writes, or its bytes are
  valid UTF-8; and Windows-1252 otherwise, which Excel writes for "CSV" in
  Spanish and the other languages of Western Europe. A character that
  cannot be decoded becomes `�`, and the field `undecodedLine` of the
  result gives its line, as below; the
  file is refused for its encoding only when it is UTF-16 cut in the
  middle of a character, `cutShort`.
- `separator`: `""` to find it, `"tab"`, `"semicolon"` or `"comma"`.
  Found, it is the one of the three that gives every row as many cells as
  the header, and of those the one that gives the header the most cells,
  the tab first, then `;`, then `,` on a tie.
- `decimal`: `""` to find it, `"point"` or `"comma"`. Found, it is the
  point when the separator is `,`; otherwise the comma when more cells
  below the header, outside the column of the names, are numbers written
  with a comma than with a point, and the point otherwise.

The three options are for a text file. For an xlsx they are not used, but
they are still checked, and one that is not among the strings above
throws an `Error`.

### A table

The fields of a `TableRead` are read-only properties, `read.refusal`,
and the columns are read with methods, `read.columnType(index)`.
`refusal` is `""` for a table. Then:

- `format` is `"text"` or `"xlsx"`;
- for a text file, how it was read: `encoding`, `"utf-8"`, `"utf-16"` or
  `"windows-1252"`; `separator`, `"tab"`, `"semicolon"` or `"comma"`;
  `decimal`, `"point"` or `"comma"`; and `undecodedLine`, the line of the
  first character that could not be decoded, or `undefined` when every
  character was;
- for an xlsx, `sheet`, the name of the sheet read, as its tab shows it;
  `encoding` and `separator` are `""` and `decimal` is `"point"`, the mark
  a text cell of an xlsx is read with;
- the column of the names: `namesHeader`, its name in the header, which
  may be empty; `namesNumber`, its column in the file, from 1; and
  `names`, an array of strings, one for each row;
- `numColumns`, the number of the other columns.

Each of the other columns is read by its **index**, from 0 to
`numColumns` − 1, in the order of the file, the column of the names not
counted: index 0 is the column right after it. For each:

- `columnName(index)`, its name in the header;
- `columnNumber(index)`, its column in the file, from 1, as the user
  sees it: in a file whose names are in column 1, index 0 is column 2,
  and in an xlsx whose table starts at column C, the names are column 3,
  `namesNumber`, and index 0 is column 4;
- `columnType(index)`, `"integer"`, `"float"`, `"boolean"` or `"text"`;
- `columnMissing(index)`, a `Uint8Array` with one entry for each row, 1
  for a missing value and 0 for a value, always filled;
- `columnIntegers(index)`, a `BigInt64Array`, so that an integer beyond
  2^53, which a JavaScript number cannot hold exactly, is exact;
  `columnFloats(index)`, a `Float64Array`; `columnBooleans(index)`, a
  `Uint8Array`, 1 for true and 0 for false; and `columnTexts(index)`, an
  array of strings. The one of the column's type has one entry for each
  row, a missing one 0, 0n or `""`, which `columnMissing` tells from a
  value; the other three are empty.

So the values of the fourth column after the names are read as:

```js
const index = 3;
const missing = read.columnMissing(index);
if (read.columnType(index) === "float") {
    const floats = read.columnFloats(index);
    for (let row = 0; row < floats.length; row++) {
        const value = missing[row] === 1 ? null : floats[row];
        // ...
    }
}
```

### A refusal

Otherwise `refusal` names why the file gives no table, and `format` is
the format found, `"text"` or `"xlsx"`, so that a line, a row or a column
is written as the user sees it: a **line** of a text file and a **row**
of an xlsx are counted from 1 as the editor and Excel number them, the
header included; a **column** is counted from 1 too, A being 1 in an
xlsx. The fields each kind needs are filled, as below, and every other
field is 0 or `""`, so a 0 in a line, a row or a column says it is not
filled. `text` is the one string field a refusal fills: a message, an
error of Excel or a name, as the table says.

| `refusal` | what the user gave | fields filled |
|---|---|---|
| `unreadable` | a file that cannot be read, a zip or a sheet damaged or cut short; `format` is `""`, since it is not known | `text`, the message of the zip reader, of calamine, the reader of xlsx, or of table_io, in English, for the console |
| `tooLarge` | a file larger than `max_bytes` | `size`, its bytes |
| `formatNotBuilt` | a format this build leaves out; the package has both, so it is not seen | none |
| `oldExcel` | an Excel 97–2003 workbook, an `.xls` | none |
| `encrypted` | an xlsx saved with a password | none |
| `notWorkbook` | a zip that holds no workbook, a `.docx` or a `.zip` of other files | none |
| `cutShort` | a text file in UTF-16 that ends in the middle of a character, most often cut on its way; `format` is `"text"` | none |
| `notText` | a file with a byte 0 that is not UTF-16, a gzipped or other binary file | none |
| `variantsFile` | a file of variants, a VCF | none |
| `empty` | a file with no row below its header, or no row at all | none |
| `emptySheet` | an xlsx whose first visible sheet holds no value | `sheet` |
| `cellError` | an xlsx with a cell holding an error calamine does not know | `text`, the error, such as `#GETTING_DATA` |
| `sheetTooLarge` | an xlsx whose rectangle passed `max_cells` | `sheet`; `row` and `column`, the first of the rectangle; `sheetRows` and `sheetColumns`, its size when it passed |
| `unclosedQuote` | a text file with a `"` never closed | `line`, where it opens; `separator` |
| `headerError` | an xlsx with an error of Excel, such as `#VALUE!`, in the header | `row`, `column`, `text`, the error |
| `unnamedColumn` | a column with values and no name in the header | `column` |
| `raggedRow` | a row of a text file with another number of cells than the header | `line`; `expected`, the cells of the header; `found`, the cells of the line; `separator` |
| `duplicateColumn` | two columns of one name | `text`, the name; `column` and `secondColumn` |
| `emptyIndividual` | a row with no name in its first cell | `line` for a text file, `row` for an xlsx |
| `duplicateIndividual` | one individual named in two rows | `text`, the name; `line` and `secondLine` for a text file, `row` and `secondRow` for an xlsx |

A CSV `id,a,b`, `x,1,2`, `y,3` gives `refusal` `"raggedRow"`, `format`
`"text"`, `line` 3, `expected` 3, `found` 2 and `separator` `"comma"`.

## convertColumn

`convertColumn(column_type, missing, integers, floats, booleans, texts,
to, decimal)` converts a column to another type, when the user changes
it. The column is given as a `TableRead` gives it: `column_type`, its
type, `"integer"`, `"float"`, `"boolean"` or `"text"`; `missing`, 1 or 0
for each row; and the array of its type, of as many entries, the other
three not read and given empty, `new BigInt64Array()`, `new
Float64Array()`, `new Uint8Array()` or `[]`. `to` is the type to convert to, among the
same four, and `decimal`, `"point"` or `"comma"`, the mark texts are read
and floats written with: for a column of an import, the `decimal` it
gave.

A value converts by the rules of "What a table is": a text to an integer
when it is a whole number, to a float when it is a number with `decimal`,
to a boolean when it is `TRUE` or `FALSE`; an integer to a float always,
the nearest float beyond 2^53; a float to an integer when it is whole and
in range; anything to a text always; a boolean to a number, or a number
to a boolean, never, unless every value is missing. A missing value stays
missing and never fails.

It returns a `Conversion`, whose fields are read-only properties and
which is freed as a `TableRead` is, below. With `numFailed` 0, the
column converted, as `missing`, `integers`, `floats`, `booleans` and
`texts`, the one of the
type `to` filled as in a `TableRead` and the other three empty.
Otherwise no value is converted: `numFailed` is how many values do not
convert, `firstRow` the row of the first among the rows of the table,
the first below the header being 1, not the line or row of the file, and
`firstText` its text, and the five arrays are empty. A text column `"1"`,
`"n.d."` converted to `"float"` gives `numFailed` 1, `firstRow` 2 and
`firstText` `"n.d."`.

## The rules of a value

`isMissing(text)`, `parseInteger(text)`, a `bigint`, `parseFloat(text,
decimal)`, `parseBoolean(text)` and `floatText(number, decimal)`, the
rules above that the import reads by, for a value the user types; the
three parses give `undefined` for a text that holds no such value, and
`floatText` writes a float as JavaScript's `String` does, with the mark
`decimal`.

## What throws, and freeing

A function throws an `Error` only for a defect of the caller: a limit
that is not a whole number in its range, an option or a type that is not
one of its strings, an index of a column that is not a whole number from
0 to `numColumns` − 1, or a column given to `convertColumn` whose arrays
do not have one entry for each row or whose `missing` or `booleans` hold
an entry other than 0 or 1. A file refused, or that cannot be read, is
never thrown.

A `TableRead` and a `Conversion` live in the memory of the wasm.
JavaScript's garbage collector frees them too, through a
`FinalizationRegistry` that wasm-bindgen registers, but at a time nobody
chooses, which may be after the next file is read: so call `free()` once,
in a `finally`; a second `free()`, or a field read after it, throws. Each
read of a field or of a column copies it out whole, a new array each
time, so read each once into a variable.

The limits below are popnei_web's own on 2 October 2026, 20,000,000
bytes, its `MAX_INDIVIDUALS_FILE_BYTES`, and 2,000,000 cells, its
`MAX_SHEET_CELLS`; a caller sets its own.

```js
import init, { importTable } from "table_io";

await init();
const read = importTable(bytes, 20_000_000, 2_000_000, "", "", "");
try {
    if (read.refusal === "") {
        const names = read.names;
        for (let index = 0; index < read.numColumns; index++) {
            const columnType = read.columnType(index);
            const missing = read.columnMissing(index);
            // ...
        }
    } else {
        // read.refusal, read.format and the fields of the table above
    }
} finally {
    read.free();
}
```

The declarations, `wasm/table_io.d.ts`, are the contract with popnei_web.
They are kept in git as `test/table_io.d.ts`, and `npm test` fails when
the ones generated differ from them.

## How it is installed

popnei_web installs it by the URL of the `.tgz` attached to a release of
table_io on GitHub, a pre-release on a tag `js-v0.2.0-dev.1`, then `dev.2`
and on, each tag used once:

```
npm install https://github.com/JoseBlanca/table_io/releases/download/js-v0.2.0-dev.1/table_io-0.2.0.tgz
```

While the two are changed together, the local build is packed with
`npm pack` here and installed in popnei_web with `npm install --no-save`
and the absolute path of the `.tgz`, never with a link, whose `.wasm`
popnei_web's development server refuses to serve.

## Building and testing it

With Rust, the target `wasm32-unknown-unknown` and the command line of
wasm-bindgen 0.2.128, the version the crate pins:

```
npm run build   # the crate in release, then wasm-bindgen into wasm/
npm test        # node's own test runner; no npm dependency
npm pack        # builds, tests, copies LICENSE and THIRD_PARTY_LICENSES.md from the root, then table_io-0.2.0.tgz
```

`THIRD_PARTY_LICENSES.md` holds the license of each crate compiled into
the `.wasm`, which their licenses ask to go with every copy; it is written
again when a dependency changes, from the list the command at its top
gives.

The build writes the paths of the home folder into the `.wasm` as `~`
(`--remap-path-prefix`, in the script's `RUSTFLAGS`), so that the paths of
the sources of the dependencies, which the `.wasm` keeps for its messages,
do not carry the name of the account that built it.
