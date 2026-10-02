//! What each kind of value becomes, but the dates and the durations, over
//! files written in memory with rust_xlsxwriter ("Each cell" of
//! `docs/specs/read.md`).

#![cfg(feature = "xlsx")]

use std::error::Error;

use rust_xlsxwriter::{Format, Formula, Workbook, Worksheet};
use table_io::{SheetCell, read_first_sheet};

/// `MAX_SHEET_CELLS` of popnei_web, the limit its light worker gives.
const MAX_SHEET_CELLS: u32 = 2_000_000;

/// A text cell of `cell_text`.
fn text(cell_text: &str) -> SheetCell {
    SheetCell::Text(cell_text.to_owned())
}

/// The cells of the first sheet of a workbook whose one worksheet
/// `write_cells` writes; an error when the file is not written or not read.
fn cells_written_by(
    write_cells: impl FnOnce(&mut Worksheet),
) -> Result<Vec<SheetCell>, Box<dyn Error>> {
    let mut workbook = Workbook::new();
    write_cells(workbook.add_worksheet());
    let bytes = workbook.save_to_buffer()?;
    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS)
        .map_err(|read_error| format!("{read_error:?}"))?;
    Ok(sheet.cells)
}

#[test]
fn a_number_with_the_format_000_is_the_number_and_not_its_three_digits() {
    let cells = cells_written_by(|worksheet| {
        let three_digits = Format::new().set_num_format("000");
        worksheet
            .write_number_with_format(0, 0, 1.0, &three_digits)
            .unwrap();
    })
    .unwrap();

    assert_eq!(cells, vec![SheetCell::Number(1.0)]);
}

#[test]
fn a_number_with_the_format_of_a_percentage_is_the_number() {
    let cells = cells_written_by(|worksheet| {
        let percentage = Format::new().set_num_format("0 %");
        worksheet
            .write_number_with_format(0, 0, 0.5, &percentage)
            .unwrap();
    })
    .unwrap();

    assert_eq!(cells, vec![SheetCell::Number(0.5)]);
}

#[test]
fn a_formula_saved_with_a_number_is_the_number() {
    let cells = cells_written_by(|worksheet| {
        worksheet
            .write_formula(0, 0, Formula::new("=1+1").set_result("2"))
            .unwrap();
    })
    .unwrap();

    assert_eq!(cells, vec![SheetCell::Number(2.0)]);
}

#[test]
fn a_formula_saved_with_a_text_is_the_text() {
    let cells = cells_written_by(|worksheet| {
        worksheet
            .write_formula(0, 0, Formula::new(r#"="x""#).set_result("x"))
            .unwrap();
    })
    .unwrap();

    assert_eq!(cells, vec![text("x")]);
}

#[test]
fn a_formula_saved_with_true_is_the_boolean() {
    let cells = cells_written_by(|worksheet| {
        worksheet
            .write_formula(0, 0, Formula::new("=TRUE()").set_result("TRUE"))
            .unwrap();
    })
    .unwrap();

    assert_eq!(cells, vec![SheetCell::Bool(true)]);
}

#[test]
fn a_formula_saved_with_the_error_na_is_its_text() {
    let cells = cells_written_by(|worksheet| {
        worksheet
            .write_formula(0, 0, Formula::new("=NA()").set_result("#N/A"))
            .unwrap();
    })
    .unwrap();

    assert_eq!(cells, vec![text("#N/A")]);
}

#[test]
fn a_formula_saved_with_the_error_of_a_division_by_zero_is_its_text() {
    let cells = cells_written_by(|worksheet| {
        worksheet
            .write_formula(0, 0, Formula::new("=1/0").set_result("#DIV/0!"))
            .unwrap();
    })
    .unwrap();

    assert_eq!(cells, vec![text("#DIV/0!")]);
}

#[test]
fn the_other_five_errors_excel_writes_are_their_texts() {
    let errors = ["#NAME?", "#NULL!", "#NUM!", "#REF!", "#VALUE!"];
    let cells = cells_written_by(|worksheet| {
        for (column, error) in (0u16..).zip(errors) {
            worksheet
                .write_formula(0, column, Formula::new("=A2").set_result(error))
                .unwrap();
        }
    })
    .unwrap();

    assert_eq!(
        cells,
        vec![
            text("#NAME?"),
            text("#NULL!"),
            text("#NUM!"),
            text("#REF!"),
            text("#VALUE!"),
        ]
    );
}

#[test]
fn a_formula_saved_with_no_value_is_the_zero_rust_xlsxwriter_saves() {
    let cells = cells_written_by(|worksheet| {
        worksheet.write_formula(0, 0, Formula::new("=1+2")).unwrap();
    })
    .unwrap();

    assert_eq!(cells, vec![SheetCell::Number(0.0)]);
}

#[test]
fn a_formula_saved_with_a_number_that_is_not_finite_is_the_text_javascript_writes() {
    // rust_xlsxwriter writes a number that is not finite as the text NAN,
    // INF or -INF, but a formula saved with a value Rust parses as a number
    // is written as a number, so these three reach calamine as numbers.
    let cells = cells_written_by(|worksheet| {
        for (column, saved_value) in (0u16..).zip(["NaN", "inf", "-inf"]) {
            worksheet
                .write_formula(0, column, Formula::new("=A2").set_result(saved_value))
                .unwrap();
        }
    })
    .unwrap();

    assert_eq!(
        cells,
        vec![text("NaN"), text("Infinity"), text("-Infinity")]
    );
}

#[test]
fn a_tenth_plus_two_tenths_is_the_number_stored_and_not_rounded() {
    let cells = cells_written_by(|worksheet| {
        worksheet.write_number(0, 0, 0.1 + 0.2).unwrap();
    })
    .unwrap();

    assert_eq!(cells, vec![SheetCell::Number(0.30000000000000004)]);
}

#[test]
fn a_text_with_spaces_at_its_ends_keeps_them() {
    let cells = cells_written_by(|worksheet| {
        worksheet.write_string(0, 0, "  sp ").unwrap();
    })
    .unwrap();

    assert_eq!(cells, vec![text("  sp ")]);
}

#[test]
fn a_text_with_a_line_break_keeps_it() {
    let cells = cells_written_by(|worksheet| {
        worksheet.write_string(0, 0, "Andalucía\noriental").unwrap();
    })
    .unwrap();

    assert_eq!(cells, vec![text("Andalucía\noriental")]);
}

#[test]
fn a_text_of_several_fonts_is_its_parts_joined() {
    let cells = cells_written_by(|worksheet| {
        let bold = Format::new().set_bold();
        let plain = Format::default();
        worksheet
            .write_rich_string(0, 0, &[(&bold, "Pobla"), (&plain, "ción 🌱")])
            .unwrap();
    })
    .unwrap();

    assert_eq!(cells, vec![text("Población 🌱")]);
}
