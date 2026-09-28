//! The parts of the zip xlsx_rs reads itself before calamine ("What xlsx_rs
//! reads before calamine" of `docs/specs/read.md`): every part read to its
//! end, so that a part whose checksum fails refuses the file, and the bytes
//! so read counted and bounded; and the date system, from the
//! `workbookPr` that is a direct child of the root of the workbook.

#[expect(
    dead_code,
    reason = "the workbooks here are of the 1904 system, and their texts are not written by hand"
)]
mod hand_written;

use std::error::Error;
use std::io::Cursor;
use std::ops::Range;

use rust_xlsxwriter::{Workbook, XlsxError};
use xlsx_rs::{
    ReadError, Sheet, SheetCell, read_first_sheet, read_first_sheet_within_unzipped_bytes,
};
use zip::ZipArchive;
use zip::result::ZipError;

use crate::hand_written::{parts_of_1904_worksheet, stored_zip};

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
