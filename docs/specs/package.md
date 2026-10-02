# table_io: the package for JavaScript

Written on 2 October 2026, before any code, from `docs/architecture.md`,
sections 1, 5, 8 and 10, which the owner approved that day with one
`.wasm` for every table. It gives the binding crate, `crates/table_io_js`,
and the package built of it, `js/table_io`, which popnei_web installs
from the URL of a release and imports in its light worker, the thread of
its tab that reads the user's files: what JavaScript calls, what it gets,
and the declarations wasm-bindgen writes of them, which are the contract
with popnei_web. The package replaces xlsx_rs's, whose one function,
`readXlsx`, it does not keep, as the owner decided on 2 October 2026;
what popnei_web changes to take it is in `docs/architecture.md`, section
9. The import, the conversion and the rules of a value are those of
`specs/import.md` and `specs/values.md`; the binding crate copies them
across and decides nothing.

## What it does

popnei_web's light worker imports the package the first time a user
loads a table, a CSV or an xlsx, awaits its `init`, which downloads and
starts the `.wasm`, calls `importTable` with the bytes of the file, its
limits and the options the user set, and reads the table or the refusal
from what it gets; when the user changes the type of a column it calls
`convertColumn`. What goes wrong because of this module is seen there:

- a field read before it was filled, a value of a column of another type
  than the one the column has, gives a table of zeros;
- an integer above 2^53 read as a JavaScript number, which cannot hold
  it, gives the user a value the file does not have, and popnei_web a
  value Vavilov Explorer does not give for the same file;
- a result not freed keeps its copy of the table in the memory of the
  wasm, which never shrinks, until the page is closed.

### How a table crosses to JavaScript

wasm-bindgen, the tool that makes a Rust function callable from
JavaScript, gives a struct of Rust to JavaScript as an object that stays
in the memory of the wasm, whose fields JavaScript reads through
functions that copy each field out, and which JavaScript frees with
`free()`. A struct that holds other structs gives each as one more such
object, to be freed too: in a trial of 2 October 2026, below, a table
holding its columns as objects gave each column as a copy every time the
list was read. So the table crosses as one object, `TableRead`, with its
columns read one by one by their index, each as arrays of JavaScript,
and the conversion takes and gives arrays: one `free()` for an import and
one for a conversion, which popnei_web calls in a `finally`.

A column is five arrays, of which the one of its type holds its values:

- `missing`, a `Uint8Array`, 1 for a missing value and 0 for a value, one
  for each row;
- `integers`, a `BigInt64Array`, for an integer column, so that an
  integer beyond 2^53 is exact, as the owner decided on 2 October 2026:
  a JavaScript number holds a whole number exactly only up to 2^53, and
  popnei_web turns a value into a number with `Number()` where it needs
  one;
- `floats`, a `Float64Array`, for a float column;
- `booleans`, a `Uint8Array`, 1 for true and 0 for false, for a boolean
  column;
- `texts`, an array of strings, for a text column.

The array of the type has one entry for each row, a missing one 0, 0, 0
or `""`; the other three are empty. The types and the options are
strings, written as below, and a string that is not one of them is a
defect of the caller, thrown as an `Error`.

| what | the strings |
|---|---|
| the type of a column | `"integer"`, `"float"`, `"boolean"`, `"text"` |
| the format found | `"text"`, `"xlsx"` |
| an encoding found | `"utf-8"`, `"utf-16"`, `"windows-1252"`; `""` for an xlsx |
| an encoding set | `""` to find it, `"utf-8"`, `"windows-1252"` |
| a separator | `"tab"`, `"semicolon"`, `"comma"`, and `""` to find it or for an xlsx |
| a decimal mark | `"point"`, `"comma"`, and `""` to find it or for an xlsx |

### What an import gives

`TableRead` holds a table or a refusal. With a table, `refusal` is `""`;
`format` the format found; `encoding`, `separator`, `decimal` and
`undecodedLine` how a text file was read, the line `undefined` when every
character was decoded; `sheet` the sheet of an xlsx; `namesHeader`,
`namesNumber` and `names` the column of the names; `numColumns` the
number of the other columns, which `columnName`, `columnNumber`,
`columnType` and the four arrays of a column give by their index, from
0; an index out of range is thrown as an `Error`.

With a refusal, `refusal` is its kind, in camelCase, and the fields its
words need are filled, the others 0 or `""`:

| `refusal` | fields filled |
|---|---|
| `tooLarge` | `size`, in bytes |
| `formatNotBuilt` | `text`, `"text"` or `"xlsx"` |
| `oldExcel`, `encrypted`, `cutShort`, `notText`, `variantsFile`, `empty` | none |
| `emptySheet` | `sheet` |
| `cellError` | `text`, the error, `#GETTING_DATA` |
| `sheetTooLarge` | `sheet`, `row` and `column`, the first of the rectangle, `sheetRows`, `sheetColumns` |
| `unclosedQuote` | `line`, `separator` |
| `tooManyCells` | `line` |
| `headerError` | `row`, `column`, of the sheet, `text`, the error |
| `unnamedColumn` | `column` |
| `raggedRow` | `line`, `expected`, `found`, `separator` |
| `duplicateColumn` | `text`, the name, `column`, `secondColumn` |
| `emptyIndividual` | `line` for a text file, `row` for an xlsx |
| `duplicateIndividual` | `text`, the name, and `line` and `secondLine` for a text file, `row` and `secondRow` for an xlsx |

A line, a row and a column are counted from 1, so a 0 says the field is
not filled. A file that cannot be read, `ImportError::Unreadable` of
`specs/import.md`, is thrown as an `Error` with its message, which
popnei_web writes to the console.

### The conversion and the rules of a value

`convertColumn` takes a column as its type and its five arrays, the type
to convert to and the decimal mark, and gives a `Conversion`: with
`numFailed` 0 the converted column, as its five arrays; otherwise
`numFailed`, the first row that fails, from 1, and its text, and no
values. `isMissing`, `parseInteger`, `parseFloat`, `parseBoolean` and
`floatText` are `specs/values.md`'s, `undefined` where the Rust gives
`None`.

## The exported functions and the declarations

The binding crate holds the structs and functions below and nothing
else; each function calls the library, and turns its types into these in
one function for each, a `match` that names every variant, as the
`coding` skill asks. What wasm-bindgen 0.2.128 declares of them in
`wasm/table_io.d.ts`, from a crate of trial built on 2 October 2026 with
these structs and functions and empty bodies, with `--target web
--remove-name-section --out-name table_io`; the doc comments of the
crate, which appear in the file above each line, are left out here:

```ts
export class Conversion {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    readonly booleans: Uint8Array;
    readonly firstRow: number;
    readonly firstText: string;
    readonly floats: Float64Array;
    readonly integers: BigInt64Array;
    readonly missing: Uint8Array;
    readonly numFailed: number;
    readonly texts: string[];
}

export class TableRead {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    columnBooleans(index: number): Uint8Array;
    columnFloats(index: number): Float64Array;
    columnIntegers(index: number): BigInt64Array;
    columnMissing(index: number): Uint8Array;
    columnName(index: number): string;
    columnNumber(index: number): number;
    columnTexts(index: number): string[];
    columnType(index: number): string;
    readonly column: number;
    readonly decimal: string;
    readonly encoding: string;
    readonly expected: number;
    readonly format: string;
    readonly found: number;
    readonly line: number;
    readonly namesHeader: string;
    readonly namesNumber: number;
    readonly names: string[];
    readonly numColumns: number;
    readonly refusal: string;
    readonly row: number;
    readonly secondColumn: number;
    readonly secondLine: number;
    readonly secondRow: number;
    readonly separator: string;
    readonly sheetColumns: number;
    readonly sheetRows: number;
    readonly sheet: string;
    readonly size: number;
    readonly text: string;
    readonly undecodedLine: number | undefined;
}

export function convertColumn(column_type: string, missing: Uint8Array, integers: BigInt64Array, floats: Float64Array, booleans: Uint8Array, texts: string[], to: string, decimal: string): Conversion;

export function floatText(number: number, decimal: string): string;

export function importTable(bytes: Uint8Array, max_bytes: number, max_cells: number, encoding: string, separator: string, decimal: string): TableRead;

export function isMissing(text: string): boolean;

export function parseBoolean(text: string): boolean | undefined;

export function parseFloat(text: string, decimal: string): number | undefined;

export function parseInteger(text: string): bigint | undefined;

/** Fetches and compiles table_io_bg.wasm, from beside table_io.js when
    it is called with no argument. */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
```

The fields of a class are given in the order of their names, as
wasm-bindgen writes them, and the arguments of a function keep the names
of the Rust, `max_bytes`. `max_bytes` is a `u32`, up to 4,294,967,295
bytes, which is more than the 4,294,967,296 bytes the wasm can hold at
most; `size` and `numFailed` are numbers, exact up to 2^53. JavaScript
cannot make a `TableRead` or a `Conversion` of its own, `private
constructor()`.

These declarations are kept in git, as `js/table_io/test/table_io.d.ts`,
with their doc comments, and the test of the package fails when what the
build writes differs from them. A change of them is a change of the
contract (`docs/architecture.md`, section 8).

## The package

As xlsx_rs's, with its new names (`docs/architecture.md`, sections 9 and
10): `js/table_io`, the package `table_io` of `"type": "module"`, whose
`exports` give `./wasm/table_io.js` and its declarations; its `build`
compiles the binding crate, which takes the library with both features,
`csv` and `xlsx`, for `wasm32-unknown-unknown` in release, with
`--remap-path-prefix` so that no path of the machine is left in the
`.wasm`, and runs wasm-bindgen as above; `npm pack` builds and tests it
before it packs `table_io-0.2.0.tgz`, with `LICENSE` and
`THIRD_PARTY_LICENSES.md`, the notices of the crates compiled into the
`.wasm`, made again for the crates of this build.

### Its size

Not measured: the crate of trial above has empty bodies. The `.wasm` of
xlsx_rs's `js-v0.1.0-dev.1` is 565,045 bytes, 300,646 gzipped with `gzip
-9`, almost all of it calamine and the zip, measured on the owner's Mac
on 29 September 2026; this package adds the import of a text file, the
table and the types, and the functions of a value. The first build
measures the `.wasm` and its JavaScript, raw and gzipped, and the report
of the plan gives them beside xlsx_rs's. The export is not in the
package, and the linker leaves out the code no exported function reaches,
so rust_xlsxwriter, compiled with the feature `xlsx`, should add nothing;
a `.wasm` larger than xlsx_rs's by more than the import of a text file
explains is looked into before the release.

## How it is verified

Under node 26.8.2, with `node --test`, over the package as it is packed,
its `init` given the bytes of the `.wasm`:

- `importTable` over the owner's `excel_es.csv` and `excel_en.xlsx` when
  they are in `tests/data/`, and until then over a CSV and an xlsx written
  by the tests of the library into `tests/data/` as `written.csv` and
  `written.xlsx`: the format, how it was read, the names, and each
  column's type and arrays, an integer column as a `BigInt64Array` with
  a value of 2^60 + 1, `1152921504606846977n`, exact;
- a refusal of each shape of the table above: `raggedRow` with its line,
  counts and separator, `duplicateIndividual` of a text file with its two
  lines and of an xlsx with its two rows, `sheetTooLarge`, `tooLarge`;
- an unreadable xlsx, a zip with no workbook: an `Error` thrown, with the
  library's message;
- an option or a type that is not one of the strings, and an index out of
  range: an `Error` thrown, the worker going on;
- `convertColumn` of a text column `"1"`, `"n.d."` to `"float"`: 1
  failed, row 2, `"n.d."`; of `"1"`, `"2"`: the floats 1 and 2;
- `floatText` against `String(x)` of node for the floats of
  `specs/values.md`'s test of `float_text`, so that table_io's text of a
  float is JavaScript's, checked by JavaScript;
- `parseInteger("9223372036854775807")` the `bigint`
  `9223372036854775807n`;
- every `free()` called once, and a `TableRead` read after its `free()`
  thrown, as wasm-bindgen makes it;
- the declarations the build writes, compared with
  `test/table_io.d.ts`.

In popnei_web, when it takes the package, from its own session: its own
tests of the light worker under Vitest, and the flow of the Individuals
step in Chromium, Firefox and WebKit.

## Open points

None. The owner decided on 2 October 2026 one `.wasm`, the integers as
`BigInt64Array`, and that the package keeps nothing of xlsx_rs's.

## Not in this spec

- The export, which joins the package when popnei_web exports a table
  (`docs/architecture.md`, section 5).
- How popnei_web loads the package on first need, what it shows while it
  downloads, and its words for each refusal: popnei_web's.
- The release, its tag and its URL: `docs/architecture.md`, section 10.
