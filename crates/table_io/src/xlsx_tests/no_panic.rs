//! No file makes `read_first_sheet` panic ("How it is verified", "No file
//! makes it panic", of `docs/specs/read.md`): a panic is a trap in the
//! wasm, which ends popnei_web's light worker.
//!
//! Two files, an xlsx written here and a compound file of the old Office
//! with `EncryptedPackage` among its bytes, are read cut short at every
//! length from 0 bytes to their whole size, and with each of their bytes in
//! turn replaced by its complement, the byte with every bit flipped. Every
//! copy has to return, a sheet, a refusal or an error, whichever; the tests
//! assert nothing of which. calamine is compiled without its checks of
//! overflow in these builds too, as in the package (`docs/architecture.md`,
//! section 6), so that a panic here is one the package has.
//!
//! A zip keeps the directory of its files at its end, so a copy of the xlsx
//! cut short stops in the reader of the zip, and a byte flipped in its
//! compressed data mostly stops at the checksum of its file: neither
//! reaches calamine's reading of the XML. So an xlsx is also built here
//! from the XML of its parts, stored in the zip uncompressed with the
//! checksum of each part worked out again, and each part table_io reads,
//! the workbook, its relationships, the table of texts, the sheet and the
//! styles, is cut short and flipped in the same way, alone, before the
//! parts are zipped again.

#![cfg(feature = "xlsx")]

use std::panic;

use crate::xlsx::{SheetCell, read_first_sheet};
use rust_xlsxwriter::{DocProperties, ExcelDateTime, Format, Formula, Workbook, XlsxError};

use crate::xlsx_tests::hand_written::stored_zip;

/// `MAX_SHEET_CELLS` of popnei_web, the limit its light worker gives.
const MAX_SHEET_CELLS: u32 = 2_000_000;

/// The eight bytes every compound file of the old Office starts with.
const COMPOUND_FILE_MARK: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// An xlsx with every kind the spec names for this test: a hidden first
/// sheet, then a sheet with a text, a number, a boolean, a date, a formula
/// saved with its value, and a population merged over two rows. Its date
/// of creation is fixed, so that its bytes, and the copy a failure names,
/// are the same at each run.
fn xlsx_of_every_kind() -> Result<Vec<u8>, XlsxError> {
    let mut workbook = Workbook::new();
    let written_on = ExcelDateTime::from_ymd(2026, 9, 28)?;
    workbook.set_properties(&DocProperties::new().set_creation_datetime(&written_on));

    let lists = workbook.add_worksheet().set_name("Listas")?;
    lists.write_string(0, 0, "a list of a form")?;
    lists.set_hidden(true);

    let individuals = workbook.add_worksheet().set_name("Individuos")?;
    for (column, name) in (0u16..).zip([
        "Individuo",
        "Población",
        "Altura",
        "Fecha",
        "Afectado",
        "Doble",
    ]) {
        individuals.write_string(0, column, name)?;
    }
    individuals.write_string(1, 0, "ind1")?;
    individuals.write_string(2, 0, "ind2")?;
    individuals.merge_range(1, 1, 2, 1, "Andalucía", &Format::new())?;
    individuals.write_number(1, 2, 1.75)?;
    individuals.write_number(2, 2, 1.62)?;
    let date_format = Format::new().set_num_format("dd/mm/yyyy");
    // 45425 is 13 May 2024 in Excel's system of 1900.
    individuals.write_number_with_format(1, 3, 45425.0, &date_format)?;
    individuals.write_number_with_format(2, 3, 45426.0, &date_format)?;
    individuals.write_boolean(1, 4, true)?;
    individuals.write_boolean(2, 4, false)?;
    individuals.write_formula(1, 5, Formula::new("=C2*2").set_result("3.5"))?;
    individuals.write_formula(2, 5, Formula::new("=C3*2").set_result("3.24"))?;
    // rust_xlsxwriter shows the active sheet, the first one unless told.
    individuals.set_active(true);

    workbook.save_to_buffer()
}

/// A compound file as an xlsx saved with a password is, as far as table_io
/// looks: the mark, zeros, and the name of the part `EncryptedPackage` in
/// UTF-16 little-endian at byte 600, as a compound file writes its names,
/// then zeros up to 1,024 bytes.
fn encrypted_compound_file() -> Vec<u8> {
    let mut bytes = COMPOUND_FILE_MARK.to_vec();
    bytes.resize(600, 0);
    bytes.extend("EncryptedPackage".encode_utf16().flat_map(u16::to_le_bytes));
    bytes.resize(1024, 0);
    bytes
}

/// Whether `read_first_sheet` returns for `bytes`, whatever it returns,
/// rather than panicking.
fn returns_for(bytes: &[u8]) -> bool {
    panic::catch_unwind(|| {
        // What the read returns is not asserted: any sheet, refusal or error
        // will do.
        let _read = read_first_sheet(bytes, MAX_SHEET_CELLS);
    })
    .is_ok()
}

/// The lengths from 0 to the whole size of `bytes` at which the file
/// `into_file` makes of the copy cut short panics.
fn lengths_that_panic(bytes: &[u8], into_file: impl Fn(&[u8]) -> Vec<u8>) -> Vec<usize> {
    (0..=bytes.len())
        .filter(|&length| {
            bytes
                .get(..length)
                .is_some_and(|cut| !returns_for(&into_file(cut)))
        })
        .collect()
}

/// The positions of the bytes of `bytes` whose complement, in a copy with
/// that byte alone changed, panics in the file `into_file` makes of it.
fn flipped_bytes_that_panic(bytes: &[u8], into_file: impl Fn(&[u8]) -> Vec<u8>) -> Vec<usize> {
    (0..bytes.len())
        .filter(|&position| {
            let copy: Vec<u8> = (0..)
                .zip(bytes)
                .map(|(other_position, &byte)| {
                    if other_position == position {
                        !byte
                    } else {
                        byte
                    }
                })
                .collect();
            !returns_for(&into_file(&copy))
        })
        .collect()
}

/// The parts of the xlsx of every kind built by hand, each its name in the
/// zip and its XML: a hidden first sheet, `Listas`, then `Individuos`, with
/// its texts in the table of texts, a number, a date with the format
/// `dd/mm/yyyy`, a boolean, a formula saved with its value, and a
/// population merged over two rows.
const PARTS_OF_EVERY_KIND: [(&str, &str); 8] = [
    (
        "[Content_Types].xml",
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/><Override PartName="/xl/worksheets/sheet2.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/><Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/><Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/></Types>"#,
    ),
    (
        "_rels/.rels",
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#,
    ),
    (
        "xl/workbook.xml",
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Listas" sheetId="1" state="hidden" r:id="rId1"/><sheet name="Individuos" sheetId="2" r:id="rId2"/></sheets></workbook>"#,
    ),
    (
        "xl/_rels/workbook.xml.rels",
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet2.xml"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/><Relationship Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"#,
    ),
    (
        "xl/sharedStrings.xml",
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="10" uniqueCount="10"><si><t>Individuo</t></si><si><t>Población</t></si><si><t>Altura</t></si><si><t>Fecha</t></si><si><t>Afectado</t></si><si><t>Doble</t></si><si><t>ind1</t></si><si><t>Andalucía</t></si><si><t>ind2</t></si><si><t>a list of a form</t></si></sst>"#,
    ),
    (
        "xl/styles.xml",
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><numFmts count="1"><numFmt numFmtId="164" formatCode="dd/mm/yyyy"/></numFmts><cellXfs count="2"><xf numFmtId="0"/><xf numFmtId="164" applyNumberFormat="1"/></cellXfs></styleSheet>"#,
    ),
    (
        "xl/worksheets/sheet1.xml",
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1" t="s"><v>9</v></c></row></sheetData></worksheet>"#,
    ),
    (
        "xl/worksheets/sheet2.xml",
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><dimension ref="A1:F3"/><sheetData><row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c><c r="C1" t="s"><v>2</v></c><c r="D1" t="s"><v>3</v></c><c r="E1" t="s"><v>4</v></c><c r="F1" t="s"><v>5</v></c></row><row r="2"><c r="A2" t="s"><v>6</v></c><c r="B2" t="s"><v>7</v></c><c r="C2"><v>1.75</v></c><c r="D2" s="1"><v>45425</v></c><c r="E2" t="b"><v>1</v></c><c r="F2"><f>C2*2</f><v>3.5</v></c></row><row r="3"><c r="A3" t="s"><v>8</v></c><c r="B3"/><c r="C3"><v>1.62</v></c><c r="D3" s="1"><v>45426</v></c><c r="E3" t="b"><v>0</v></c><c r="F3"><f>C3*2</f><v>3.24</v></c></row></sheetData><mergeCells count="1"><mergeCell ref="B2:B3"/></mergeCells></worksheet>"#,
    ),
];

/// The parts of [`PARTS_OF_EVERY_KIND`] that table_io reads, through
/// calamine, and that the tests damage: the workbook, its relationships,
/// the table of texts, the sheet `Individuos` and the styles.
const DAMAGED_PARTS: [&str; 5] = [
    "xl/workbook.xml",
    "xl/_rels/workbook.xml.rels",
    "xl/sharedStrings.xml",
    "xl/worksheets/sheet2.xml",
    "xl/styles.xml",
];

/// The xlsx of [`PARTS_OF_EVERY_KIND`], stored uncompressed, with the XML
/// of the part `part_name` replaced by `part_bytes`.
fn xlsx_with_part(part_name: &str, part_bytes: &[u8]) -> Vec<u8> {
    let files: Vec<(&str, &[u8])> = PARTS_OF_EVERY_KIND
        .iter()
        .map(|&(name, xml)| {
            if name == part_name {
                (name, part_bytes)
            } else {
                (name, xml.as_bytes())
            }
        })
        .collect();
    stored_zip(&files)
}

/// The parts of [`DAMAGED_PARTS`], each its name and its XML.
fn damaged_parts() -> Vec<(&'static str, &'static str)> {
    PARTS_OF_EVERY_KIND
        .into_iter()
        .filter(|(name, _)| DAMAGED_PARTS.contains(name))
        .collect()
}

#[test]
fn the_xlsx_of_every_kind_is_a_sheet_when_whole() {
    // So that the copies below are copies of a file calamine reads.
    let bytes = xlsx_of_every_kind().unwrap();

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!(sheet.name, "Individuos");
    assert_eq!((sheet.num_rows, sheet.num_columns), (3, 6));
}

#[test]
fn the_xlsx_of_every_kind_cut_short_at_any_length_does_not_panic() {
    let bytes = xlsx_of_every_kind().unwrap();

    let lengths = lengths_that_panic(&bytes, <[u8]>::to_vec);

    assert!(
        lengths.is_empty(),
        "the xlsx of {} bytes panics when cut at these lengths: {lengths:?}",
        bytes.len()
    );
}

#[test]
fn the_xlsx_of_every_kind_with_any_byte_flipped_does_not_panic() {
    let bytes = xlsx_of_every_kind().unwrap();

    let positions = flipped_bytes_that_panic(&bytes, <[u8]>::to_vec);

    assert!(
        positions.is_empty(),
        "the xlsx of {} bytes panics with the byte at these positions, from 0, flipped: {positions:?}",
        bytes.len()
    );
}

#[test]
fn the_encrypted_compound_file_cut_short_at_any_length_does_not_panic() {
    let bytes = encrypted_compound_file();

    let lengths = lengths_that_panic(&bytes, <[u8]>::to_vec);

    assert!(
        lengths.is_empty(),
        "the compound file of {} bytes panics when cut at these lengths: {lengths:?}",
        bytes.len()
    );
}

#[test]
fn the_encrypted_compound_file_with_any_byte_flipped_does_not_panic() {
    let bytes = encrypted_compound_file();

    let positions = flipped_bytes_that_panic(&bytes, <[u8]>::to_vec);

    assert!(
        positions.is_empty(),
        "the compound file of {} bytes panics with the byte at these positions, from 0, flipped: {positions:?}",
        bytes.len()
    );
}

#[test]
fn the_xlsx_of_every_kind_built_from_its_parts_is_a_sheet_when_whole() {
    // So that the damaged copies below are copies of a file whose every
    // part calamine reads: the table of texts, the date format of the
    // styles, the merged range and the hidden first sheet.
    // No part is named "", so none is replaced.
    let bytes = xlsx_with_part("", b"");

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!(sheet.name, "Individuos");
    assert_eq!(
        (
            sheet.first_row,
            sheet.first_column,
            sheet.num_rows,
            sheet.num_columns
        ),
        (1, 1, 3, 6)
    );
    let text = |cell_text: &str| SheetCell::Text(cell_text.to_owned());
    assert_eq!(
        sheet.cells,
        [
            text("Individuo"),
            text("Población"),
            text("Altura"),
            text("Fecha"),
            text("Afectado"),
            text("Doble"),
            text("ind1"),
            text("Andalucía"),
            SheetCell::Number(1.75),
            text("2024-05-13"),
            SheetCell::Bool(true),
            SheetCell::Number(3.5),
            text("ind2"),
            text("Andalucía"),
            SheetCell::Number(1.62),
            text("2024-05-14"),
            SheetCell::Bool(false),
            SheetCell::Number(3.24),
        ]
    );
}

#[test]
fn each_part_of_the_xlsx_of_every_kind_cut_short_at_any_length_does_not_panic() {
    // A cut only shortens the XML, so it cannot make the count of the
    // table of texts larger (below).
    let panics: Vec<(&str, Vec<usize>)> = damaged_parts()
        .into_iter()
        .map(|(part_name, xml)| {
            let lengths = lengths_that_panic(xml.as_bytes(), |cut| xlsx_with_part(part_name, cut));
            (part_name, lengths)
        })
        .filter(|(_, lengths)| !lengths.is_empty())
        .collect();

    assert_eq!(damaged_parts().len(), DAMAGED_PARTS.len());
    assert!(
        panics.is_empty(),
        "these parts panic when cut at these lengths: {panics:?}"
    );
}

#[test]
fn each_part_of_the_xlsx_of_every_kind_with_any_byte_flipped_does_not_panic() {
    // calamine reserves room for as many texts as `uniqueCount` of the
    // table of texts says, before it reads them, and natively a reservation
    // too large for memory aborts the process, which `catch_unwind` cannot
    // catch, instead of panicking. A flip cannot make that count larger:
    // the complement of a digit, 0x30 to 0x39, is 0xC6 to 0xCF, never a
    // digit, so no digit is added to a count, and a digit flipped leaves a
    // count calamine does not parse. A byte whose complement is a digit is
    // one of 0xC6 to 0xCF, which the parts do not hold, as asserted here,
    // so that a change of the parts that breaks this fails the test before
    // it aborts it.
    for (part_name, xml) in damaged_parts() {
        let positions: Vec<usize> = (0..)
            .zip(xml.as_bytes())
            .filter(|&(_, byte)| (!byte).is_ascii_digit())
            .map(|(position, _)| position)
            .collect();
        assert!(
            positions.is_empty(),
            "{part_name} has bytes whose complement is a digit at {positions:?}"
        );
    }
    let panics: Vec<(&str, Vec<usize>)> = damaged_parts()
        .into_iter()
        .map(|(part_name, xml)| {
            let positions =
                flipped_bytes_that_panic(xml.as_bytes(), |copy| xlsx_with_part(part_name, copy));
            (part_name, positions)
        })
        .filter(|(_, positions)| !positions.is_empty())
        .collect();

    assert_eq!(damaged_parts().len(), DAMAGED_PARTS.len());
    assert!(
        panics.is_empty(),
        "these parts panic with the byte at these positions, from 0, flipped: {panics:?}"
    );
}
