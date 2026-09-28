//! The files refused by their first bytes, those calamine cannot read, and
//! the workbook with no visible worksheet ("The refusals" of
//! `docs/specs/read.md`, points 1 to 4).

use rust_xlsxwriter::{Chart, ChartType, Workbook};
use xlsx_rs::{ReadError, Refusal, read_first_sheet};

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
