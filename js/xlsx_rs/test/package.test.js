// The package as it is released, under node: the wasm loaded from its
// bytes, a sheet read, a refusal, a file calamine cannot read, and the
// declarations of the contract with popnei_web ("The package, built" of
// docs/specs/read.md).
//
// Until the owner's excel_en.xlsx and encrypted.xlsx exist, the files read
// are those of tests/data/ that crates/xlsx_rs/tests/write_fixtures.rs
// writes, written.xlsx, empty_first_sheet.xlsx, table_at_c2.xlsx,
// getting_data.xlsx and wide_table_at_c2.xlsx, and the refusal is that of a
// CSV.

import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { before, test } from "node:test";

import init, { readXlsx } from "../wasm/xlsx_rs.js";

/** MAX_SHEET_CELLS of popnei_web, the limit its light worker gives. */
const MAX_SHEET_CELLS = 2_000_000;

const packageDir = new URL("../", import.meta.url);
const dataDir = new URL("../../../tests/data/", import.meta.url);

before(async () => {
    // node's fetch does not read the .wasm beside xlsx_rs.js, as a browser
    // does, so init is given its bytes, in the object wasm-bindgen asks for.
    const wasmBytes = await readFile(new URL("wasm/xlsx_rs_bg.wasm", packageDir));
    await init({ module_or_path: wasmBytes });
});

/** The JavaScript type of a cell, "null" for an empty one. */
function typeOfCell(cell) {
    return cell === null ? "null" : typeof cell;
}

/**
 * Reads `bytes` with the limit `maxCells`, MAX_SHEET_CELLS unless given,
 * gives back the fields of what readXlsx returns, and frees it.
 */
function fieldsOfRead(bytes, maxCells = MAX_SHEET_CELLS) {
    const read = readXlsx(bytes, maxCells);
    try {
        return {
            refusal: read.refusal,
            detail: read.detail,
            sheet: read.sheet,
            firstRow: read.firstRow,
            firstColumn: read.firstColumn,
            numRows: read.numRows,
            numColumns: read.numColumns,
            cells: read.cells,
        };
    } finally {
        read.free();
    }
}

test("a CSV named .xlsx is refused as notXlsx", () => {
    const fields = fieldsOfRead(new TextEncoder().encode("id,pop\n"));

    assert.equal(fields.refusal, "notXlsx");
    assert.equal(fields.detail, "");
    assert.equal(fields.sheet, "");
    assert.deepEqual(fields.cells, []);
});

test("a compound file of the old Office is refused as oldExcel", () => {
    // The mark every compound file starts with, followed by zeros: no
    // encrypted xlsx inside it.
    const bytes = new Uint8Array(512);
    bytes.set([0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]);

    const fields = fieldsOfRead(bytes);

    assert.equal(fields.refusal, "oldExcel");
    assert.equal(fields.sheet, "");
    assert.deepEqual(fields.cells, []);
});

test("written.xlsx is read with every cell and its type", async () => {
    const bytes = await readFile(new URL("written.xlsx", dataDir));

    const fields = fieldsOfRead(bytes);

    assert.equal(fields.refusal, "");
    assert.equal(fields.detail, "");
    assert.equal(fields.sheet, "Individuos");
    assert.deepEqual(
        [fields.firstRow, fields.firstColumn, fields.numRows, fields.numColumns],
        [1, 1, 5, 5],
    );
    // B3, in the population merged over B2 and B3, is "Andalucía". The
    // dates are not final: task 3.2 of docs/plans/read.md makes those of
    // the column Fecha the texts "2024-05-13" to "2024-05-16". This test is
    // to fail then, and be changed with it.
    assert.deepEqual(fields.cells, [
        "Individuo", "Población", "Altura", "Fecha", "Afectado",
        "ind1", "Andalucía", 1.75, 45425, true,
        "ind2", "Andalucía", 1.62, 45426, false,
        "ind3", "Murcia", null, 45427, true,
        "ind4", "Murcia", 1.55, 45428, false,
    ]);
    assert.deepEqual(fields.cells.map(typeOfCell), [
        "string", "string", "string", "string", "string",
        "string", "string", "number", "number", "boolean",
        "string", "string", "number", "number", "boolean",
        "string", "string", "null", "number", "boolean",
        "string", "string", "number", "number", "boolean",
    ]);
});

test("a first sheet with no value is refused as emptySheet with its name", async () => {
    const bytes = await readFile(new URL("empty_first_sheet.xlsx", dataDir));

    const fields = fieldsOfRead(bytes);

    assert.deepEqual(fields, {
        refusal: "emptySheet",
        detail: "",
        sheet: "Notas",
        firstRow: 0,
        firstColumn: 0,
        numRows: 0,
        numColumns: 0,
        cells: [],
    });
});

test("a table at C2 starts at row 2 and column 3", async () => {
    const bytes = await readFile(new URL("table_at_c2.xlsx", dataDir));

    const fields = fieldsOfRead(bytes);

    assert.deepEqual(fields, {
        refusal: "",
        detail: "",
        sheet: "Individuos",
        firstRow: 2,
        firstColumn: 3,
        numRows: 3,
        numColumns: 3,
        cells: [
            "Individuo", "Altura", "Afectado",
            "ind1", 1.75, true,
            "ind2", 1.62, false,
        ],
    });
});

test("a cell saved with an error calamine does not know is refused as cellError with its text", async () => {
    const bytes = await readFile(new URL("getting_data.xlsx", dataDir));

    const fields = fieldsOfRead(bytes);

    assert.deepEqual(fields, {
        refusal: "cellError",
        detail: "#GETTING_DATA",
        sheet: "",
        firstRow: 0,
        firstColumn: 0,
        numRows: 0,
        numColumns: 0,
        cells: [],
    });
});

test("a table at C2 of 5 columns read with a limit of 16 cells is refused as sheetTooLarge at row 5", async () => {
    // Rows 2 to 4 are 15 cells; C5 makes the rectangle 4 rows of 5
    // columns, 20 cells, past the limit.
    const bytes = await readFile(new URL("wide_table_at_c2.xlsx", dataDir));

    const fields = fieldsOfRead(bytes, 16);

    assert.deepEqual(fields, {
        refusal: "sheetTooLarge",
        detail: "",
        sheet: "Individuos",
        firstRow: 2,
        firstColumn: 3,
        numRows: 4,
        numColumns: 5,
        cells: [],
    });
});

test("a file calamine cannot read throws an Error with its message", async () => {
    const bytes = await readFile(new URL("written.xlsx", dataDir));

    assert.throws(() => readXlsx(bytes.subarray(0, 500), MAX_SHEET_CELLS), {
        name: "Error",
        message: "Zip error: invalid Zip archive: Could not find EOCD",
    });
});

/**
 * `declarations` without the block `export interface InitOutput { ... }`,
 * the functions wasm-bindgen exports for its own JavaScript, which change
 * with its version and which popnei_web does not read. Throws when the
 * block is not there, so that the comparison cannot pass on a file whose
 * block was not found.
 */
function withoutInitOutput(declarations) {
    const initOutput = /^export interface InitOutput \{\n(?:.*\n)*?\}\n/m;
    assert.match(declarations, initOutput);
    return declarations.replace(initOutput, "");
}

test("the declarations generated are the ones kept in git", async () => {
    // A difference is a change of the contract with popnei_web: the spec
    // first, then test/xlsx_rs.d.ts, then a new release. The kept file is
    // the whole file wasm-bindgen writes, InitOutput among it, which the
    // comparison leaves out of both.
    const generated = await readFile(new URL("wasm/xlsx_rs.d.ts", packageDir), "utf8");
    const kept = await readFile(new URL("test/xlsx_rs.d.ts", packageDir), "utf8");

    assert.equal(withoutInitOutput(generated), withoutInitOutput(kept));
});
