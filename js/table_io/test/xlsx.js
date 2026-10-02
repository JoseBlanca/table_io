// An xlsx written by the test, as the bytes of a zip whose parts are
// stored without compression: the five parts calamine and table_io read of
// a workbook of one sheet, and the cells the test gives. Node has no zip of
// its own and the package has no npm dependency, so the zip is written
// here, by its records: the header of each part, the directory of the
// parts, and the record that ends the directory.

/** The CRC-32 of the zip, of each byte value, by its polynomial 0xEDB88320. */
const crcTable = Array.from({ length: 256 }, (_, byteValue) => {
    let crc = byteValue;
    for (let bit = 0; bit < 8; bit++) {
        crc = crc & 1 ? 0xedb88320 ^ (crc >>> 1) : crc >>> 1;
    }
    return crc >>> 0;
});

/** The CRC-32 of `bytes`. */
function crc32(bytes) {
    let crc = 0xffffffff;
    for (const byte of bytes) {
        crc = crcTable[(crc ^ byte) & 0xff] ^ (crc >>> 8);
    }
    return (crc ^ 0xffffffff) >>> 0;
}

/**
 * The bytes of a zip that holds `parts`, an array of [name, text], each
 * stored without compression, the texts in UTF-8.
 */
export function zipOf(parts) {
    const encoder = new TextEncoder();
    const localRecords = [];
    const directoryRecords = [];
    let offset = 0;
    for (const [name, text] of parts) {
        const nameBytes = encoder.encode(name);
        const contents = encoder.encode(text);
        const crc = crc32(contents);
        const local = new DataView(new ArrayBuffer(30));
        local.setUint32(0, 0x04034b50, true);
        local.setUint16(4, 20, true);
        local.setUint32(14, crc, true);
        local.setUint32(18, contents.length, true);
        local.setUint32(22, contents.length, true);
        local.setUint16(26, nameBytes.length, true);
        localRecords.push(new Uint8Array(local.buffer), nameBytes, contents);
        const central = new DataView(new ArrayBuffer(46));
        central.setUint32(0, 0x02014b50, true);
        central.setUint16(4, 20, true);
        central.setUint16(6, 20, true);
        central.setUint32(16, crc, true);
        central.setUint32(20, contents.length, true);
        central.setUint32(24, contents.length, true);
        central.setUint16(28, nameBytes.length, true);
        central.setUint32(42, offset, true);
        directoryRecords.push(new Uint8Array(central.buffer), nameBytes);
        offset += 30 + nameBytes.length + contents.length;
    }
    const directorySize = directoryRecords.reduce((size, record) => size + record.length, 0);
    const end = new DataView(new ArrayBuffer(22));
    end.setUint32(0, 0x06054b50, true);
    end.setUint16(8, parts.length, true);
    end.setUint16(10, parts.length, true);
    end.setUint32(12, directorySize, true);
    end.setUint32(16, offset, true);
    const records = [...localRecords, ...directoryRecords, new Uint8Array(end.buffer)];
    const bytes = new Uint8Array(records.reduce((size, record) => size + record.length, 0));
    let position = 0;
    for (const record of records) {
        bytes.set(record, position);
        position += record.length;
    }
    return bytes;
}

/** `text` with the five characters XML escapes escaped. */
function escaped(text) {
    return text
        .replaceAll("&", "&amp;")
        .replaceAll("<", "&lt;")
        .replaceAll(">", "&gt;")
        .replaceAll('"', "&quot;");
}

/** The letters of the column `column` of a sheet, from 1 for A. */
function columnLetters(column) {
    let letters = "";
    for (let rest = column; rest > 0; rest = Math.floor((rest - 1) / 26)) {
        letters = String.fromCharCode(65 + ((rest - 1) % 26)) + letters;
    }
    return letters;
}

/**
 * The XML of a cell at `reference` holding `cell`: a string as a text, a
 * number, a boolean, `{ error: "#N/A" }` as an error of Excel, and null as
 * no cell at all.
 */
function cellXml(reference, cell) {
    if (cell === null) {
        return "";
    }
    if (typeof cell === "string") {
        return `<c r="${reference}" t="inlineStr"><is><t xml:space="preserve">${escaped(cell)}</t></is></c>`;
    }
    if (typeof cell === "number") {
        return `<c r="${reference}"><v>${cell}</v></c>`;
    }
    if (typeof cell === "boolean") {
        return `<c r="${reference}" t="b"><v>${cell ? 1 : 0}</v></c>`;
    }
    return `<c r="${reference}" t="e"><v>${escaped(cell.error)}</v></c>`;
}

/** The XML of the worksheet whose rows are `rows`, from A1. */
export function sheetXml(rows) {
    const rowsXml = rows
        .map((cells, rowIndex) => {
            const row = rowIndex + 1;
            const cellsXml = cells
                .map((cell, columnIndex) => cellXml(`${columnLetters(columnIndex + 1)}${row}`, cell))
                .join("");
            return `<row r="${row}">${cellsXml}</row>`;
        })
        .join("");
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
        '<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">' +
        `<sheetData>${rowsXml}</sheetData></worksheet>`
    );
}

/**
 * The parts of a workbook of one sheet, named `sheetName`, whose worksheet
 * is the XML `worksheet`, as [name, text].
 */
export function workbookParts(worksheet, sheetName = "Sheet1") {
    return [
        [
            "[Content_Types].xml",
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
                '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">' +
                '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>' +
                '<Default Extension="xml" ContentType="application/xml"/>' +
                '<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>' +
                '<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>' +
                "</Types>",
        ],
        [
            "_rels/.rels",
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
                '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">' +
                '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>' +
                "</Relationships>",
        ],
        [
            "xl/workbook.xml",
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
                '<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" ' +
                'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">' +
                `<sheets><sheet name="${escaped(sheetName)}" sheetId="1" r:id="rId1"/></sheets></workbook>`,
        ],
        [
            "xl/_rels/workbook.xml.rels",
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
                '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">' +
                '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>' +
                "</Relationships>",
        ],
        ["xl/worksheets/sheet1.xml", worksheet],
    ];
}

/**
 * The bytes of an xlsx of one sheet, `sheetName`, whose rows are `rows`
 * from A1, each an array of cells as `cellXml` takes them.
 */
export function xlsxOf(rows, sheetName = "Sheet1") {
    return zipOf(workbookParts(sheetXml(rows), sheetName));
}
