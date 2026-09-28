//! Every cell of a merged range with the value of its first cell, within
//! the rectangle of the values ("Merged cells" of `docs/specs/read.md`).

mod hand_written;

use rust_xlsxwriter::{Format, Workbook};
use xlsx_rs::{ReadError, Sheet, SheetCell, read_first_sheet};

use crate::hand_written::{text_cell, xlsx_of_worksheet};

/// `MAX_SHEET_CELLS` of popnei_web, the limit its light worker gives.
const MAX_SHEET_CELLS: u32 = 2_000_000;

/// A text cell of `cell_text`.
fn text(cell_text: &str) -> SheetCell {
    SheetCell::Text(cell_text.to_owned())
}

#[test]
fn a_population_merged_over_rows_2_to_4_is_the_population_of_the_three() {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "Individuo").unwrap();
    worksheet.write_string(0, 1, "Población").unwrap();
    for (row, individual) in (1u32..).zip(["ind1", "ind2", "ind3", "ind4"]) {
        worksheet.write_string(row, 0, individual).unwrap();
    }
    worksheet
        .merge_range(1, 1, 3, 1, "Andalucía", &Format::new())
        .unwrap();
    worksheet.write_string(4, 1, "Murcia").unwrap();
    let bytes = workbook.save_to_buffer().unwrap();

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!(
        sheet.cells,
        vec![
            text("Individuo"),
            text("Población"),
            text("ind1"),
            text("Andalucía"),
            text("ind2"),
            text("Andalucía"),
            text("ind3"),
            text("Andalucía"),
            text("ind4"),
            text("Murcia"),
        ]
    );
}

#[test]
fn a_name_merged_over_two_columns_of_the_header_is_the_name_of_both() {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "Individuo").unwrap();
    worksheet
        .merge_range(0, 1, 0, 2, "Origen", &Format::new())
        .unwrap();
    worksheet.write_string(1, 0, "ind1").unwrap();
    worksheet.write_string(1, 1, "Andalucía").unwrap();
    worksheet.write_string(1, 2, "Sevilla").unwrap();
    let bytes = workbook.save_to_buffer().unwrap();

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!(
        sheet.cells,
        vec![
            text("Individuo"),
            text("Origen"),
            text("Origen"),
            text("ind1"),
            text("Andalucía"),
            text("Sevilla"),
        ]
    );
}

#[test]
fn a_merged_range_reaching_past_the_values_fills_only_the_rectangle() {
    // The name is merged over B2 to E3, and the values reach no further
    // than row 4 and column C, so D and E are not in the rectangle and the
    // rectangle does not grow to hold them.
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(1, 0, "Individuo").unwrap();
    worksheet
        .merge_range(1, 1, 2, 4, "Origen", &Format::new())
        .unwrap();
    worksheet.write_string(3, 0, "ind1").unwrap();
    worksheet.write_string(3, 2, "Sevilla").unwrap();
    let bytes = workbook.save_to_buffer().unwrap();

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!(
        sheet,
        Sheet {
            name: "Sheet1".to_owned(),
            first_row: 2,
            first_column: 1,
            num_rows: 3,
            num_columns: 3,
            cells: vec![
                text("Individuo"),
                text("Origen"),
                text("Origen"),
                SheetCell::Empty,
                text("Origen"),
                text("Origen"),
                text("ind1"),
                SheetCell::Empty,
                text("Sevilla"),
            ],
        }
    );
}

/// The `<sheetData>` of a header and three individuals from A1, with the
/// population `Andalucía` at B2 and nothing else in column B: the cells of
/// the tests of ranges rust_xlsxwriter does not write.
fn individuals_with_a_population_at_b2() -> String {
    [
        format!(
            r#"<row r="1">{}{}</row>"#,
            text_cell("A1", "Individuo"),
            text_cell("B1", "Población")
        ),
        format!(
            r#"<row r="2">{}{}</row>"#,
            text_cell("A2", "ind1"),
            text_cell("B2", "Andalucía")
        ),
        format!(r#"<row r="3">{}</row>"#, text_cell("A3", "ind2")),
        format!(r#"<row r="4">{}</row>"#, text_cell("A4", "ind3")),
    ]
    .concat()
}

#[test]
fn a_range_written_from_its_last_cell_b4_b2_is_the_range_b2_b4() {
    // Excel writes a range from its first cell; this file writes it from
    // its last, which calamine gives as it is, start B4 and end B2.
    let bytes = xlsx_of_worksheet(&format!(
        r#"<sheetData>{}</sheetData><mergeCells count="1"><mergeCell ref="B4:B2"/></mergeCells>"#,
        individuals_with_a_population_at_b2()
    ));

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!(
        sheet.cells,
        vec![
            text("Individuo"),
            text("Población"),
            text("ind1"),
            text("Andalucía"),
            text("ind2"),
            text("Andalucía"),
            text("ind3"),
            text("Andalucía"),
        ]
    );
}

#[test]
fn two_merged_ranges_that_overlap_make_the_file_unreadable() {
    // B2:B3 and B3:B4 share B3, whose value would depend on which of the
    // two the file writes first.
    let bytes = xlsx_of_worksheet(&format!(
        r#"<sheetData>{}</sheetData><mergeCells count="2"><mergeCell ref="B2:B3"/><mergeCell ref="B3:B4"/></mergeCells>"#,
        individuals_with_a_population_at_b2()
    ));

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable(
            "overlapping merged ranges".to_owned()
        ))
    );
}

#[test]
fn the_values_the_file_holds_in_a_merged_ranges_other_cells_give_way_to_its_first() {
    // B2:B5 is merged, and the file holds Murcia at B3 and Sevilla at B5,
    // as LibreOffice can keep them hidden; Sevilla still makes the
    // rectangle reach row 5.
    let rows = [
        format!(
            r#"<row r="1">{}{}</row>"#,
            text_cell("A1", "Individuo"),
            text_cell("B1", "Población")
        ),
        format!(
            r#"<row r="2">{}{}</row>"#,
            text_cell("A2", "ind1"),
            text_cell("B2", "Andalucía")
        ),
        format!(
            r#"<row r="3">{}{}</row>"#,
            text_cell("A3", "ind2"),
            text_cell("B3", "Murcia")
        ),
        format!(r#"<row r="4">{}</row>"#, text_cell("A4", "ind3")),
        format!(r#"<row r="5">{}</row>"#, text_cell("B5", "Sevilla")),
    ]
    .concat();
    let bytes = xlsx_of_worksheet(&format!(
        r#"<sheetData>{rows}</sheetData><mergeCells count="1"><mergeCell ref="B2:B5"/></mergeCells>"#
    ));

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!((sheet.num_rows, sheet.num_columns), (5, 2));
    assert_eq!(
        sheet.cells,
        vec![
            text("Individuo"),
            text("Población"),
            text("ind1"),
            text("Andalucía"),
            text("ind2"),
            text("Andalucía"),
            text("ind3"),
            text("Andalucía"),
            SheetCell::Empty,
            text("Andalucía"),
        ]
    );
}
