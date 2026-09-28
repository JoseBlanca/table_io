// The package as it is released, under node: the wasm loaded from its
// bytes, a sheet read, a refusal, a file calamine cannot read, and the
// declarations of the contract with popnei_web ("The package, built" of
// docs/specs/read.md).
//
// The owner's excel_en.xlsx, excel_1904.xlsx and encrypted.xlsx, in
// tests/data/, are read with the cells the owner typed. The files of
// tests/data/ that crates/xlsx_rs/tests/write_fixtures.rs writes,
// written.xlsx, empty_first_sheet.xlsx, table_at_c2.xlsx, getting_data.xlsx,
// wide_table_at_c2.xlsx and unique_count.xlsx, and a CSV, are read in any
// case.

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
    // B3, in the population merged over B2 and B3, is "Andalucía", and
    // the dates of the column Fecha are text in ISO 8601.
    assert.deepEqual(fields.cells, [
        "Individuo", "Población", "Altura", "Fecha", "Afectado",
        "ind1", "Andalucía", 1.75, "2024-05-13", true,
        "ind2", "Andalucía", 1.62, "2024-05-14", false,
        "ind3", "Murcia", null, "2024-05-15", true,
        "ind4", "Murcia", 1.55, "2024-05-16", false,
    ]);
    assert.deepEqual(fields.cells.map(typeOfCell), [
        "string", "string", "string", "string", "string",
        "string", "string", "number", "string", "boolean",
        "string", "string", "number", "string", "boolean",
        "string", "string", "null", "string", "boolean",
        "string", "string", "number", "string", "boolean",
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

test("a file that is not a zip past its first bytes throws an Error with the message of the zip", async () => {
    const bytes = await readFile(new URL("written.xlsx", dataDir));

    assert.throws(() => readXlsx(bytes.subarray(0, 500), MAX_SHEET_CELLS), {
        name: "Error",
        message: "invalid Zip archive: Could not find EOCD",
    });
});

/**
 * The cells under `header` in the first row of the sheet of `fields`, from
 * the second row to the last; empty when no cell of the first row is
 * `header`.
 */
function columnUnder(fields, header) {
    const column = fields.cells.slice(0, fields.numColumns).indexOf(header);
    if (column === -1) {
        return [];
    }
    const cells = [];
    for (let row = 1; row < fields.numRows; row++) {
        cells.push(fields.cells[row * fields.numColumns + column]);
    }
    return cells;
}

/** Whether `cellText` is a string of the form `pattern`, where each `d` of
 * the pattern is a digit from 0 to 9 and every other character is itself. */
function hasForm(cellText, pattern) {
    return (
        typeof cellText === "string" &&
        cellText.length === pattern.length &&
        [...pattern].every((patternCharacter, position) =>
            patternCharacter === "d"
                ? /[0-9]/.test(cellText[position])
                : cellText[position] === patternCharacter,
        )
    );
}

// What "Made by the owner" of docs/specs/read.md says excel_en.xlsx holds,
// as crates/xlsx_rs/tests/owner_files.rs asserts it, with the cells the
// owner typed. The file is a copy of excel_es.xlsx, since an xlsx stores
// formulas and errors in English whatever the language of Excel.
test("excel_en.xlsx gives the cells of English Excel", async () => {
    const bytes = await readFile(new URL("excel_en.xlsx", dataDir));

    const fields = fieldsOfRead(bytes);

    assert.equal(fields.refusal, "");
    assert.equal(fields.sheet, "Hoja1");
    assert.deepEqual(
        [fields.firstRow, fields.firstColumn, fields.numRows, fields.numColumns],
        [1, 1, 7, 7],
    );
    const headerRow = fields.cells.slice(0, fields.numColumns);
    for (const name of ["Individuo", "Población", "Altura", "Fecha", "Hora", "Afectado", "Código"]) {
        assert.ok(headerRow.includes(name), `no ${name} in the header ${JSON.stringify(headerRow)}`);
    }
    const expectedInColumns = [
        ["Altura", 1.75],
        ["Fecha", "2024-05-13"],
        ["Hora", "14:30:00"],
        ["Afectado", true],
        ["Afectado", false],
        ["Código", 7],
    ];
    for (const [header, cell] of expectedInColumns) {
        const column = columnUnder(fields, header);
        assert.ok(
            column.includes(cell),
            `no ${JSON.stringify(cell)} under ${header}, which holds ${JSON.stringify(column)}`,
        );
    }
    for (const cell of ["001", "#N/A", "#DIV/0!"]) {
        assert.ok(fields.cells.includes(cell), `no cell is ${cell}`);
    }
    const populations = columnUnder(fields, "Población");
    assert.ok(
        populations.some(
            (population, row) => population !== null && population === populations[row + 1],
        ),
        `no population in two rows running, as a merged one is, in ${JSON.stringify(populations)}`,
    );
    const rows = [];
    for (let row = 0; row < fields.numRows; row++) {
        rows.push(fields.cells.slice(row * fields.numColumns, (row + 1) * fields.numColumns));
    }
    assert.ok(rows.some((row) => row.every((cell) => cell === null)), "no blank row");
    const kinds = [
        ["Altura", "a number", (cell) => typeof cell === "number"],
        ["Fecha", "a text dddd-dd-dd", (cell) => hasForm(cell, "dddd-dd-dd")],
        ["Hora", "a text dd:dd:dd", (cell) => hasForm(cell, "dd:dd:dd")],
        ["Afectado", "a boolean", (cell) => typeof cell === "boolean"],
    ];
    for (const [header, kind, isOfKind] of kinds) {
        const otherCells = columnUnder(fields, header).filter((cell) => cell !== null && !isOfKind(cell));
        assert.deepEqual(otherCells, [], `under ${header}, cells that are not ${kind}`);
    }
    const individuals = columnUnder(fields, "Individuo");
    const rowsWithNoPopulation = individuals
        .map((individual, row) => [individual, populations[row], row + 1])
        .filter(([individual, population]) => individual !== null && population === null)
        .map(([, , row]) => row);
    assert.deepEqual(
        rowsWithNoPopulation,
        [],
        "rows, from 0 for the header, with an Individuo and no Población, as a merged range lost leaves them",
    );

    // The cells the owner typed, from A1, row after row: B3:B4 merged,
    // G2:G4 formatted 000, row 5 blank, G6 =NA() and G7 =1/0.
    assert.deepEqual(fields.cells, [
        "Individuo", "Población", "Altura", "Fecha", "Hora", "Afectado", "Código",
        "ind1", "Andalucía", 1.75, "2024-05-13", "14:30:00", true, 7,
        "ind2", "Castilla y León", 1.62, "2024-05-14", "09:05:00", false, 12,
        "001", "Castilla y León", 1.8, "2024-05-15", "18:45:00", true, 3,
        null, null, null, null, null, null, null,
        "ind4", "Murcia", 1.55, "2024-05-16", "07:00:00", false, "#N/A",
        "ind5", "Murcia", 1.7, "2024-05-17", "12:15:00", true, "#DIV/0!",
    ]);
});

// The owner's file, saved by Excel for Mac in the date system of 1904,
// with an element workbookPr of another namespace after the one of the
// workbook, which calamine alone read as the system of 1900, 2020-05-12.
test("excel_1904.xlsx gives the date and the time as Excel shows them", async () => {
    const bytes = await readFile(new URL("excel_1904.xlsx", dataDir));

    const fields = fieldsOfRead(bytes);

    assert.deepEqual(fields, {
        refusal: "",
        detail: "",
        sheet: "Sheet1",
        firstRow: 1,
        firstColumn: 1,
        numRows: 2,
        numColumns: 2,
        cells: ["Fecha", "Hora", "2024-05-13", "14:30:00"],
    });
});

// A table of texts of two texts that says it holds 400,000,000, for which
// calamine reserved room before it read the first, and the package
// trapped, a failure that ends the worker, before xlsx_rs checked it.
test("unique_count.xlsx, whose table of texts says it holds 400,000,000 texts, throws an Error", async () => {
    const bytes = await readFile(new URL("unique_count.xlsx", dataDir));

    assert.throws(() => readXlsx(bytes, MAX_SHEET_CELLS), {
        name: "Error",
        message: "too many texts",
    });
    // The package still reads after it, as it does not after a trap.
    const written = await readFile(new URL("written.xlsx", dataDir));
    assert.equal(fieldsOfRead(written).sheet, "Individuos");
});

test("encrypted.xlsx is refused as encrypted", async () => {
    const bytes = await readFile(new URL("encrypted.xlsx", dataDir));

    const fields = fieldsOfRead(bytes);

    assert.equal(fields.refusal, "encrypted");
    assert.equal(fields.sheet, "");
    assert.deepEqual(fields.cells, []);
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
