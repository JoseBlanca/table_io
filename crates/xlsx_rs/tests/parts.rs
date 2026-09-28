//! The parts of the zip xlsx_rs reads itself before calamine ("What xlsx_rs
//! reads before calamine" of `docs/specs/read.md`): every part read to its
//! end, so that a part whose checksum fails refuses the file, and the bytes
//! so read counted and bounded.

use std::error::Error;
use std::io::Cursor;
use std::ops::Range;

use rust_xlsxwriter::{Workbook, XlsxError};
use xlsx_rs::{
    ReadError, Sheet, SheetCell, read_first_sheet, read_first_sheet_within_unzipped_bytes,
};
use zip::ZipArchive;
use zip::result::ZipError;

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
