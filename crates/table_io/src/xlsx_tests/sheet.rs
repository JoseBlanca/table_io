//! Which sheet is read and the rectangle of its values, over files written
//! in memory with rust_xlsxwriter ("The sheet read" of `docs/specs/read.md`).

#![cfg(feature = "xlsx")]
#![allow(clippy::arithmetic_side_effects, reason = "small literals in tests")]

use crate::xlsx::{ReadError, Refusal, Sheet, SheetCell, read_first_sheet};
use rust_xlsxwriter::{Chart, ChartType, Format, Workbook, Worksheet, XlsxError};

/// `MAX_SHEET_CELLS` of popnei_web, the limit its light worker gives.
const MAX_SHEET_CELLS: u32 = 2_000_000;

/// A text cell of `cell_text`.
fn text(cell_text: &str) -> SheetCell {
    SheetCell::Text(cell_text.to_owned())
}

/// Writes a table of a header and two individuals from `first_row` and
/// `first_column`, counted from 0 as rust_xlsxwriter counts them, with a
/// text of accents and an emoji, numbers, booleans, and a blank cell with
/// a format and no value at the height of the second individual.
fn write_table(
    worksheet: &mut Worksheet,
    first_row: u32,
    first_column: u16,
) -> Result<(), XlsxError> {
    let header = ["Individuo", "Población", "Altura", "Afectado"];
    for (offset, name) in (0u16..).zip(header) {
        worksheet.write_string(first_row, first_column + offset, name)?;
    }
    let second_row = first_row + 1;
    worksheet.write_string(second_row, first_column, "ind1")?;
    worksheet.write_string(second_row, first_column + 1, "Andalucía 🌱")?;
    worksheet.write_number(second_row, first_column + 2, 1.75)?;
    worksheet.write_boolean(second_row, first_column + 3, true)?;
    let third_row = first_row + 2;
    worksheet.write_string(third_row, first_column, "ind2")?;
    worksheet.write_string(third_row, first_column + 1, "Murcia")?;
    worksheet.write_blank(third_row, first_column + 2, &Format::new().set_bold())?;
    worksheet.write_boolean(third_row, first_column + 3, false)?;
    Ok(())
}

/// The cells of the table of `write_table`, row after row.
fn table_cells() -> Vec<SheetCell> {
    vec![
        text("Individuo"),
        text("Población"),
        text("Altura"),
        text("Afectado"),
        text("ind1"),
        text("Andalucía 🌱"),
        SheetCell::Number(1.75),
        SheetCell::Bool(true),
        text("ind2"),
        text("Murcia"),
        SheetCell::Empty,
        SheetCell::Bool(false),
    ]
}

#[test]
fn a_table_at_a1_gives_its_cells_and_empty_for_the_blank_one() {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet().set_name("Individuos").unwrap();
    write_table(worksheet, 0, 0).unwrap();
    let bytes = workbook.save_to_buffer().unwrap();

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!(
        sheet,
        Sheet {
            name: "Individuos".to_owned(),
            first_row: 1,
            first_column: 1,
            num_rows: 3,
            num_columns: 4,
            cells: table_cells(),
        }
    );
}

#[test]
fn a_table_at_c3_starts_at_row_3_and_column_3_with_the_same_cells() {
    let mut workbook = Workbook::new();
    write_table(workbook.add_worksheet(), 2, 2).unwrap();
    let bytes = workbook.save_to_buffer().unwrap();

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!(
        sheet,
        Sheet {
            name: "Sheet1".to_owned(),
            first_row: 3,
            first_column: 3,
            num_rows: 3,
            num_columns: 4,
            cells: table_cells(),
        }
    );
}

#[test]
fn a_table_at_d2_starts_at_row_2_and_column_4() {
    // The row and the column of the first cell differ, so a rectangle whose
    // row and column were swapped does not pass.
    let mut workbook = Workbook::new();
    write_table(workbook.add_worksheet(), 1, 3).unwrap();
    let bytes = workbook.save_to_buffer().unwrap();

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!(
        (
            sheet.first_row,
            sheet.first_column,
            sheet.num_rows,
            sheet.num_columns
        ),
        (2, 4, 3, 4)
    );
    assert_eq!(sheet.cells, table_cells());
}

#[test]
fn a_row_with_its_second_cell_not_written_is_empty_there() {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "id").unwrap();
    worksheet.write_string(0, 1, "pop").unwrap();
    worksheet.write_string(0, 2, "height").unwrap();
    worksheet.write_string(1, 0, "ind1").unwrap();
    worksheet.write_number(1, 2, 1.8).unwrap();
    let bytes = workbook.save_to_buffer().unwrap();

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!((sheet.num_rows, sheet.num_columns), (2, 3));
    assert_eq!(
        sheet.cells,
        vec![
            text("id"),
            text("pop"),
            text("height"),
            text("ind1"),
            SheetCell::Empty,
            SheetCell::Number(1.8),
        ]
    );
}

#[test]
fn a_hidden_first_sheet_is_passed_over_for_the_second() {
    let mut workbook = Workbook::new();
    let lists = workbook.add_worksheet().set_name("Listas").unwrap();
    lists.write_string(0, 0, "a list of a form").unwrap();
    lists.set_hidden(true);
    let individuals = workbook.add_worksheet().set_name("Individuos").unwrap();
    individuals.write_string(0, 0, "id").unwrap();
    individuals.write_string(1, 0, "ind1").unwrap();
    // rust_xlsxwriter shows the active sheet, the first one unless told.
    individuals.set_active(true);
    let bytes = workbook.save_to_buffer().unwrap();

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!(sheet.name, "Individuos");
    assert_eq!(sheet.cells, vec![text("id"), text("ind1")]);
}

#[test]
fn a_very_hidden_first_sheet_is_passed_over_for_the_second() {
    // A sheet very hidden is one only a macro can show again.
    let mut workbook = Workbook::new();
    let lists = workbook.add_worksheet().set_name("Listas").unwrap();
    lists.write_string(0, 0, "a list of a form").unwrap();
    lists.set_very_hidden(true);
    let individuals = workbook.add_worksheet().set_name("Individuos").unwrap();
    individuals.write_string(0, 0, "id").unwrap();
    individuals.write_string(1, 0, "ind1").unwrap();
    individuals.set_active(true);
    let bytes = workbook.save_to_buffer().unwrap();

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!(sheet.name, "Individuos");
    assert_eq!(sheet.cells, vec![text("id"), text("ind1")]);
}

#[test]
fn a_hidden_row_and_a_hidden_column_are_read() {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    write_table(worksheet, 0, 0).unwrap();
    worksheet.set_row_hidden(1).unwrap();
    worksheet.set_column_hidden(1).unwrap();
    let bytes = workbook.save_to_buffer().unwrap();

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!((sheet.num_rows, sheet.num_columns), (3, 4));
    assert_eq!(sheet.cells, table_cells());
}

#[test]
fn a_blank_cell_with_a_format_outside_the_values_does_not_widen_the_rectangle() {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "id").unwrap();
    worksheet
        .write_blank(9, 4, &Format::new().set_bold())
        .unwrap();
    let bytes = workbook.save_to_buffer().unwrap();

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!(
        sheet,
        Sheet {
            name: "Sheet1".to_owned(),
            first_row: 1,
            first_column: 1,
            num_rows: 1,
            num_columns: 1,
            cells: vec![text("id")],
        }
    );
}

#[test]
fn values_at_b1_c1_and_a3_give_the_rectangle_from_a1_of_3_by_3() {
    // The first column of the rectangle is not that of its first row.
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 1, "pop").unwrap();
    worksheet.write_number(0, 2, 1.75).unwrap();
    worksheet.write_boolean(2, 0, true).unwrap();
    let bytes = workbook.save_to_buffer().unwrap();

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!(
        sheet,
        Sheet {
            name: "Sheet1".to_owned(),
            first_row: 1,
            first_column: 1,
            num_rows: 3,
            num_columns: 3,
            cells: vec![
                SheetCell::Empty,
                text("pop"),
                SheetCell::Number(1.75),
                SheetCell::Empty,
                SheetCell::Empty,
                SheetCell::Empty,
                SheetCell::Bool(true),
                SheetCell::Empty,
                SheetCell::Empty,
            ],
        }
    );
}

#[test]
fn a_chart_sheet_first_is_passed_over_for_the_worksheet_after_it() {
    let mut workbook = Workbook::new();
    let mut chart = Chart::new(ChartType::Column);
    chart.add_series().set_values("Individuos!$B$2:$B$3");
    workbook
        .add_chartsheet()
        .set_name("Gráfico")
        .unwrap()
        .insert_chart(0, 0, &chart)
        .unwrap();
    let individuals = workbook.add_worksheet().set_name("Individuos").unwrap();
    individuals.write_string(0, 0, "id").unwrap();
    individuals.write_number(0, 1, 7.0).unwrap();
    let bytes = workbook.save_to_buffer().unwrap();

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!(sheet.name, "Individuos");
    assert_eq!(sheet.cells, vec![text("id"), SheetCell::Number(7.0)]);
}

#[test]
fn a_first_sheet_with_no_value_is_refused_with_its_name() {
    let mut workbook = Workbook::new();
    workbook.add_worksheet().set_name("Notas").unwrap();
    let individuals = workbook.add_worksheet().set_name("Individuos").unwrap();
    individuals.write_string(0, 0, "id").unwrap();
    let bytes = workbook.save_to_buffer().unwrap();

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Refused(Refusal::EmptySheet {
            sheet: "Notas".to_owned()
        }))
    );
}

#[test]
fn a_first_sheet_holding_only_a_blank_cell_with_a_format_is_refused_with_its_name() {
    let mut workbook = Workbook::new();
    let notes = workbook.add_worksheet().set_name("Notas").unwrap();
    notes.write_blank(1, 1, &Format::new().set_bold()).unwrap();
    let individuals = workbook.add_worksheet().set_name("Individuos").unwrap();
    individuals.write_string(0, 0, "id").unwrap();
    let bytes = workbook.save_to_buffer().unwrap();

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Refused(Refusal::EmptySheet {
            sheet: "Notas".to_owned()
        }))
    );
}
