//! The files refused by their first bytes, those calamine cannot read, and
//! the workbook with no visible worksheet, the cell with an error calamine
//! does not know, and the sheet too large ("The refusals" of
//! `docs/specs/read.md`, points 1 to 6).

mod hand_written;

use rust_xlsxwriter::{Chart, ChartType, Formula, Workbook};
use xlsx_rs::{ReadError, Refusal, SheetCell, read_first_sheet};

use crate::hand_written::{text_cell, xlsx_of_worksheet};

/// `MAX_SHEET_CELLS` of popnei_web, the limit its light worker gives.
const MAX_SHEET_CELLS: u32 = 2_000_000;

/// The eight bytes every compound file of the old Office starts with.
const COMPOUND_FILE_MARK: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

#[test]
fn a_csv_is_refused_as_not_an_xlsx() {
    let read = read_first_sheet(b"id,pop\n", MAX_SHEET_CELLS);

    assert_eq!(read, Err(ReadError::Refused(Refusal::NotXlsx)));
}

#[test]
fn an_empty_file_is_refused_as_not_an_xlsx() {
    let read = read_first_sheet(b"", MAX_SHEET_CELLS);

    assert_eq!(read, Err(ReadError::Refused(Refusal::NotXlsx)));
}

#[test]
fn an_empty_zip_is_refused_as_not_an_xlsx() {
    // The 22 bytes of the end record of a zip with no file, which starts
    // with PK and the bytes 5 and 6, not 3 and 4.
    let mut bytes = vec![b'P', b'K', 5, 6];
    bytes.resize(22, 0);

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(read, Err(ReadError::Refused(Refusal::NotXlsx)));
}

#[test]
fn the_first_500_bytes_of_an_xlsx_are_unreadable_with_calamines_message() {
    let mut workbook = Workbook::new();
    workbook.add_worksheet().write_string(0, 0, "id").unwrap();
    let bytes = workbook.save_to_buffer().unwrap();
    assert!(bytes.len() > 500);

    let read = read_first_sheet(&bytes[..500], MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable(
            "Zip error: invalid Zip archive: Could not find EOCD".to_owned()
        ))
    );
}

#[test]
fn a_compound_file_with_no_encrypted_package_is_old_excel() {
    let mut bytes = COMPOUND_FILE_MARK.to_vec();
    bytes.resize(512, 0);

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(read, Err(ReadError::Refused(Refusal::OldExcel)));
}

/// A compound file as an xlsx saved with a password is, as far as xlsx_rs
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

#[test]
fn a_compound_file_with_an_encrypted_package_is_encrypted() {
    let read = read_first_sheet(&encrypted_compound_file(), MAX_SHEET_CELLS);

    assert_eq!(read, Err(ReadError::Refused(Refusal::Encrypted)));
}

#[test]
fn a_compound_file_with_an_encrypted_package_cut_short_is_encrypted() {
    // calamine's reader of compound files panicked on such files cut short,
    // which in the wasm is a trap; 632 is the byte just after the name.
    let bytes = encrypted_compound_file();
    for length in [632, 700, 1000, 1023] {
        let read = read_first_sheet(&bytes[..length], MAX_SHEET_CELLS);

        assert_eq!(
            read,
            Err(ReadError::Refused(Refusal::Encrypted)),
            "cut at {length} bytes"
        );
    }
}

#[test]
fn a_compound_file_cut_before_the_name_is_old_excel() {
    let bytes = encrypted_compound_file();
    for length in [8, 512, 631] {
        let read = read_first_sheet(&bytes[..length], MAX_SHEET_CELLS);

        assert_eq!(
            read,
            Err(ReadError::Refused(Refusal::OldExcel)),
            "cut at {length} bytes"
        );
    }
}

#[test]
fn a_workbook_whose_only_worksheet_is_hidden_is_unreadable() {
    // rust_xlsxwriter shows the active sheet whatever it was told, so the
    // one sheet shown is a chart sheet, which is not a worksheet.
    let mut workbook = Workbook::new();
    let lists = workbook.add_worksheet().set_name("Listas").unwrap();
    lists.write_number(0, 0, 1.0).unwrap();
    lists.set_hidden(true);
    let mut chart = Chart::new(ChartType::Column);
    chart.add_series().set_values("Listas!$A$1:$A$1");
    workbook
        .add_chartsheet()
        .insert_chart(0, 0, &chart)
        .unwrap()
        .set_active(true);
    let bytes = workbook.save_to_buffer().unwrap();

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable("no visible worksheet".to_owned()))
    );
}

#[test]
fn a_formula_saved_with_an_error_calamine_does_not_know_is_refused_with_its_text() {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "id").unwrap();
    worksheet
        .write_formula(1, 0, Formula::new("=A1").set_result("#GETTING_DATA"))
        .unwrap();
    let bytes = workbook.save_to_buffer().unwrap();

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Refused(Refusal::CellError {
            error: "#GETTING_DATA".to_owned()
        }))
    );
}

/// The last column of Excel, XFD, counted from 0 as rust_xlsxwriter counts
/// the columns.
const LAST_COLUMN: u16 = 16_383;

#[test]
fn a_value_at_a1_and_one_at_xfd200_are_a_sheet_too_large_of_200_rows_and_16384_columns() {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "id").unwrap();
    worksheet.write_string(199, LAST_COLUMN, "a note").unwrap();
    let bytes = workbook.save_to_buffer().unwrap();

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Refused(Refusal::SheetTooLarge {
            sheet: "Sheet1".to_owned(),
            first_row: 1,
            first_column: 1,
            num_rows: 200,
            num_columns: 16_384,
        }))
    );
}

#[test]
fn a_value_at_xfd1_over_column_a_down_to_row_200_is_too_large_at_row_123() {
    // 122 rows of 16,384 columns are 1,998,848 cells, and 123 rows are
    // 2,015,232, the first rectangle above 2,000,000.
    //
    // A150 holds a formula saved with #GETTING_DATA, an error calamine
    // refuses the sheet at. A read that went on past row 123, or that
    // checked the limit only once the whole sheet was read, meets it and
    // gives CellError; a read stopped at row 123 never reaches it.
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet().set_name("Individuos").unwrap();
    worksheet.write_string(0, LAST_COLUMN, "a note").unwrap();
    for row in 0..200 {
        if row == 149 {
            worksheet
                .write_formula(row, 0, Formula::new("=A1").set_result("#GETTING_DATA"))
                .unwrap();
        } else {
            worksheet.write_string(row, 0, "ind").unwrap();
        }
    }
    let bytes = workbook.save_to_buffer().unwrap();

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Refused(Refusal::SheetTooLarge {
            sheet: "Individuos".to_owned(),
            first_row: 1,
            first_column: 1,
            num_rows: 123,
            num_columns: 16_384,
        }))
    );
}

#[test]
fn a_rectangle_of_as_many_cells_as_the_limit_is_read_and_one_more_is_refused() {
    // Two rows of three columns from B2: 6 cells.
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    for row in 1..3 {
        for column in 1..4 {
            worksheet.write_number(row, column, 1.0).unwrap();
        }
    }
    let bytes = workbook.save_to_buffer().unwrap();

    let read_at_the_limit = read_first_sheet(&bytes, 6);
    let read_past_the_limit = read_first_sheet(&bytes, 5);

    assert_eq!(read_at_the_limit.map(|sheet| sheet.cells.len()), Ok(6));
    assert_eq!(
        read_past_the_limit,
        Err(ReadError::Refused(Refusal::SheetTooLarge {
            sheet: "Sheet1".to_owned(),
            first_row: 2,
            first_column: 2,
            num_rows: 2,
            num_columns: 3,
        }))
    );
}

/// An xlsx whose row 1 writes the cell A1 once for each of `cell_texts`,
/// which rust_xlsxwriter does not do: it keeps the last value written.
fn a1_written_as_each_of(cell_texts: &[&str]) -> Vec<u8> {
    let cells: String = cell_texts
        .iter()
        .map(|cell_text| text_cell("A1", cell_text))
        .collect();
    xlsx_of_worksheet(&format!(
        r#"<sheetData><row r="1">{cells}</row></sheetData>"#
    ))
}

#[test]
fn a_cell_written_3_times_with_a_limit_of_2_cells_is_unreadable() {
    let bytes = a1_written_as_each_of(&["ind1", "ind2", "ind3"]);

    let read = read_first_sheet(&bytes, 2);

    assert_eq!(
        read,
        Err(ReadError::Unreadable(
            "cells written more than once".to_owned()
        ))
    );
}

#[test]
fn a_cell_written_twice_with_a_limit_of_2_cells_takes_the_last_value() {
    let bytes = a1_written_as_each_of(&["ind1", "ind2"]);

    let read = read_first_sheet(&bytes, 2);

    assert_eq!(
        read.map(|sheet| sheet.cells),
        Ok(vec![SheetCell::Text("ind2".to_owned())])
    );
}
