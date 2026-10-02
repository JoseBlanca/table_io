//! An xlsx written by hand from the XML of its one worksheet, for the
//! files rust_xlsxwriter does not write: a merged range written from its
//! last cell, two ranges that overlap, a value in a merged range's other
//! cells, a cell written more than once, a text longer than the 32,767
//! characters Excel allows, and a workbook in the date system of 1904.
//!
//! The xlsx is a zip whose files are stored as they are, with no
//! compression, which a zip may do and calamine reads, so that it needs no
//! crate of zip. It holds the five files calamine needs to find the first
//! sheet, and the texts of the cells are written in the cells themselves,
//! `t="inlineStr"`, so that there is no table of shared texts. A workbook
//! of 1904 holds a sixth, `xl/styles.xml`, where the formats of its
//! numbers are, which calamine finds by that name with no relationship to
//! it. `src/xlsx_tests/no_panic.rs` zips with [`stored_zip`] the parts of a
//! workbook of its own, some of them damaged, and `src/xlsx_tests/parts.rs` the
//! parts of a workbook with their names or their XML changed, or with a
//! table of shared texts added, `xl/sharedStrings.xml`, which calamine too
//! finds by that name; `tests/write_fixtures.rs` writes one such file into
//! `tests/data/`. The tests of the library, `src/xlsx_tests/` and
//! `src/bounds_tests.rs`, take it as the module
//! `crate::xlsx_tests::hand_written`, loaded once by its path.
//!
//! Each module that takes it is declared `#[cfg(test)]`, so that clippy
//! reads its `unwrap`s as those of a test: a size of a test's file past a
//! `u16` or a `u32` is a mistake of the test, which the panic reports.

/// The xlsx of one worksheet, `Sheet1`, whose element `<worksheet>` holds
/// `worksheet_body`: its `<sheetData>`, and its `<mergeCells>` when it has
/// merged ranges.
pub fn xlsx_of_worksheet(worksheet_body: &str) -> Vec<u8> {
    xlsx_of_parts(&parts_of_worksheet(worksheet_body))
}

/// The parts [`xlsx_of_worksheet`] zips, each a name and its XML, in the
/// order of the zip, for a test that adds a part, or changes the names or
/// the XML of the parts, before it zips them with [`xlsx_of_parts`].
pub fn parts_of_worksheet(worksheet_body: &str) -> Vec<(String, String)> {
    [
        ("[Content_Types].xml", CONTENT_TYPES.to_owned()),
        ("_rels/.rels", PACKAGE_RELATIONSHIPS.to_owned()),
        ("xl/workbook.xml", workbook("")),
        (
            "xl/_rels/workbook.xml.rels",
            WORKBOOK_RELATIONSHIPS.to_owned(),
        ),
        ("xl/worksheets/sheet1.xml", worksheet(worksheet_body)),
    ]
    .into_iter()
    .map(|(name, xml)| (name.to_owned(), xml))
    .collect()
}

/// The zip of `parts`, each a name and its XML, as [`stored_zip`] writes
/// it.
pub fn xlsx_of_parts(parts: &[(String, String)]) -> Vec<u8> {
    let files: Vec<(&str, &str)> = parts
        .iter()
        .map(|(name, xml)| (name.as_str(), xml.as_str()))
        .collect();
    stored_zip(&files)
}

/// The XML of a table of texts, the part `xl/sharedStrings.xml`, which
/// holds once each text the cells hold, a cell `t="s"` giving the place of
/// its text there, from 0. `sst_attributes` are the attributes of its root
/// element `<sst>`, such as `uniqueCount="2"`, the number of texts the
/// table says it holds; each of `texts` is an element `<si>`.
pub fn shared_strings(sst_attributes: &str, texts: &[&str]) -> String {
    let text_elements: String = texts
        .iter()
        .map(|text| format!("<si><t>{text}</t></si>"))
        .collect();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" {sst_attributes}>{text_elements}</sst>"#
    )
}

/// The xlsx of one worksheet, as [`xlsx_of_worksheet`] gives it, in the
/// date system of 1904, `date1904="1"` in the `<workbookPr>` of its
/// workbook, with the formats of numbers `number_formats`: a cell written
/// with `s="1"` has the first of them, `s="2"` the second, and so on, and
/// a cell with no `s` has none.
pub fn xlsx_of_1904_worksheet(worksheet_body: &str, number_formats: &[&str]) -> Vec<u8> {
    xlsx_of_parts(&parts_of_1904_worksheet(worksheet_body, number_formats))
}

/// The parts [`xlsx_of_1904_worksheet`] zips, each a name and its XML, in
/// the order of the zip, for a test that changes their names or their XML
/// before it zips them with [`xlsx_of_parts`].
pub fn parts_of_1904_worksheet(
    worksheet_body: &str,
    number_formats: &[&str],
) -> Vec<(String, String)> {
    [
        ("[Content_Types].xml", CONTENT_TYPES.to_owned()),
        ("_rels/.rels", PACKAGE_RELATIONSHIPS.to_owned()),
        ("xl/workbook.xml", workbook(r#"<workbookPr date1904="1"/>"#)),
        (
            "xl/_rels/workbook.xml.rels",
            WORKBOOK_RELATIONSHIPS.to_owned(),
        ),
        ("xl/styles.xml", styles(number_formats)),
        ("xl/worksheets/sheet1.xml", worksheet(worksheet_body)),
    ]
    .into_iter()
    .map(|(name, xml)| (name.to_owned(), xml))
    .collect()
}

/// The text of a cell of `reference`, such as `A1`, written in the cell.
pub fn text_cell(reference: &str, cell_text: &str) -> String {
    format!(r#"<c r="{reference}" t="inlineStr"><is><t>{cell_text}</t></is></c>"#)
}

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>"#;

const PACKAGE_RELATIONSHIPS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#;

/// The XML of the worksheet whose element `<worksheet>` holds
/// `worksheet_body`.
fn worksheet(worksheet_body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">{worksheet_body}</worksheet>"#
    )
}

/// The XML of the workbook of the one sheet `Sheet1`, with
/// `workbook_properties`, a `<workbookPr>` or nothing, before its sheets.
fn workbook(workbook_properties: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">{workbook_properties}<sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets></workbook>"#
    )
}

/// The XML of the styles of a workbook whose formats of numbers are
/// `number_formats`. Each format is a `<numFmt>` of its own number, from
/// 164, the first that is not one of Excel's built-in formats, as Excel and
/// rust_xlsxwriter number them; and each has a style of a cell, an `<xf>`
/// of `<cellXfs>` that names it, whose place in `<cellXfs>` is the `s` of a
/// cell. The first `<xf>`, `s="0"`, is the style of a cell with no `s`,
/// the format General. calamine takes a cell for a date or a duration by
/// the format its `<xf>` names, and reads nothing else of the styles.
fn styles(number_formats: &[&str]) -> String {
    let format_ids = (164..).map(|format_id: u32| format_id.to_string());
    let (num_fmts, cell_xfs): (String, String) = number_formats
        .iter()
        .zip(format_ids)
        .map(|(format_code, format_id)| {
            (
                format!(r#"<numFmt numFmtId="{format_id}" formatCode="{format_code}"/>"#),
                format!(r#"<xf numFmtId="{format_id}" applyNumberFormat="1"/>"#),
            )
        })
        .unzip();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><numFmts>{num_fmts}</numFmts><cellXfs><xf numFmtId="0"/>{cell_xfs}</cellXfs></styleSheet>"#
    )
}

const WORKBOOK_RELATIONSHIPS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#;

/// A zip of `files`, each a name and its bytes, stored with no
/// compression: a local header and the bytes of each file, with their
/// CRC-32, then the central directory that lists them, then the record that
/// ends the zip. The bytes need not be XML nor UTF-8.
pub fn stored_zip<Contents: AsRef<[u8]>>(files: &[(&str, Contents)]) -> Vec<u8> {
    let mut zip = Vec::new();
    let mut central_directory = Vec::new();
    for (name, contents) in files {
        let contents = contents.as_ref();
        let offset = u32::try_from(zip.len()).unwrap();
        let name_length = u16::try_from(name.len()).unwrap();
        let size = u32::try_from(contents.len()).unwrap();
        let crc = crc32(contents);

        zip.extend(0x0403_4b50u32.to_le_bytes());
        // Version 2.0, no flags, stored, a time of 0 and 1 January 1980.
        zip.extend([20, 0, 0, 0, 0, 0, 0, 0, 0x21, 0]);
        zip.extend(crc.to_le_bytes());
        zip.extend(size.to_le_bytes());
        zip.extend(size.to_le_bytes());
        zip.extend(name_length.to_le_bytes());
        zip.extend(0u16.to_le_bytes());
        zip.extend(name.as_bytes());
        zip.extend(contents);

        central_directory.extend(0x0201_4b50u32.to_le_bytes());
        central_directory.extend([20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0x21, 0]);
        central_directory.extend(crc.to_le_bytes());
        central_directory.extend(size.to_le_bytes());
        central_directory.extend(size.to_le_bytes());
        central_directory.extend(name_length.to_le_bytes());
        // No extra field, no comment, disk 0, no attributes.
        central_directory.extend([0; 12]);
        central_directory.extend(offset.to_le_bytes());
        central_directory.extend(name.as_bytes());
    }
    let directory_offset = u32::try_from(zip.len()).unwrap();
    let directory_size = u32::try_from(central_directory.len()).unwrap();
    let num_files = u16::try_from(files.len()).unwrap();
    zip.extend(central_directory);
    zip.extend(0x0605_4b50u32.to_le_bytes());
    zip.extend([0, 0, 0, 0]);
    zip.extend(num_files.to_le_bytes());
    zip.extend(num_files.to_le_bytes());
    zip.extend(directory_size.to_le_bytes());
    zip.extend(directory_offset.to_le_bytes());
    zip.extend(0u16.to_le_bytes());
    zip
}

/// The CRC-32 of `bytes`, as a zip checks each file with it, worked out a
/// byte at a time with a table of the CRC of each byte, so that a part of
/// 50 MB takes a fraction of a second in a test.
fn crc32(bytes: &[u8]) -> u32 {
    let table: [u32; 256] = std::array::from_fn(|byte| {
        let mut crc = u32::try_from(byte).unwrap();
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
        crc
    });
    let mut crc = u32::MAX;
    for byte in bytes {
        let table_index = usize::from(crc.to_le_bytes()[0] ^ byte);
        crc = (crc >> 8) ^ table.get(table_index).copied().unwrap_or_default();
    }
    !crc
}
