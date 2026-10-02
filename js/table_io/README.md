# table_io, the wasm package

The WebAssembly build of table_io, which imports a table from the bytes
of a CSV, a TSV or an xlsx as typed columns, for popnei_web's light
worker. It is what wasm-bindgen generates from the crate
`crates/table_io_js`, with no code of its own around it. What an import
gives and refuses is in `docs/specs/import.md` of the repository,
github.com/JoseBlanca/table_io, the rules of a value and of a conversion in
`docs/specs/values.md`, and the package in `docs/specs/package.md`.

## What it exports

- `init`, the default export, which loads and compiles `table_io_bg.wasm`
  and is awaited once, before anything else is called. Called with no
  argument, it fetches the `.wasm` from beside `table_io.js`, as a browser
  or a bundler resolves it. Under node, whose `fetch` does not read a
  file, give it the bytes: `init({ module_or_path: bytes })`.
- `importTable(bytes, max_bytes, max_cells, encoding, separator, decimal)`,
  which reads the bytes of a file, its format found from its first
  bytes, and returns a `TableRead`. The bytes must be a `Uint8Array`, as
  `new Uint8Array(await file.arrayBuffer())` gives them: an `ArrayBuffer`
  or a `DataView` given in its place is read as no bytes, an empty file,
  with no `Error`, which TypeScript refuses from the declarations but
  JavaScript does not. `max_bytes` is the largest file
  accepted, in bytes, a whole number from 0 to 2^53; `max_cells` the
  largest rectangle of the values of an xlsx, in cells, a whole number
  from 0 to 4,294,967,295. `encoding` is `""` to find it, `"utf-8"` or
  `"windows-1252"`; `separator` `""`, `"tab"`, `"semicolon"` or
  `"comma"`; `decimal` `""`, `"point"` or `"comma"`; the three are for a
  text file and ignored for an xlsx.
- A `TableRead` holds a table or a refusal. Its `refusal` is `""` for a
  table, and then `format` is `"text"` or `"xlsx"`; `encoding`,
  `separator`, `decimal` and `undecodedLine` how a text file was read, and
  `decimal` `"point"` for an xlsx; `sheet` the sheet of an xlsx;
  `namesHeader`, `namesNumber` and `names` the first column of the file,
  which names the individuals; and `numColumns` the number of the other
  columns. Each of them is read by its index, from 0, with `columnName`,
  `columnNumber`, `columnType`, `"integer"`, `"float"`, `"boolean"` or
  `"text"`, and five arrays: `columnMissing`, 1 for a missing value, and
  `columnIntegers`, a `BigInt64Array`, `columnFloats`, `columnBooleans`
  and `columnTexts`, of which the one of the column's type holds its
  values and the other three are empty.
- Otherwise `refusal` is the kind of the refusal, `"tooLarge"`,
  `"oldExcel"`, `"raggedRow"` and the others of `docs/specs/package.md`,
  with `format` and the fields its words need, the others 0 or `""`. A
  file that cannot be read, cut short or damaged, is the kind
  `"unreadable"`, with the message of the zip crate, of calamine or of
  table_io in `text`.
- `convertColumn(column_type, missing, integers, floats, booleans, texts,
  to, decimal)`, which converts a column, given as a `TableRead` gives
  it, to the type `to`, and returns a `Conversion`: with `numFailed` 0 the
  column converted, as its five arrays; otherwise how many values do not
  convert, and the row, from 1, and the text of the first.
- `isMissing(text)`, `parseInteger(text)`, a `bigint`, `parseFloat(text,
  decimal)`, `parseBoolean(text)` and `floatText(number, decimal)`, the
  rules of a value the import reads by; the three parses give `undefined`
  for a text that holds no such value.

Every function throws an `Error` only for a defect of the caller: a limit
that is not a whole number in its range, an option or a type that is not
one of its strings, an index of a column that is not a whole number from
0 to `numColumns` − 1, or a column given
to `convertColumn` whose arrays do not match. A file refused or that
cannot be read is never thrown.

A `TableRead` and a `Conversion` live in the memory of the wasm.
JavaScript's garbage collector frees them too, through a
`FinalizationRegistry` that wasm-bindgen registers, but at a time nobody
chooses, which may be after the next file is read: so call `free()` once,
in a `finally`; a second `free()`, or a field read after it, throws. Each
read of a field or of a column copies it out whole, a new array each
time, so read each once into a variable.

```js
import init, { importTable } from "table_io";

await init();
const read = importTable(bytes, 50_000_000, 2_000_000, "", "", "");
try {
    if (read.refusal === "") {
        const names = read.names;
        for (let index = 0; index < read.numColumns; index++) {
            const columnType = read.columnType(index);
            const missing = read.columnMissing(index);
            // ...
        }
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
