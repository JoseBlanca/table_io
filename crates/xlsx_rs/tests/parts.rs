//! The parts of the zip xlsx_rs reads itself before calamine ("What xlsx_rs
//! reads before calamine" of `docs/specs/read.md`): every part read to its
//! end, so that a part whose checksum fails refuses the file, and the bytes
//! so read counted and bounded; the date system, from the `workbookPr` that
//! is a direct child of the root of the workbook; and the bounds of the
//! parts calamine reads whole when it opens the file.

#[expect(
    dead_code,
    reason = "the texts here are in a table of texts, not written in the cells by hand"
)]
mod hand_written;

use std::error::Error;
use std::io::{Cursor, Write};
use std::ops::Range;

use rust_xlsxwriter::{Workbook, XlsxError};
use xlsx_rs::{
    ReadError, Sheet, SheetCell, read_first_sheet, read_first_sheet_within_text_table_bytes,
    read_first_sheet_within_unzipped_bytes,
};
use zip::result::ZipError;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::hand_written::{
    parts_of_1904_worksheet, parts_of_worksheet, shared_strings, stored_zip, xlsx_of_parts,
};

/// `MAX_SHEET_CELLS` of popnei_web, the limit its light worker gives.
const MAX_SHEET_CELLS: u32 = 2_000_000;

/// An xlsx written by rust_xlsxwriter with a table of 200 individuals, each
/// with a name, a population and a height, under a header.
fn xlsx_of_individuals() -> Result<Vec<u8>, XlsxError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "id")?;
    worksheet.write_string(0, 1, "pop")?;
    worksheet.write_string(0, 2, "height")?;
    for row in 1..=200u32 {
        worksheet.write_string(row, 0, format!("ind{row}"))?;
        let population = if row.is_multiple_of(2) {
            "north"
        } else {
            "south"
        };
        worksheet.write_string(row, 1, population)?;
        worksheet.write_number(row, 2, f64::from(row) * 1.37)?;
    }
    workbook.save_to_buffer()
}

/// The positions in `bytes`, an xlsx, of the compressed data of the part
/// `part_name`, as the zip's directory gives them.
fn compressed_data_of(bytes: &[u8], part_name: &str) -> Result<Range<usize>, Box<dyn Error>> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))?;
    let part = archive.by_name(part_name)?;
    let start = usize::try_from(part.data_start().ok_or("no start of the data")?)?;
    let length = usize::try_from(part.compressed_size())?;
    let end = start
        .checked_add(length)
        .ok_or("the data ends past a usize")?;
    Ok(start..end)
}

/// The number of bytes the parts of `bytes`, an xlsx, hold unzipped, as
/// the zip's directory gives them.
fn unzipped_size_of(bytes: &[u8]) -> Result<u64, ZipError> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))?;
    let mut unzipped_size: u64 = 0;
    for part_index in 0..archive.len() {
        let part_size = archive.by_index(part_index)?.size();
        unzipped_size = unzipped_size.saturating_add(part_size);
    }
    Ok(unzipped_size)
}

/// The messages of the zip crate for a part whose compressed bytes are
/// damaged: its checksum, and the errors of its reader of deflate, the
/// compression of a zip.
const ZIP_MESSAGES_OF_DAMAGED_DATA: [&str; 3] = [
    "Invalid checksum",
    "corrupt deflate stream",
    "incomplete deflate stream",
];

// Every copy is either read with the cells of the unchanged file or refused
// with the message of the zip crate, before calamine reads a cell: calamine
// alone read such copies with cells missing, and xlsx_rs, before it read
// every part to its end, refused some with calamine's message of the XML
// and some as an empty sheet or a sheet too large.
#[test]
fn a_sheet_damaged_in_its_compressed_bytes_is_refused_by_the_zip_or_read_whole() {
    let bytes = xlsx_of_individuals().unwrap();
    let unchanged_sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();
    assert_eq!(unchanged_sheet.num_rows, 201);
    let sheet_data = compressed_data_of(&bytes, "xl/worksheets/sheet1.xml").unwrap();
    assert!(sheet_data.len() > 1_000);

    let mut num_refused: u32 = 0;
    let mut other_reads: Vec<(usize, Result<Sheet, ReadError>)> = Vec::new();
    for position in sheet_data {
        let mut damaged_bytes = bytes.clone();
        damaged_bytes[position] = !damaged_bytes[position];
        let read = read_first_sheet(&damaged_bytes, MAX_SHEET_CELLS);
        let is_refused_by_the_zip = matches!(
            &read,
            Err(ReadError::Unreadable(message))
                if ZIP_MESSAGES_OF_DAMAGED_DATA.contains(&message.as_str())
        );
        if is_refused_by_the_zip {
            num_refused = num_refused.checked_add(1).unwrap();
        } else if read.as_ref() != Ok(&unchanged_sheet) {
            other_reads.push((position, read));
        }
    }

    assert_eq!(
        other_reads.len(),
        0,
        "{} copies read otherwise, the first {:?}",
        other_reads.len(),
        other_reads
            .first()
            .map(|(position, read)| (position, read.as_ref().err()))
    );
    assert!(num_refused > 1_000);
}

#[test]
fn the_first_500_bytes_of_an_xlsx_are_unreadable_with_the_zip_crates_message() {
    let bytes = xlsx_of_individuals().unwrap();

    let read = read_first_sheet(&bytes[..500], MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable(
            "invalid Zip archive: Could not find EOCD".to_owned()
        ))
    );
}

#[test]
fn a_file_that_unzips_to_more_bytes_than_the_bound_is_unreadable() {
    let bytes = xlsx_of_individuals().unwrap();
    assert!(unzipped_size_of(&bytes).unwrap() > 4_000);

    let read = read_first_sheet_within_unzipped_bytes(&bytes, MAX_SHEET_CELLS, 4_000);

    assert_eq!(
        read,
        Err(ReadError::Unreadable(
            "the file unzips to more than 4,000 bytes".to_owned()
        ))
    );
}

#[test]
fn a_file_that_unzips_to_as_many_bytes_as_the_bound_is_read() {
    let bytes = xlsx_of_individuals().unwrap();
    let unzipped_size = unzipped_size_of(&bytes).unwrap();

    let bounded_read =
        read_first_sheet_within_unzipped_bytes(&bytes, MAX_SHEET_CELLS, unzipped_size);
    let one_byte_less = read_first_sheet_within_unzipped_bytes(
        &bytes,
        MAX_SHEET_CELLS,
        unzipped_size.checked_sub(1).unwrap(),
    );

    let sheet = bounded_read.unwrap();
    assert_eq!(sheet.cells[0], SheetCell::Text("id".to_owned()));
    assert_eq!(sheet.num_rows, 201);
    assert!(matches!(one_byte_less, Err(ReadError::Unreadable(_))));
}

/// The sheet of one cell, A1, 43963 with the format `dd/mm/yyyy`, in a
/// workbook of the 1904 system built from its parts by
/// [`parts_of_1904_worksheet`], after `change_parts` has changed them; 43963
/// is 13 May 2024 in the system of 1904 and 12 May 2020 in that of 1900.
fn read_of_1904_date(
    change_parts: impl FnOnce(&mut Vec<(String, String)>),
) -> Result<Sheet, ReadError> {
    let mut parts = parts_of_1904_worksheet(
        r#"<sheetData><row r="1"><c r="A1" s="1"><v>43963</v></c></row></sheetData>"#,
        &["dd/mm/yyyy"],
    );
    change_parts(&mut parts);
    let files: Vec<(&str, &str)> = parts
        .iter()
        .map(|(name, xml)| (name.as_str(), xml.as_str()))
        .collect();
    read_first_sheet(&stored_zip(&files), MAX_SHEET_CELLS)
}

/// The XML of the part `part_name` of `parts`, or `None` when there is no
/// such part.
fn xml_of<'parts>(
    parts: &'parts mut [(String, String)],
    part_name: &str,
) -> Option<&'parts mut String> {
    parts
        .iter_mut()
        .find(|(name, _)| name == part_name)
        .map(|(_, xml)| xml)
}

/// The element `extLst` Excel 365 writes at the end of a workbook, with an
/// element `workbookPr` of the namespace `x15` whose attributes are
/// `x15_attributes`.
fn ext_lst_with_x15_workbook_pr(x15_attributes: &str) -> String {
    format!(
        r#"<extLst><ext uri="{{140A7094-0E35-4892-8432-C4D2E57EDEB5}}" xmlns:x15="http://schemas.microsoft.com/office/spreadsheetml/2010/11/main"><x15:workbookPr {x15_attributes}/></ext></extLst></workbook>"#
    )
}

#[test]
fn the_workbook_pr_of_the_root_gives_the_date_system_and_not_the_one_in_ext_lst() {
    let read = read_of_1904_date(|parts| {
        let workbook = xml_of(parts, "xl/workbook.xml").unwrap();
        *workbook = workbook.replace(
            "</workbook>",
            &ext_lst_with_x15_workbook_pr(r#"chartTrackingRefBase="1""#),
        );
    });

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Text("2024-05-13".to_owned())]
    );
}

#[test]
fn a_date1904_only_on_the_workbook_pr_in_ext_lst_leaves_the_system_of_1900() {
    let read = read_of_1904_date(|parts| {
        let workbook = xml_of(parts, "xl/workbook.xml").unwrap();
        *workbook = workbook
            .replace(r#"<workbookPr date1904="1"/>"#, "<workbookPr/>")
            .replace(
                "</workbook>",
                &ext_lst_with_x15_workbook_pr(r#"date1904="1""#),
            );
    });

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Text("2020-05-12".to_owned())]
    );
}

#[test]
fn a_date1904_of_true_is_the_system_of_1904() {
    let read = read_of_1904_date(|parts| {
        let workbook = xml_of(parts, "xl/workbook.xml").unwrap();
        *workbook = workbook.replace(r#"date1904="1""#, r#"date1904="true""#);
    });

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Text("2024-05-13".to_owned())]
    );
}

// A file Excel saves as "Strict Open XML" has other namespaces for its
// parts and for the types of its relationships, those of
// http://purl.oclc.org/ooxml/; the parts of the package, `_rels/.rels`, keep
// theirs.
#[test]
fn a_strict_workbook_of_the_1904_system_gives_its_dates_in_that_system() {
    let read = read_of_1904_date(|parts| {
        for (_, xml) in parts.iter_mut() {
            *xml = xml
                .replace(
                    "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
                    "http://purl.oclc.org/ooxml/spreadsheetml/main",
                )
                .replace(
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
                    "http://purl.oclc.org/ooxml/officeDocument/relationships",
                );
        }
        assert!(
            xml_of(parts, "xl/workbook.xml")
                .unwrap()
                .contains("purl.oclc.org/ooxml/spreadsheetml")
        );
    });

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Text("2024-05-13".to_owned())]
    );
}

// calamine finds a part by its name ignoring case, with `\` in the names of
// the zip read as `/`, and so does xlsx_rs: a workbook it did not find
// would be given the system of 1900 while calamine read its sheet.
#[test]
fn a_workbook_whose_parts_are_named_in_capitals_and_with_backslashes_is_found() {
    let read = read_of_1904_date(|parts| {
        for (name, _) in parts.iter_mut() {
            *name = name.to_ascii_uppercase().replace('/', "\\");
        }
        assert!(xml_of(parts, "XL\\WORKBOOK.XML").is_some());
    });

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Text("2024-05-13".to_owned())]
    );
}

#[test]
fn a_workbook_in_the_folder_its_relationship_gives_is_found_there() {
    let read = read_of_1904_date(|parts| {
        for (name, _) in parts.iter_mut() {
            if let Some(name_in_folder) = name.strip_prefix("xl/") {
                *name = format!("book/{name_in_folder}");
            }
        }
        let package_relationships = xml_of(parts, "_rels/.rels").unwrap();
        *package_relationships = package_relationships.replace(
            r#"Target="xl/workbook.xml""#,
            r#"Target="/book/workbook.xml""#,
        );
    });

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Text("2024-05-13".to_owned())]
    );
}

/// The relationship of `_rels/.rels` of type `officeDocument` whose
/// target is `target`, with the attributes `type_attributes` in the place
/// of its `Type`.
fn office_document_relationship(id: &str, type_attributes: &str, target: &str) -> String {
    format!(r#"<Relationship Id="{id}" {type_attributes} Target="{target}"/>"#)
}

/// The attribute `Type` of a relationship of type `officeDocument`.
const OFFICE_DOCUMENT_TYPE: &str =
    r#"Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument""#;

/// The date of [`read_of_1904_date`] with a copy of its workbook, of the
/// system of 1900, in the folder `data/`, and `_rels/.rels` holding
/// `relationships` inside its element `Relationships` and `after_end` after
/// its end: the date is 2024-05-13 when the workbook of `xl/` is taken,
/// and 2020-05-12 when the copy is.
fn read_of_1904_date_with_package_relationships(
    relationships: &str,
    after_end: &str,
) -> Result<Sheet, ReadError> {
    read_of_1904_date(|parts| {
        let copies: Vec<(String, String)> = parts
            .iter()
            .filter_map(|(name, xml)| {
                let name_in_folder = name.strip_prefix("xl/")?;
                Some((
                    format!("data/{name_in_folder}"),
                    xml.replace(r#"<workbookPr date1904="1"/>"#, "<workbookPr/>"),
                ))
            })
            .collect();
        parts.extend(copies);
        for (name, xml) in parts.iter_mut() {
            if name == "_rels/.rels" {
                let head = xml
                    .split_once("<Relationship ")
                    .map_or("", |(head, _)| head)
                    .to_owned();
                assert!(head.ends_with('>'), "no relationship in {xml}");
                *xml = format!("{head}{relationships}</Relationships>{after_end}");
            }
        }
    })
}

#[test]
fn the_last_office_document_relationship_gives_the_workbook() {
    let read = read_of_1904_date_with_package_relationships(
        &[
            office_document_relationship("rId1", OFFICE_DOCUMENT_TYPE, "data/workbook.xml"),
            office_document_relationship("rId2", OFFICE_DOCUMENT_TYPE, "xl/workbook.xml"),
        ]
        .concat(),
        "",
    );

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Text("2024-05-13".to_owned())]
    );
}

// calamine's macro get_attrs! stops once it has found as many attributes
// as it looks for, a name written twice counted twice: this relationship
// has the type "x" and no target.
#[test]
fn a_relationship_with_its_type_written_twice_is_not_the_workbook() {
    let read = read_of_1904_date_with_package_relationships(
        &[
            office_document_relationship("rId1", OFFICE_DOCUMENT_TYPE, "xl/workbook.xml"),
            office_document_relationship(
                "rId2",
                &format!(r#"{OFFICE_DOCUMENT_TYPE} Type="x""#),
                "data/workbook.xml",
            ),
        ]
        .concat(),
        "",
    );

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Text("2024-05-13".to_owned())]
    );
}

#[test]
fn a_relationship_after_the_end_of_relationships_is_not_read() {
    let read = read_of_1904_date_with_package_relationships(
        &office_document_relationship("rId1", OFFICE_DOCUMENT_TYPE, "xl/workbook.xml"),
        &office_document_relationship("rId2", OFFICE_DOCUMENT_TYPE, "data/workbook.xml"),
    );

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Text("2024-05-13".to_owned())]
    );
}

#[test]
fn a_file_whose_package_relationships_name_no_workbook_is_unreadable() {
    let other_type = r#"Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties""#;
    for relationships in [
        String::new(),
        office_document_relationship("rId1", other_type, "xl/workbook.xml"),
        format!(r#"<Relationship Id="rId1" {OFFICE_DOCUMENT_TYPE}/>"#),
    ] {
        let read = read_of_1904_date_with_package_relationships(&relationships, "");

        assert_eq!(
            read,
            Err(ReadError::Unreadable(
                "the file names no workbook".to_owned()
            )),
            "{relationships}"
        );
    }
}

#[test]
fn a_file_with_no_package_relationships_is_unreadable() {
    let read = read_of_1904_date(|parts| parts.retain(|(name, _)| name != "_rels/.rels"));

    assert_eq!(
        read,
        Err(ReadError::Unreadable(
            "the file names no workbook".to_owned()
        ))
    );
}

/// `xml` with spaces after its declaration, `<?xml ... ?>`, up to
/// `num_bytes` bytes, which XML reads as it read `xml`; `None` when `xml`
/// has no declaration or more bytes than that.
fn padded(xml: &str, num_bytes: usize) -> Option<String> {
    let declaration_end = xml.find("?>")?.checked_add(2)?;
    let (declaration, rest) = xml.split_at(declaration_end);
    let num_spaces = num_bytes.checked_sub(xml.len())?;
    Some(format!("{declaration}{}{rest}", " ".repeat(num_spaces)))
}

/// The parts calamine reads whole when it opens a file, besides the table
/// of texts: the package relationships, which give the folder of the
/// workbook; the workbook; its relationships; and the styles.
const SETTINGS_PARTS: [&str; 4] = [
    "_rels/.rels",
    "xl/workbook.xml",
    "xl/_rels/workbook.xml.rels",
    "xl/styles.xml",
];

/// The sheet of one cell, A1, the number 7, in a workbook of the 1904
/// system whose part `part_name` is padded to `num_bytes` bytes;
/// `None` when there is no such part, or it has more bytes than that.
fn read_with_part_of(part_name: &str, num_bytes: usize) -> Option<Result<Sheet, ReadError>> {
    let mut parts = parts_of_1904_worksheet(
        r#"<sheetData><row r="1"><c r="A1"><v>7</v></c></row></sheetData>"#,
        &[],
    );
    let xml = xml_of(&mut parts, part_name)?;
    *xml = padded(xml, num_bytes)?;
    Some(read_first_sheet(&xlsx_of_parts(&parts), MAX_SHEET_CELLS))
}

#[test]
fn a_part_calamine_reads_whole_past_50_000_000_bytes_is_unreadable() {
    for part_name in SETTINGS_PARTS {
        let read = read_with_part_of(part_name, 50_000_001).unwrap();

        assert_eq!(
            read,
            Err(ReadError::Unreadable(
                "a part of the file is too large".to_owned()
            )),
            "{part_name}"
        );
    }
}

#[test]
fn a_part_calamine_reads_whole_of_50_000_000_bytes_is_read() {
    for part_name in SETTINGS_PARTS {
        let read = read_with_part_of(part_name, 50_000_000).unwrap();

        assert_eq!(read.unwrap().cells, [SheetCell::Number(7.0)], "{part_name}");
    }
}

/// The sheet of one cell, A1, the number 7, in a workbook of the 1904
/// system with a copy of its part `part_name` renamed `copy_name`, padded
/// to `num_bytes` bytes; `None` when there is no such part, or it has
/// more bytes than that.
fn read_with_copy_of_part(
    part_name: &str,
    copy_name: &str,
    num_bytes: usize,
) -> Option<Result<Sheet, ReadError>> {
    let mut parts = parts_of_1904_worksheet(
        r#"<sheetData><row r="1"><c r="A1"><v>7</v></c></row></sheetData>"#,
        &[],
    );
    let xml = xml_of(&mut parts, part_name)?.clone();
    parts.push((copy_name.to_owned(), padded(&xml, num_bytes)?));
    Some(read_first_sheet(&xlsx_of_parts(&parts), MAX_SHEET_CELLS))
}

// calamine reads the parts of the folder the package relationships give,
// which xlsx_rs does not look for: every part whose name ends as one of
// them is held to their bound, `\` read as `/` and ignoring case.
#[test]
fn a_part_named_as_one_calamine_reads_whole_in_another_folder_is_held_to_the_same_bound() {
    for (part_name, copy_name) in [
        ("xl/workbook.xml", "data/workbook.xml"),
        ("xl/_rels/workbook.xml.rels", "data/_rels/workbook.xml.rels"),
        ("xl/styles.xml", "data/styles.xml"),
        ("xl/styles.xml", "DATA\\STYLES.XML"),
        ("xl/workbook.xml", "otherworkbook.xml"),
        ("_rels/.rels", "_RELS\\.RELS"),
    ] {
        let past_bound = read_with_copy_of_part(part_name, copy_name, 50_000_001).unwrap();
        let at_bound = read_with_copy_of_part(part_name, copy_name, 50_000_000).unwrap();

        assert_eq!(
            past_bound,
            Err(ReadError::Unreadable(
                "a part of the file is too large".to_owned()
            )),
            "{copy_name}"
        );
        assert_eq!(
            at_bound.unwrap().cells,
            [SheetCell::Number(7.0)],
            "{copy_name}"
        );
    }
}

/// The sheet whose A1 and B1 are the texts 0 and 1 of a table of texts,
/// `xl/sharedStrings.xml` with `shared_strings_xml`, in the parts of
/// [`parts_of_worksheet`].
fn parts_with_texts(shared_strings_xml: String) -> Vec<(String, String)> {
    let mut parts = parts_of_worksheet(
        r#"<sheetData><row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c></row></sheetData>"#,
    );
    parts.push(("xl/sharedStrings.xml".to_owned(), shared_strings_xml));
    parts
}

/// The sheet of [`parts_with_texts`], its table of texts `"id"` and
/// `"pop"` with the attributes `sst_attributes`.
fn read_with_texts(sst_attributes: &str) -> Result<Sheet, ReadError> {
    let parts = parts_with_texts(shared_strings(sst_attributes, &["id", "pop"]));
    read_first_sheet(&xlsx_of_parts(&parts), MAX_SHEET_CELLS)
}

/// The cells of the sheet of [`read_with_texts`].
fn the_two_texts() -> Vec<SheetCell> {
    vec![
        SheetCell::Text("id".to_owned()),
        SheetCell::Text("pop".to_owned()),
    ]
}

#[test]
fn a_table_of_texts_past_the_bound_of_its_bytes_is_too_much_text() {
    let shared_strings_xml = shared_strings(r#"uniqueCount="2""#, &["id", "pop"]);
    let num_bytes = u64::try_from(shared_strings_xml.len()).unwrap();
    let bytes = xlsx_of_parts(&parts_with_texts(shared_strings_xml));

    let past_bound = read_first_sheet_within_text_table_bytes(
        &bytes,
        MAX_SHEET_CELLS,
        num_bytes.checked_sub(1).unwrap(),
    );
    let at_bound = read_first_sheet_within_text_table_bytes(&bytes, MAX_SHEET_CELLS, num_bytes);

    assert_eq!(
        past_bound,
        Err(ReadError::Unreadable("too much text".to_owned()))
    );
    assert_eq!(at_bound.unwrap().cells, the_two_texts());
}

/// A table of texts of `num_texts` texts, the first two `id` and `pop` and
/// the others empty, `<si/>`, with no `uniqueCount`.
fn shared_strings_of_num_texts(num_texts: usize) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><si><t>id</t></si><si><t>pop</t></si>{}</sst>"#,
        "<si/>".repeat(num_texts.saturating_sub(2))
    )
}

#[test]
#[ignore = "takes 12 s in cargo test, past 10 s; run before each release"]
fn a_table_of_more_than_10_000_000_texts_is_unreadable() {
    let bytes = xlsx_of_parts(&parts_with_texts(shared_strings_of_num_texts(10_000_001)));

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable("too many texts".to_owned()))
    );
}

#[test]
#[ignore = "takes 19 s in cargo test, past 10 s; run before each release"]
fn a_table_of_10_000_000_texts_is_read() {
    let bytes = xlsx_of_parts(&parts_with_texts(shared_strings_of_num_texts(10_000_000)));

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(read.unwrap().cells, the_two_texts());
}

// calamine reserves room for as many texts as the first uniqueCount of the
// first sst says, before it reads the first text, and whatever the texts
// that follow: 400,000,000 trapped the package.
#[test]
fn a_table_of_texts_whose_unique_count_passes_10_000_000_is_unreadable() {
    for unique_count in [
        "10000001",
        "400000000",
        "00010000001",
        "4294967296",
        "123456789012345678901234567890",
    ] {
        let read = read_with_texts(&format!(r#"uniqueCount="{unique_count}""#));

        assert_eq!(
            read,
            Err(ReadError::Unreadable("too many texts".to_owned())),
            "uniqueCount {unique_count}"
        );
    }
}

#[test]
fn a_table_of_texts_cut_short_after_a_unique_count_past_10_000_000_is_unreadable() {
    let shared_strings_xml = shared_strings(r#"uniqueCount="400000000""#, &["id", "pop"]);
    let cut_short = shared_strings_xml.replace("</sst>", "");

    let read = read_first_sheet(
        &xlsx_of_parts(&parts_with_texts(cut_short)),
        MAX_SHEET_CELLS,
    );

    assert_eq!(
        read,
        Err(ReadError::Unreadable("too many texts".to_owned()))
    );
}

// Every uniqueCount of every sst is read, and every si counted, in any part
// whose name ends in sharedStrings.xml: more than calamine can hold.
#[test]
fn a_unique_count_past_10_000_000_anywhere_in_a_table_of_texts_is_unreadable() {
    let two_texts = shared_strings(r#"uniqueCount="2""#, &["id", "pop"]);
    for shared_strings_xml in [
        two_texts.replace("</sst>", r#"</sst><sst uniqueCount="400000000"/>"#),
        two_texts.replace(
            r#"uniqueCount="2""#,
            r#"uniqueCount="2" uniqueCount="400000000""#,
        ),
        two_texts.replace("<si>", r#"<x:sst uniqueCount="400000000"/><si>"#),
    ] {
        let read = read_first_sheet(
            &xlsx_of_parts(&parts_with_texts(shared_strings_xml.clone())),
            MAX_SHEET_CELLS,
        );

        assert_eq!(
            read,
            Err(ReadError::Unreadable("too many texts".to_owned())),
            "{shared_strings_xml}"
        );
    }
}

#[test]
fn a_table_of_texts_in_another_folder_is_counted() {
    let mut parts = parts_with_texts(shared_strings(r#"uniqueCount="2""#, &["id", "pop"]));
    parts.push((
        "data/sharedStrings.xml".to_owned(),
        shared_strings(r#"uniqueCount="400000000""#, &[]),
    ));

    let read = read_first_sheet(&xlsx_of_parts(&parts), MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable("too many texts".to_owned()))
    );
}

// calamine reserves room for 10,000,000 texts, 240 MB natively, and reads
// the two there are.
#[test]
fn a_table_of_texts_whose_unique_count_is_10_000_000_is_read() {
    let read = read_with_texts(r#"uniqueCount="10000000""#);

    assert_eq!(read.unwrap().cells, the_two_texts());
}

// A uniqueCount larger than the texts is a file calamine reads, and the
// room it reserves is within the bound; one with a sign is not a number to
// calamine, which reserves nothing for it.
#[test]
fn a_table_of_texts_whose_unique_count_is_within_10_000_000_missing_or_not_a_number_is_read() {
    for sst_attributes in [
        r#"count="5" uniqueCount="2""#,
        r#"uniqueCount="1""#,
        r#"uniqueCount="1000""#,
        r#"uniqueCount="0010000000""#,
        "",
        r#"uniqueCount="many""#,
        r#"uniqueCount="-3""#,
        r#"uniqueCount="+400000000""#,
        r#"uniqueCount="""#,
        r#"uniquecount="400000000""#,
    ] {
        let read = read_with_texts(sst_attributes);

        assert_eq!(read.unwrap().cells, the_two_texts(), "{sst_attributes}");
    }
}

#[test]
fn a_table_of_texts_in_another_folder_is_held_to_the_bound_of_its_bytes() {
    let shared_strings_xml = shared_strings(r#"uniqueCount="2""#, &["id", "pop"]);
    let num_bytes = u64::try_from(shared_strings_xml.len()).unwrap();
    let mut parts = parts_with_texts(shared_strings(r#"uniqueCount="2""#, &["id", "pop"]));
    parts.push((
        "data/sharedStrings.xml".to_owned(),
        format!("{shared_strings_xml} "),
    ));
    let bytes = xlsx_of_parts(&parts);

    let read = read_first_sheet_within_text_table_bytes(&bytes, MAX_SHEET_CELLS, num_bytes);

    assert_eq!(read, Err(ReadError::Unreadable("too much text".to_owned())));
}

// calamine finds the table of texts by its name ignoring case, and so does
// xlsx_rs: a table it did not find would reach calamine unchecked.
#[test]
fn a_table_of_texts_named_in_capitals_is_checked() {
    let read_of_unique_count = |unique_count: &str| {
        let mut parts = parts_with_texts(shared_strings(
            &format!(r#"uniqueCount="{unique_count}""#),
            &["id", "pop"],
        ));
        for (name, _) in &mut parts {
            if name == "xl/sharedStrings.xml" {
                *name = "XL/SHAREDSTRINGS.XML".to_owned();
            }
        }
        read_first_sheet(&xlsx_of_parts(&parts), MAX_SHEET_CELLS)
    };

    assert_eq!(read_of_unique_count("2").unwrap().cells, the_two_texts());
    assert_eq!(
        read_of_unique_count("400000000"),
        Err(ReadError::Unreadable("too many texts".to_owned()))
    );
}

/// The sheet of one cell, A1, the number 7, with `num_merged_ranges`
/// merged ranges of two cells each outside its rectangle, D1:E1, D2:E2
/// and so on, in a part named `xl/worksheets/data.xml`, which the
/// relationship of the sheet names as `relationship_target`, read with a
/// limit of 4 cells.
fn read_with_merged_ranges(
    num_merged_ranges: u32,
    relationship_target: &str,
) -> Result<Sheet, ReadError> {
    let merge_cells: String = (1..=num_merged_ranges)
        .map(|row| format!(r#"<mergeCell ref="D{row}:E{row}"/>"#))
        .collect();
    let mut parts = parts_of_worksheet(&format!(
        r#"<sheetData><row r="1"><c r="A1"><v>7</v></c></row></sheetData><mergeCells count="{num_merged_ranges}">{merge_cells}</mergeCells>"#
    ));
    for (name, xml) in &mut parts {
        if name == "xl/worksheets/sheet1.xml" {
            *name = "xl/worksheets/data.xml".to_owned();
        }
        if name == "xl/_rels/workbook.xml.rels" {
            *xml = xml.replace(
                r#"Target="worksheets/sheet1.xml""#,
                &format!(r#"Target="{relationship_target}""#),
            );
        }
    }
    read_first_sheet(&xlsx_of_parts(&parts), 4)
}

// The part of the sheet is found by the relationship the workbook gives
// it, relative to the folder of the workbook or from the root of the zip,
// and not by its name.
#[test]
fn a_sheet_with_more_merged_ranges_than_the_limit_of_cells_is_unreadable() {
    for relationship_target in ["worksheets/data.xml", "/xl/worksheets/data.xml"] {
        let read = read_with_merged_ranges(5, relationship_target);

        assert_eq!(
            read,
            Err(ReadError::Unreadable("too many merged ranges".to_owned())),
            "{relationship_target}"
        );
    }
}

#[test]
fn a_sheet_with_as_many_merged_ranges_as_the_limit_of_cells_is_read() {
    for relationship_target in ["worksheets/data.xml", "/xl/worksheets/data.xml"] {
        let read = read_with_merged_ranges(4, relationship_target);

        assert_eq!(
            read.unwrap().cells,
            [SheetCell::Number(7.0)],
            "{relationship_target}"
        );
    }
}

/// A part of a zip, a name and its bytes, given as `head`, then `middle`
/// written `num_middles` times, then `tail`, so that a part of hundreds of
/// MB is deflated without being held whole.
struct RepeatedPart<'part> {
    name: &'part str,
    head: &'part str,
    middle: &'part str,
    num_middles: u64,
    tail: &'part str,
}

/// A zip of `parts`, each deflated, the compression Excel uses.
fn deflated_zip(parts: &[RepeatedPart<'_>]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .large_file(true);
    for part in parts {
        writer.start_file(part.name, options)?;
        writer.write_all(part.head.as_bytes())?;
        // The middle written 4,096 times at once, so that the writer is not
        // called once for each of millions of elements.
        let middles_of_a_chunk = part.middle.repeat(4_096);
        let num_chunks = part.num_middles / 4_096;
        for _ in 0..num_chunks {
            writer.write_all(middles_of_a_chunk.as_bytes())?;
        }
        for _ in 0..part.num_middles % 4_096 {
            writer.write_all(part.middle.as_bytes())?;
        }
        writer.write_all(part.tail.as_bytes())?;
    }
    Ok(writer.finish()?.into_inner())
}

/// The parts of [`parts_of_worksheet`] with the sheet of one cell, A1, the
/// number 7, as the [`RepeatedPart`]s of [`deflated_zip`], none repeated.
fn parts_of_one_number() -> Vec<(String, String)> {
    parts_of_worksheet(r#"<sheetData><row r="1"><c r="A1"><v>7</v></c></row></sheetData>"#)
}

/// `parts` as [`RepeatedPart`]s that repeat nothing.
fn unrepeated(parts: &[(String, String)]) -> Vec<RepeatedPart<'_>> {
    parts
        .iter()
        .map(|(name, xml)| RepeatedPart {
            name,
            head: xml,
            middle: "",
            num_middles: 0,
            tail: "",
        })
        .collect()
}

/// A form feed, the byte `0C`, which calamine's reader of attributes takes
/// for a space before the name of an attribute, and quick-xml's for part
/// of the name.
const FORM_FEED: &str = "\x0C";

// The three files of the review of 28 September 2026 that trapped the
// package, each with a form feed before the name of an attribute, which
// calamine read and xlsx_rs, reading the attributes with quick-xml, did not.
#[test]
fn a_unique_count_after_a_form_feed_is_read_as_calamine_reads_it() {
    let mut parts = parts_with_texts(shared_strings(
        &format!(r#"count="2" {FORM_FEED}uniqueCount="400000000""#),
        &["id", "pop"],
    ));
    parts.sort_by_key(|(name, _)| name == "xl/sharedStrings.xml");
    let bytes = deflated_zip(&unrepeated(&parts)).unwrap();

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable("too many texts".to_owned()))
    );
}

#[test]
fn a_type_of_the_package_relationships_after_a_form_feed_is_read_as_calamine_reads_it() {
    let mut parts = parts_with_texts(shared_strings(
        r#"count="2" uniqueCount="400000000""#,
        &["id", "pop"],
    ));
    let package_relationships = xml_of(&mut parts, "_rels/.rels").unwrap();
    *package_relationships =
        package_relationships.replace(r#" Type="#, &format!(r#" {FORM_FEED}Type="#));
    assert!(package_relationships.contains(FORM_FEED));
    let bytes = deflated_zip(&unrepeated(&parts)).unwrap();

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable("too many texts".to_owned()))
    );
}

/// The xlsx of [`parts_of_one_number`] whose sheet has a form feed before
/// the attribute `name` in the workbook, and `num_merged_ranges` merged
/// ranges, each `D1:E1`, outside its rectangle, deflated.
fn xlsx_of_form_feed_name(num_merged_ranges: u64) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut parts = parts_of_one_number();
    let workbook = xml_of(&mut parts, "xl/workbook.xml").ok_or("no workbook")?;
    *workbook = workbook.replace(r#" name="#, &format!(r#" {FORM_FEED}name="#));
    let sheet = xml_of(&mut parts, "xl/worksheets/sheet1.xml").ok_or("no sheet")?;
    let (sheet_head, sheet_tail) = sheet
        .split_once("</worksheet>")
        .ok_or("no end of the sheet")?;
    let sheet_head = format!("{sheet_head}<mergeCells>");
    let sheet_tail = format!("</mergeCells></worksheet>{sheet_tail}");
    let mut repeated_parts = unrepeated(&parts);
    let merged_part = repeated_parts
        .iter_mut()
        .find(|part| part.name == "xl/worksheets/sheet1.xml")
        .ok_or("no sheet")?;
    merged_part.head = &sheet_head;
    merged_part.middle = r#"<mergeCell ref="D1:E1"/>"#;
    merged_part.num_middles = num_merged_ranges;
    merged_part.tail = &sheet_tail;
    deflated_zip(&repeated_parts)
}

#[test]
fn a_name_of_the_sheet_after_a_form_feed_is_read_as_calamine_reads_it() {
    let bytes = xlsx_of_form_feed_name(50).unwrap();

    let read = read_first_sheet(&bytes, 10);

    assert_eq!(
        read,
        Err(ReadError::Unreadable("too many merged ranges".to_owned()))
    );
}

// The file of the review held 36,000,000 merged ranges, which calamine
// held in 583 MB natively.
#[test]
#[ignore = "takes 15 s in cargo test, past 10 s; run before each release"]
fn a_name_of_the_sheet_after_a_form_feed_with_36_000_000_merged_ranges_is_unreadable() {
    let bytes = xlsx_of_form_feed_name(36_000_000).unwrap();

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable("too many merged ranges".to_owned()))
    );
}
