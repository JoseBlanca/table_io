//! The messages of the bounds of "What xlsx_rs reads before calamine" of
//! `docs/specs/read.md`, tested with files of a few KB through
//! `read_first_sheet_within`, which takes the bounds as an argument: a file
//! that reaches the real bounds unzips to hundreds of MB. The files are
//! written by the helper of `tests/hand_written/`, which the tests of
//! `tests/` share, taken here by its path.

#[expect(
    dead_code,
    reason = "the tests of the bounds use only the zip of the parts and the workbooks of one sheet"
)]
#[path = "../tests/hand_written/mod.rs"]
mod hand_written;

use std::io::Cursor;

use rust_xlsxwriter::{Workbook, XlsxError};
use zip::ZipArchive;
use zip::result::ZipError;

use crate::bounds_tests::hand_written::{parts_of_worksheet, shared_strings, xlsx_of_parts};
use crate::parts::PartBounds;
use crate::{PART_BOUNDS, ReadError, Sheet, SheetCell, read_first_sheet_within};

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

/// The bounds of `read_first_sheet`, with `change` made to them.
fn bounds_with(change: impl FnOnce(&mut PartBounds)) -> PartBounds {
    let mut bounds = PART_BOUNDS;
    change(&mut bounds);
    bounds
}

#[test]
fn a_file_that_unzips_to_more_bytes_than_the_bound_is_unreadable() {
    let bytes = xlsx_of_individuals().unwrap();
    assert!(unzipped_size_of(&bytes).unwrap() > 4_000);

    let read = read_first_sheet_within(
        &bytes,
        MAX_SHEET_CELLS,
        bounds_with(|bounds| bounds.max_unzipped_bytes = 4_000),
    );

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

    let bounded_read = read_first_sheet_within(
        &bytes,
        MAX_SHEET_CELLS,
        bounds_with(|bounds| bounds.max_unzipped_bytes = unzipped_size),
    );
    let one_byte_less = read_first_sheet_within(
        &bytes,
        MAX_SHEET_CELLS,
        bounds_with(|bounds| bounds.max_unzipped_bytes = unzipped_size - 1),
    );

    let sheet = bounded_read.unwrap();
    assert_eq!(sheet.cells[0], SheetCell::Text("id".to_owned()));
    assert_eq!(sheet.num_rows, 201);
    assert!(matches!(one_byte_less, Err(ReadError::Unreadable(_))));
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

/// The cells of the sheet of [`parts_with_texts`] with the texts `id` and
/// `pop`.
fn the_two_texts() -> Vec<SheetCell> {
    vec![
        SheetCell::Text("id".to_owned()),
        SheetCell::Text("pop".to_owned()),
    ]
}

/// What `read_first_sheet_within` gives for `parts`, zipped, within
/// `bounds`.
fn read_within(parts: &[(String, String)], bounds: PartBounds) -> Result<Sheet, ReadError> {
    read_first_sheet_within(&xlsx_of_parts(parts), MAX_SHEET_CELLS, bounds)
}

#[test]
fn a_table_of_texts_past_the_bound_of_its_bytes_is_too_much_text() {
    let shared_strings_xml = shared_strings(r#"uniqueCount="2""#, &["id", "pop"]);
    let num_bytes = u64::try_from(shared_strings_xml.len()).unwrap();
    let parts = parts_with_texts(shared_strings_xml);

    let past_bound = read_within(
        &parts,
        bounds_with(|bounds| bounds.max_text_table_bytes = num_bytes - 1),
    );
    let at_bound = read_within(
        &parts,
        bounds_with(|bounds| bounds.max_text_table_bytes = num_bytes),
    );

    assert_eq!(
        past_bound,
        Err(ReadError::Unreadable("too much text".to_owned()))
    );
    assert_eq!(at_bound.unwrap().cells, the_two_texts());
}

#[test]
fn a_table_of_texts_in_another_folder_is_held_to_the_bound_of_its_bytes() {
    let shared_strings_xml = shared_strings(r#"uniqueCount="2""#, &["id", "pop"]);
    let num_bytes = u64::try_from(shared_strings_xml.len()).unwrap();
    let mut parts = parts_with_texts(shared_strings_xml.clone());
    parts.push((
        "data/sharedStrings.xml".to_owned(),
        format!("{shared_strings_xml} "),
    ));

    let read = read_within(
        &parts,
        bounds_with(|bounds| bounds.max_text_table_bytes = num_bytes),
    );

    assert_eq!(read, Err(ReadError::Unreadable("too much text".to_owned())));
}
