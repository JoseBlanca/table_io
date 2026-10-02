// The package as it is released, under node: the wasm loaded from its
// bytes, the tables and the refusals importTable gives, the Errors it
// throws for a defect of the caller, convertColumn and the rules of a
// value, free(), and the declarations of the contract with popnei_web
// ("How it is verified" of docs/specs/package.md).
//
// The owner's excel_en.xlsx and encrypted.xlsx, in tests/data/, and the
// files crates/table_io/tests/write_fixtures.rs writes there, written.xlsx,
// written.csv, empty_first_sheet.xlsx, getting_data.xlsx,
// wide_table_at_c2.xlsx and unique_count.xlsx, are read from it; the other
// cases are an xlsx written by test/xlsx.js or a text file given as its
// characters, which TextEncoder turns into the bytes of UTF-8.

import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { before, test } from "node:test";

import init, {
    convertColumn,
    floatText,
    importTable,
    isMissing,
    parseBoolean,
    parseFloat,
    parseInteger,
} from "../wasm/table_io.js";
import { workbookParts, sheetXml, xlsxOf, zipOf } from "./xlsx.js";

/** MAX_INDIVIDUALS_FILE_BYTES of popnei_web, 20 MB, the limit of bytes its light worker gives. */
const MAX_BYTES = 20_000_000;
/** MAX_SHEET_CELLS of popnei_web, the limit of cells its light worker gives. */
const MAX_CELLS = 2_000_000;

const packageDir = new URL("../", import.meta.url);
const dataDir = new URL("../../../tests/data/", import.meta.url);

before(async () => {
    // node's fetch does not read the .wasm beside table_io.js, as a browser
    // does, so init is given its bytes, in the object wasm-bindgen asks for.
    const wasmBytes = await readFile(new URL("wasm/table_io_bg.wasm", packageDir));
    await init({ module_or_path: wasmBytes });
});

/** The bytes of the file `name` of tests/data/. */
function dataFile(name) {
    return readFile(new URL(name, dataDir));
}

/**
 * Imports `bytes` with the limits and the options given, MAX_BYTES,
 * MAX_CELLS and every option to be found unless given, gives back every
 * field of what importTable returns and each column's, and frees it.
 */
function fieldsOfImport(
    bytes,
    { maxBytes = MAX_BYTES, maxCells = MAX_CELLS, encoding = "", separator = "", decimal = "" } = {},
) {
    const read = importTable(bytes, maxBytes, maxCells, encoding, separator, decimal);
    try {
        const columns = [];
        for (let index = 0; index < read.numColumns; index++) {
            columns.push({
                name: read.columnName(index),
                number: read.columnNumber(index),
                type: read.columnType(index),
                missing: read.columnMissing(index),
                integers: read.columnIntegers(index),
                floats: read.columnFloats(index),
                booleans: read.columnBooleans(index),
                texts: read.columnTexts(index),
            });
        }
        return {
            refusal: read.refusal,
            format: read.format,
            text: read.text,
            size: read.size,
            line: read.line,
            secondLine: read.secondLine,
            row: read.row,
            secondRow: read.secondRow,
            column: read.column,
            secondColumn: read.secondColumn,
            expected: read.expected,
            found: read.found,
            sheetRows: read.sheetRows,
            sheetColumns: read.sheetColumns,
            encoding: read.encoding,
            separator: read.separator,
            decimal: read.decimal,
            undecodedLine: read.undecodedLine,
            sheet: read.sheet,
            namesHeader: read.namesHeader,
            namesNumber: read.namesNumber,
            names: read.names,
            numColumns: read.numColumns,
            columns,
        };
    } finally {
        read.free();
    }
}

/** The fields of a TableRead with no table and no field of a refusal filled. */
const NO_TABLE = {
    refusal: "",
    format: "",
    text: "",
    size: 0,
    line: 0,
    secondLine: 0,
    row: 0,
    secondRow: 0,
    column: 0,
    secondColumn: 0,
    expected: 0,
    found: 0,
    sheetRows: 0,
    sheetColumns: 0,
    encoding: "",
    separator: "",
    decimal: "",
    undecodedLine: undefined,
    sheet: "",
    namesHeader: "",
    namesNumber: 0,
    names: [],
    numColumns: 0,
    columns: [],
};

/** The fields of a table of an xlsx, before its sheet, names and columns. */
const XLSX_TABLE = {
    ...NO_TABLE,
    format: "xlsx",
    decimal: "point",
};

/** The bytes of `text` in UTF-8, a text file as the test writes it. */
function utf8(text) {
    return new TextEncoder().encode(text);
}

/**
 * A column as `fieldsOfImport` gives it: `name`, `number`, `type`, its
 * values `values`, null for a missing one, in the array of its type, and
 * the other three arrays empty.
 */
function column(name, number, type, values) {
    const missing = Uint8Array.from(values, (columnValue) => (columnValue === null ? 1 : 0));
    const ofType = (wanted, makeArray, zero, entryOf) =>
        type === wanted
            ? makeArray(values.map((columnValue) => (columnValue === null ? zero : entryOf(columnValue))))
            : makeArray([]);
    return {
        name,
        number,
        type,
        missing,
        integers: ofType("integer", (entries) => BigInt64Array.from(entries), 0n, (entry) => entry),
        floats: ofType("float", (entries) => Float64Array.from(entries), 0, (entry) => entry),
        booleans: ofType("boolean", (entries) => Uint8Array.from(entries), 0, (entry) => (entry ? 1 : 0)),
        texts: ofType("text", (entries) => entries, "", (entry) => entry),
    };
}

// The table of crates/table_io/tests/write_fixtures.rs: B2:B3 merged, so
// that ind2's population is Andalucía too, C4 empty, and the dates of the
// column Fecha text in ISO 8601.
test("written.xlsx is read as its table, each column typed", async () => {
    const fields = fieldsOfImport(await dataFile("written.xlsx"));

    assert.deepEqual(fields, {
        ...XLSX_TABLE,
        sheet: "Individuos",
        namesHeader: "Individuo",
        namesNumber: 1,
        names: ["ind1", "ind2", "ind3", "ind4"],
        numColumns: 4,
        columns: [
            column("Población", 2, "text", ["Andalucía", "Andalucía", "Murcia", "Murcia"]),
            column("Altura", 3, "float", [1.75, 1.62, null, 1.55]),
            column("Fecha", 4, "text", ["2024-05-13", "2024-05-14", "2024-05-15", "2024-05-16"]),
            column("Afectado", 5, "boolean", [true, false, true, false]),
        ],
    });
});

// What "Made by the owner" of docs/specs/read.md says excel_en.xlsx holds:
// B3:B4 merged, G2:G4 formatted 000, row 5 blank, G6 =NA() and G7 =1/0.
test("excel_en.xlsx is read as the table the owner typed", async () => {
    const fields = fieldsOfImport(await dataFile("excel_en.xlsx"));

    assert.deepEqual(fields, {
        ...XLSX_TABLE,
        sheet: "Hoja1",
        namesHeader: "Individuo",
        namesNumber: 1,
        names: ["ind1", "ind2", "001", "ind4", "ind5"],
        numColumns: 6,
        columns: [
            column("Población", 2, "text", ["Andalucía", "Castilla y León", "Castilla y León", "Murcia", "Murcia"]),
            column("Altura", 3, "float", [1.75, 1.62, 1.8, 1.55, 1.7]),
            column("Fecha", 4, "text", ["2024-05-13", "2024-05-14", "2024-05-15", "2024-05-16", "2024-05-17"]),
            column("Hora", 5, "text", ["14:30:00", "09:05:00", "18:45:00", "07:00:00", "12:15:00"]),
            column("Afectado", 6, "boolean", [true, false, true, false, true]),
            column("Código", 7, "integer", [7n, 12n, 3n, null, null]),
        ],
    });
});

test("an integer of an xlsx past 2^53, 2^60 + 1 written as text, is exact in its BigInt64Array", () => {
    const bytes = xlsxOf([["id", "n"], ["A", "1152921504606846977"], ["B", -3]]);

    const fields = fieldsOfImport(bytes);

    assert.equal(fields.refusal, "");
    assert.deepEqual(fields.columns, [column("n", 2, "integer", [1152921504606846977n, -3n])]);
});

test("a table whose header starts at C3 gives the columns of the sheet, C for the names", () => {
    const bytes = xlsxOf([
        [],
        [],
        [null, null, "id", "h", "pop"],
        [null, null, "A", 1.5, "P1"],
        [null, null, "B", 2.5, "P2"],
    ]);

    const fields = fieldsOfImport(bytes);

    assert.deepEqual(fields, {
        ...XLSX_TABLE,
        sheet: "Sheet1",
        namesHeader: "id",
        namesNumber: 3,
        names: ["A", "B"],
        numColumns: 2,
        columns: [column("h", 4, "float", [1.5, 2.5]), column("pop", 5, "text", ["P1", "P2"])],
    });
});

// The table of write_written_csv of crates/table_io/tests/write_fixtures.rs,
// a CSV as a Spanish Excel writes it: Windows-1252, ";", the decimal comma
// and CRLF, the population of ind2 quoted with the separator in it, the
// height of ind3 empty and its Número NA, and 2^60 + 1 exact.
test("written.csv is read as Windows-1252 with semicolons and the comma, each column typed", async () => {
    const fields = fieldsOfImport(await dataFile("written.csv"));

    assert.deepEqual(fields, {
        ...NO_TABLE,
        format: "text",
        encoding: "windows-1252",
        separator: "semicolon",
        decimal: "comma",
        namesHeader: "Individuo",
        namesNumber: 1,
        names: ["ind1", "ind2", "ind3", "ind4"],
        numColumns: 4,
        columns: [
            column("Población", 2, "text", ["Andalucía", "Castilla; León", "Murcia", "Murcia"]),
            column("Altura", 3, "float", [1.75, 1.62, null, 1.55]),
            column("Número", 4, "integer", [1152921504606846977n, -3n, null, 12n]),
            column("Afectado", 5, "boolean", [true, false, true, false]),
        ],
    });
});

test("a ragged row of a text file is refused with its line, its cells, the header's and the separator", () => {
    const bytes = utf8("id;pop;;x\nA;P1;;1\nB;P2\n");

    assert.deepEqual(fieldsOfImport(bytes), {
        ...NO_TABLE,
        refusal: "raggedRow",
        format: "text",
        line: 3,
        expected: 4,
        found: 2,
        separator: "semicolon",
    });
});

// The owner's excel_es.csv, saved by Excel in Spanish on Windows as "CSV
// (delimitado por comas)" ("How it is verified" of docs/specs/package.md).
test("excel_es.csv is read as Windows-1252 with semicolons and the comma", { skip: "waits for tests/data/excel_es.csv, made by the owner" }, async () => {
    const fields = fieldsOfImport(await dataFile("excel_es.csv"));

    assert.deepEqual(
        [fields.refusal, fields.format, fields.encoding, fields.separator, fields.decimal, fields.undecodedLine],
        ["", "text", "windows-1252", "semicolon", "comma", undefined],
    );
});

// Each byte of Windows-1252 that is not ASCII, ó, í, ú, is not UTF-8, and
// so is the replacement character, the first on line 1.
test("written.csv read with the encoding set to utf-8 shows the replacement character from line 1", async () => {
    const fields = fieldsOfImport(await dataFile("written.csv"), { encoding: "utf-8" });

    assert.deepEqual(fields, {
        ...NO_TABLE,
        format: "text",
        encoding: "utf-8",
        separator: "semicolon",
        decimal: "comma",
        undecodedLine: 1,
        namesHeader: "Individuo",
        namesNumber: 1,
        names: ["ind1", "ind2", "ind3", "ind4"],
        numColumns: 4,
        columns: [
            column("Poblaci\uFFFDn", 2, "text", ["Andaluc\uFFFDa", "Castilla; Le\uFFFDn", "Murcia", "Murcia"]),
            column("Altura", 3, "float", [1.75, 1.62, null, 1.55]),
            column("N\uFFFDmero", 4, "integer", [1152921504606846977n, -3n, null, 12n]),
            column("Afectado", 5, "boolean", [true, false, true, false]),
        ],
    });
});

// Split at the comma, the header is one cell and the row of ind1, whose
// height is 1,75, two.
test("written.csv read with the separator set to comma is refused at its first row with a comma", async () => {
    const fields = fieldsOfImport(await dataFile("written.csv"), { separator: "comma" });

    assert.deepEqual(fields, {
        ...NO_TABLE,
        refusal: "raggedRow",
        format: "text",
        line: 2,
        expected: 1,
        found: 2,
        separator: "comma",
    });
});

test("written.csv read with the decimal mark set to point has its heights as text", async () => {
    const fields = fieldsOfImport(await dataFile("written.csv"), { decimal: "point" });

    assert.deepEqual(fields, {
        ...NO_TABLE,
        format: "text",
        encoding: "windows-1252",
        separator: "semicolon",
        decimal: "point",
        namesHeader: "Individuo",
        namesNumber: 1,
        names: ["ind1", "ind2", "ind3", "ind4"],
        numColumns: 4,
        columns: [
            column("Población", 2, "text", ["Andalucía", "Castilla; León", "Murcia", "Murcia"]),
            column("Altura", 3, "text", ["1,75", "1,62", null, "1,55"]),
            column("Número", 4, "integer", [1152921504606846977n, -3n, null, 12n]),
            column("Afectado", 5, "boolean", [true, false, true, false]),
        ],
    });
});

// The text of UTF-8 `Población` read as Windows-1252, each of the two bytes
// of ó a character.
test("a file of UTF-8 read with the encoding set to windows-1252 shows each byte as a character", () => {
    const fields = fieldsOfImport(utf8("id;Población\nA;x\n"), { encoding: "windows-1252" });

    assert.deepEqual(fields, {
        ...NO_TABLE,
        format: "text",
        encoding: "windows-1252",
        separator: "semicolon",
        decimal: "point",
        namesHeader: "id",
        namesNumber: 1,
        names: ["A"],
        numColumns: 1,
        columns: [column("PoblaciÃ³n", 2, "text", ["x"])],
    });
});

test("a file of commas read with the separator set to semicolon is one column", () => {
    const fields = fieldsOfImport(utf8("id,pop\nA,P1\n"), { separator: "semicolon" });

    assert.deepEqual(fields, {
        ...NO_TABLE,
        format: "text",
        encoding: "utf-8",
        separator: "semicolon",
        decimal: "point",
        namesHeader: "id,pop",
        namesNumber: 1,
        names: ["A,P1"],
        numColumns: 0,
        columns: [],
    });
});

test("a quoted number read with the comma as separator and the decimal mark set to comma is a float", () => {
    const fields = fieldsOfImport(utf8('id,h\nA,"1,5"\n'), { decimal: "comma" });

    assert.deepEqual(fields, {
        ...NO_TABLE,
        format: "text",
        encoding: "utf-8",
        separator: "comma",
        decimal: "comma",
        namesHeader: "id",
        namesNumber: 1,
        names: ["A"],
        numColumns: 1,
        columns: [column("h", 2, "float", [1.5])],
    });
});

// The mark FF FE and two bytes for each character, little endian, as
// Excel writes "Unicode Text".
test("a file of UTF-16 little endian with its mark is read as utf-16 with the tab", () => {
    const text = "id\tpop\nA\tP1\n";
    const bytes = new Uint8Array(2 + 2 * text.length);
    bytes.set([0xff, 0xfe]);
    for (let index = 0; index < text.length; index++) {
        const unit = text.charCodeAt(index);
        bytes.set([unit & 0xff, unit >> 8], 2 + 2 * index);
    }

    assert.deepEqual(fieldsOfImport(bytes), {
        ...NO_TABLE,
        format: "text",
        encoding: "utf-16",
        separator: "tab",
        decimal: "point",
        namesHeader: "id",
        namesNumber: 1,
        names: ["A"],
        numColumns: 1,
        columns: [column("pop", 2, "text", ["P1"])],
    });
});

test("a quote never closed is refused as unclosedQuote with the line of its cell and the separator", () => {
    assert.deepEqual(fieldsOfImport(utf8('id,pop\nA,"P1\nB,P2\n')), {
        ...NO_TABLE,
        refusal: "unclosedQuote",
        format: "text",
        line: 2,
        separator: "comma",
    });
});

// The population of the first A is quoted over lines 2 and 3, so that the
// second A is on line 5 of the file and in row 4 of the table: the lines
// given are those of the file.
test("an individual in two lines of a text file is refused with its name and its two lines", () => {
    const bytes = utf8('id,pop\nA,"P\n1"\nB,P2\nA,P3\n');

    assert.deepEqual(fieldsOfImport(bytes), {
        ...NO_TABLE,
        refusal: "duplicateIndividual",
        format: "text",
        text: "A",
        line: 2,
        secondLine: 5,
    });
});

test("an individual in two rows of an xlsx is refused with its name and its two rows of the sheet", () => {
    const bytes = xlsxOf([["id", "pop"], ["A", "P1"], ["B", "P2"], ["A", "P3"]]);

    assert.deepEqual(fieldsOfImport(bytes), {
        ...NO_TABLE,
        refusal: "duplicateIndividual",
        format: "xlsx",
        text: "A",
        row: 2,
        secondRow: 4,
    });
});

test("a row of an xlsx with no individual is refused with its row of the sheet", () => {
    const bytes = xlsxOf([["id", "pop"], ["A", "P1"], [null, "P2"]]);

    assert.deepEqual(fieldsOfImport(bytes), {
        ...NO_TABLE,
        refusal: "emptyIndividual",
        format: "xlsx",
        row: 3,
    });
});

test("an error of Excel in the header is refused as headerError with its row, column and text", () => {
    const bytes = xlsxOf([["id", { error: "#N/A" }], ["A", 1]]);

    assert.deepEqual(fieldsOfImport(bytes), {
        ...NO_TABLE,
        refusal: "headerError",
        format: "xlsx",
        text: "#N/A",
        row: 1,
        column: 2,
    });
});

test("a column with no name and a value is refused as unnamedColumn with its column", () => {
    const bytes = xlsxOf([["id", null, "pop"], ["A", "x", "P1"]]);

    assert.deepEqual(fieldsOfImport(bytes), {
        ...NO_TABLE,
        refusal: "unnamedColumn",
        format: "xlsx",
        column: 2,
    });
});

test("two columns of one name are refused as duplicateColumn with the name and both columns", () => {
    const bytes = xlsxOf([["id", "pop", "pop"], ["A", 1, 2]]);

    assert.deepEqual(fieldsOfImport(bytes), {
        ...NO_TABLE,
        refusal: "duplicateColumn",
        format: "xlsx",
        text: "pop",
        column: 2,
        secondColumn: 3,
    });
});

test("a sheet of a header alone is refused as empty", () => {
    assert.deepEqual(fieldsOfImport(xlsxOf([["id", "pop"]])), {
        ...NO_TABLE,
        refusal: "empty",
        format: "xlsx",
    });
});

test("a table at C2 of 5 columns with a limit of 16 cells is refused as sheetTooLarge at row 5", async () => {
    // Rows 2 to 4 are 15 cells; C5 makes the rectangle 4 rows of 5
    // columns, 20 cells, past the limit.
    const fields = fieldsOfImport(await dataFile("wide_table_at_c2.xlsx"), { maxCells: 16 });

    assert.deepEqual(fields, {
        ...NO_TABLE,
        refusal: "sheetTooLarge",
        format: "xlsx",
        sheet: "Individuos",
        row: 2,
        column: 3,
        sheetRows: 4,
        sheetColumns: 5,
    });
});

test("a file one byte past the limit of bytes is refused as tooLarge with its size", async () => {
    const bytes = await dataFile("written.xlsx");

    const fields = fieldsOfImport(bytes, { maxBytes: bytes.length - 1 });

    assert.deepEqual(fields, {
        ...NO_TABLE,
        refusal: "tooLarge",
        format: "xlsx",
        size: bytes.length,
    });
});

test("a compound file of the old Office is refused as oldExcel", () => {
    // The mark every compound file starts with, followed by zeros: no
    // encrypted xlsx inside it.
    const bytes = new Uint8Array(512);
    bytes.set([0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]);

    assert.deepEqual(fieldsOfImport(bytes), { ...NO_TABLE, refusal: "oldExcel", format: "xlsx" });
});

test("encrypted.xlsx is refused as encrypted", async () => {
    assert.deepEqual(fieldsOfImport(await dataFile("encrypted.xlsx")), {
        ...NO_TABLE,
        refusal: "encrypted",
        format: "xlsx",
    });
});

test("a zip with no workbook is refused as notWorkbook", () => {
    const bytes = zipOf([["individuals.csv", "id,pop\nA,P1\n"]]);

    assert.deepEqual(fieldsOfImport(bytes), { ...NO_TABLE, refusal: "notWorkbook", format: "xlsx" });
});

test("a first sheet with no value is refused as emptySheet with its name", async () => {
    assert.deepEqual(fieldsOfImport(await dataFile("empty_first_sheet.xlsx")), {
        ...NO_TABLE,
        refusal: "emptySheet",
        format: "xlsx",
        sheet: "Notas",
    });
});

test("a cell saved with an error calamine does not know is refused as cellError with its text", async () => {
    assert.deepEqual(fieldsOfImport(await dataFile("getting_data.xlsx")), {
        ...NO_TABLE,
        refusal: "cellError",
        format: "xlsx",
        text: "#GETTING_DATA",
    });
});

test("an xlsx whose sheet is cut short is unreadable, with calamine's message, and nothing thrown", () => {
    const parts = workbookParts(sheetXml([["id", "pop"], ["A", "P1"]]));
    const [sheetName, sheet] = parts[4];
    // Cut inside the attribute r of the cell A1.
    parts[4] = [sheetName, sheet.slice(0, sheet.indexOf('r="A1"') + 4)];

    assert.deepEqual(fieldsOfImport(zipOf(parts)), {
        ...NO_TABLE,
        refusal: "unreadable",
        text: "Xml error: syntax error: attribute value not closed: `\"` not found before end of input",
    });
});

test("a zip cut short is unreadable, with the message of the zip crate", async () => {
    const bytes = await dataFile("written.xlsx");

    assert.deepEqual(fieldsOfImport(bytes.subarray(0, 500)), {
        ...NO_TABLE,
        refusal: "unreadable",
        text: "invalid Zip archive: Could not find EOCD",
    });
});

// A table of texts of two texts that says it holds 400,000,000, for which
// calamine reserved room before it read the first, and the package
// trapped, a failure that ends the worker, before table_io checked it.
test("unique_count.xlsx, whose table of texts says it holds 400,000,000 texts, is unreadable", async () => {
    assert.deepEqual(fieldsOfImport(await dataFile("unique_count.xlsx")), {
        ...NO_TABLE,
        refusal: "unreadable",
        text: "too many texts",
    });
    // The package still reads after it, as it does not after a trap.
    assert.equal(fieldsOfImport(await dataFile("empty_first_sheet.xlsx")).sheet, "Notas");
});

test("a limit of cells of 5 × 10^9 or −1, and a limit of bytes of 1.5, throw an Error", () => {
    const bytes = xlsxOf([["id", "pop"], ["A", "P1"]]);

    assert.throws(() => importTable(bytes, MAX_BYTES, 5e9, "", "", ""), {
        name: "Error",
        message: "max_cells is 5000000000, not a whole number from 0 to 4294967295",
    });
    assert.throws(() => importTable(bytes, MAX_BYTES, -1, "", "", ""), {
        name: "Error",
        message: "max_cells is -1, not a whole number from 0 to 4294967295",
    });
    assert.throws(() => importTable(bytes, 1.5, MAX_CELLS, "", "", ""), {
        name: "Error",
        message: "max_bytes is 1.5, not a whole number from 0 to 2^53",
    });
    assert.throws(() => importTable(bytes, Number.NaN, MAX_CELLS, "", "", ""), { name: "Error" });
    assert.throws(() => importTable(bytes, 2 ** 53 + 2, MAX_CELLS, "", "", ""), { name: "Error" });
});

test("the largest limits, 2^53 bytes and 4,294,967,295 cells, are taken", () => {
    const read = importTable(xlsxOf([["id", "pop"], ["A", "P1"]]), 2 ** 53, 4_294_967_295, "", "", "");
    try {
        assert.equal(read.format, "xlsx");
    } finally {
        read.free();
    }
});

test("an option that is not one of its strings throws an Error, and the package goes on", () => {
    const bytes = xlsxOf([["id", "pop"], ["A", "P1"]]);

    assert.throws(() => importTable(bytes, MAX_BYTES, MAX_CELLS, "latin1", "", ""), {
        name: "Error",
        message: 'the encoding is "latin1", not "", "utf-8" or "windows-1252"',
    });
    assert.throws(() => importTable(bytes, MAX_BYTES, MAX_CELLS, "", ",", ""), {
        name: "Error",
        message: 'the separator is ",", not "", "tab", "semicolon" or "comma"',
    });
    assert.throws(() => importTable(bytes, MAX_BYTES, MAX_CELLS, "", "", "."), {
        name: "Error",
        message: 'the decimal mark is ".", not "", "point" or "comma"',
    });
    assert.throws(() => parseFloat("1.5", ""), {
        name: "Error",
        message: 'the decimal mark is "", not "point" or "comma"',
    });
    assert.throws(() => floatText(1.5, "Point"), { name: "Error" });
    assert.throws(() => convertColumn("int", new Uint8Array([0]), new BigInt64Array([1n]), new Float64Array(), new Uint8Array(), [], "text", "point"), {
        name: "Error",
        message: 'the type is "int", not "integer", "float", "boolean" or "text"',
    });
    assert.throws(() => convertColumn("text", new Uint8Array([0]), new BigInt64Array(), new Float64Array(), new Uint8Array(), ["1"], "Float", "point"), {
        name: "Error",
    });

    assert.equal(fieldsOfImport(bytes).names[0], "A");
});

test("a column of an index out of range throws an Error, and the package goes on", () => {
    const read = importTable(xlsxOf([["id", "pop"], ["A", "P1"]]), MAX_BYTES, MAX_CELLS, "", "", "");
    try {
        assert.equal(read.numColumns, 1);
        assert.throws(() => read.columnName(1), { name: "Error", message: "no column 1: the table has 1 columns" });
        assert.throws(() => read.columnIntegers(7), { name: "Error" });
        assert.equal(read.columnName(0), "pop");
    } finally {
        read.free();
    }
});

test("an index of a column that is not a whole number from 0 to numColumns − 1 throws an Error", () => {
    const read = importTable(xlsxOf([["id", "a", "b"], ["A", 1, "x"]]), MAX_BYTES, MAX_CELLS, "", "", "");
    try {
        assert.equal(read.numColumns, 2);
        const methods = [
            "columnName",
            "columnNumber",
            "columnType",
            "columnMissing",
            "columnIntegers",
            "columnFloats",
            "columnBooleans",
            "columnTexts",
        ];
        for (const index of [2 ** 32, 1.5, undefined, -1, 2, Number.NaN, Infinity]) {
            for (const method of methods) {
                assert.throws(() => read[method](index), { name: "Error" }, `${method}(${index})`);
            }
        }
        assert.throws(() => read.columnName(2 ** 32), {
            name: "Error",
            message: "no column 4294967296: the table has 2 columns",
        });
        assert.throws(() => read.columnName(1.5), { message: "no column 1.5: the table has 2 columns" });
        assert.equal(read.columnName(1), "b");
    } finally {
        read.free();
    }
});

test("a column of a refusal has no index in range", () => {
    const read = importTable(zipOf([["a.txt", "x"]]), MAX_BYTES, MAX_CELLS, "", "", "");
    try {
        assert.equal(read.refusal, "notWorkbook");
        assert.throws(() => read.columnTexts(0), { name: "Error", message: "no column 0: the table has 0 columns" });
    } finally {
        read.free();
    }
});

/** The fields of a Conversion, which it frees. */
function fieldsOfConversion(conversion) {
    try {
        return {
            numFailed: conversion.numFailed,
            firstRow: conversion.firstRow,
            firstText: conversion.firstText,
            missing: conversion.missing,
            integers: conversion.integers,
            floats: conversion.floats,
            booleans: conversion.booleans,
            texts: conversion.texts,
        };
    } finally {
        conversion.free();
    }
}

/** The arrays of a text column of `texts`, as convertColumn takes them. */
function textColumn(texts) {
    return [
        "text",
        new Uint8Array(texts.length),
        new BigInt64Array(),
        new Float64Array(),
        new Uint8Array(),
        texts,
    ];
}

test("a text column 1, n.d. converted to float fails at row 2, n.d.", () => {
    const conversion = convertColumn(...textColumn(["1", "n.d."]), "float", "point");

    assert.deepEqual(fieldsOfConversion(conversion), {
        numFailed: 1,
        firstRow: 2,
        firstText: "n.d.",
        missing: new Uint8Array(),
        integers: new BigInt64Array(),
        floats: new Float64Array(),
        booleans: new Uint8Array(),
        texts: [],
    });
});

test("a text column 1, 2 converted to float gives the floats 1 and 2", () => {
    const conversion = convertColumn(...textColumn(["1", "2"]), "float", "point");

    assert.deepEqual(fieldsOfConversion(conversion), {
        numFailed: 0,
        firstRow: 0,
        firstText: "",
        missing: new Uint8Array([0, 0]),
        integers: new BigInt64Array(),
        floats: new Float64Array([1, 2]),
        booleans: new Uint8Array(),
        texts: [],
    });
});

test("an integer column with a missing value converted to text keeps it missing", () => {
    const conversion = convertColumn(
        "integer",
        new Uint8Array([0, 1, 0]),
        new BigInt64Array([-9223372036854775808n, 0n, 1152921504606846977n]),
        new Float64Array(),
        new Uint8Array(),
        [],
        "text",
        "comma",
    );

    const fields = fieldsOfConversion(conversion);

    assert.deepEqual(fields.missing, new Uint8Array([0, 1, 0]));
    assert.deepEqual(fields.texts, ["-9223372036854775808", "", "1152921504606846977"]);
});

test("a column whose arrays do not match throws an Error", () => {
    assert.throws(
        () => convertColumn("float", new Uint8Array([0, 0]), new BigInt64Array(), new Float64Array([1]), new Uint8Array(), [], "text", "point"),
        { name: "Error", message: "floats has 1 entries and missing 2" },
    );
    assert.throws(
        () => convertColumn("boolean", new Uint8Array([2]), new BigInt64Array(), new Float64Array(), new Uint8Array([1]), [], "text", "point"),
        { name: "Error", message: "missing holds 2, not 0 or 1" },
    );
});

// The floats of the test of float_text of docs/specs/values.md, each
// with the text node 26.8.2 printed for String(x) on the owner's Mac.
const FLOAT_TEXTS = [
    [1, "1"],
    [1.5, "1.5"],
    [0.1 + 0.2, "0.30000000000000004"],
    [1e20, "100000000000000000000"],
    [1e21, "1e+21"],
    [1.2345678901234568e21, "1.2345678901234568e+21"],
    [1e-6, "0.000001"],
    [1e-7, "1e-7"],
    [1.5e-7, "1.5e-7"],
    [-0, "0"],
    [5e-324, "5e-324"],
    [Number.MAX_VALUE, "1.7976931348623157e+308"],
    [Infinity, "Infinity"],
    [-Infinity, "-Infinity"],
    [NaN, "NaN"],
    [-1.5, "-1.5"],
    [-1.5e-7, "-1.5e-7"],
    [100000000000000.125, "100000000000000.12"],
    [12345678901234.0625, "12345678901234.062"],
    [1.8774474796234095e-7, "1.8774474796234095e-7"],
    [0.00008012601628271273, "0.00008012601628271273"],
    [94861526562499990000, "94861526562499990000"],
    [2.0372681319713593e-10, "2.0372681319713593e-10"],
    [2 ** -24, "5.960464477539063e-8"],
];

test("floatText writes each float of values.md's test as node's String does", () => {
    for (const [number, text] of FLOAT_TEXTS) {
        assert.equal(String(number), text, `node's String of ${text}`);
        assert.equal(floatText(number, "point"), String(number), `floatText of ${text}`);
    }
    assert.equal(floatText(1.5, "comma"), "1,5");
    assert.equal(floatText(1.5e-7, "comma"), "1,5e-7");
});

test("parseInteger gives 2^63 − 1 as the bigint 9223372036854775807n", () => {
    assert.equal(parseInteger("9223372036854775807"), 9223372036854775807n);
    assert.equal(parseInteger("9223372036854775808"), undefined);
    assert.equal(parseInteger("12.0"), undefined);
});

test("isMissing, parseFloat and parseBoolean are the rules of a value", () => {
    assert.equal(isMissing("NA"), true);
    assert.equal(isMissing("na"), false);
    assert.equal(parseFloat("-1,75", "comma"), -1.75);
    assert.equal(parseFloat("1,5", "point"), undefined);
    assert.equal(parseBoolean("True"), true);
    assert.equal(parseBoolean("VERDADERO"), undefined);
});

test("free() releases a TableRead and a Conversion once, and a read after it throws", () => {
    const read = importTable(zipOf([["a.txt", "x"]]), MAX_BYTES, MAX_CELLS, "", "", "");
    read.free();
    assert.throws(() => read.refusal, { message: "null pointer passed to rust" });

    const conversion = convertColumn(...textColumn(["1"]), "float", "point");
    conversion.free();
    assert.throws(() => conversion.floats, { message: "null pointer passed to rust" });
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
    // first, then test/table_io.d.ts, then a new release. The kept file is
    // the whole file wasm-bindgen writes, InitOutput among it, which the
    // comparison leaves out of both.
    const generated = await readFile(new URL("wasm/table_io.d.ts", packageDir), "utf8");
    const kept = await readFile(new URL("test/table_io.d.ts", packageDir), "utf8");

    assert.equal(withoutInitOutput(generated), withoutInitOutput(kept));
});
