# xlsx_rs, the wasm package

The WebAssembly build of xlsx_rs, which reads the first visible sheet of
an xlsx into its cells, for popnei_web's light worker. It is what
wasm-bindgen generates from the crate `crates/xlsx_rs_js`, with no code of
its own around it. What each cell and each refusal is, is in
`docs/specs/read.md` of the repository, github.com/JoseBlanca/xlsx_rs.

## What it exports

- `init`, the default export, which loads and compiles `xlsx_rs_bg.wasm`
  and is awaited once, before anything else is called. Called with no
  argument, it fetches the `.wasm` from beside `xlsx_rs.js`, as a browser
  or a bundler resolves it. Under node, whose `fetch` does not read a
  file, give it the bytes: `init({ module_or_path: bytes })`.
- `readXlsx(bytes, maxCells)`, which reads the `Uint8Array` of an xlsx and
  returns an `XlsxRead`. Its `refusal` is `""` for a sheet read, and then
  `sheet` is its name; `firstRow` and `firstColumn`, the first row and
  column of the rectangle of its values, numbered from 1 as Excel numbers
  them; `numRows` and `numColumns`, the size of that rectangle; and
  `cells`, row after row, each `null`, a string, a number or a boolean.
  Otherwise `refusal` is `"notXlsx"`, `"oldExcel"`, `"encrypted"`,
  `"emptySheet"`, `"cellError"` or `"sheetTooLarge"`, with the fields
  that refusal's words need. A sheet whose rectangle would hold more than
  `maxCells` cells is refused as `"sheetTooLarge"`.
- `readXlsx` throws an `Error` for a file it cannot read, a file cut
  short or damaged, or one past a bound xlsx_rs checks before calamine,
  with the message of the zip crate, of calamine or of xlsx_rs, whichever
  failed.

An `XlsxRead` lives in the memory of the wasm, up to `maxCells` cells of
it. JavaScript's garbage collector frees it too, through a
`FinalizationRegistry` that wasm-bindgen registers, but at a time nobody
chooses, which may be after the next file is read: so call `free()` in a
`finally`. Read each field once, `cells` above all, which makes a new
array at each read.

```js
import init, { readXlsx } from "xlsx_rs";

await init();
const read = readXlsx(bytes, 2_000_000);
try {
    if (read.refusal === "") {
        const cells = read.cells;
        // ...
    }
} finally {
    read.free();
}
```

The declarations, `wasm/xlsx_rs.d.ts`, are the contract with popnei_web.
They are kept in git as `test/xlsx_rs.d.ts`, and `npm test` fails when
the ones generated differ from them.

## How it is installed

popnei_web installs it by the URL of the `.tgz` attached to a release of
xlsx_rs on GitHub, a pre-release on a tag `js-v0.1.0-dev.1`, then `dev.2`
and on, each tag used once:

```
npm install https://github.com/JoseBlanca/xlsx_rs/releases/download/js-v0.1.0-dev.1/xlsx_rs-0.1.0.tgz
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
npm pack        # builds, tests, copies LICENSE and THIRD_PARTY_LICENSES.md from the root, then xlsx_rs-0.1.0.tgz
```

`THIRD_PARTY_LICENSES.md` holds the license of each crate compiled into
the `.wasm`, which their licenses ask to go with every copy; it is written
again when a dependency changes, from the list the command at its top
gives.

The build writes the paths of the home folder into the `.wasm` as `~`
(`--remap-path-prefix`, in the script's `RUSTFLAGS`), so that the paths of
the sources of the dependencies, which the `.wasm` keeps for its messages,
do not carry the name of the account that built it.
