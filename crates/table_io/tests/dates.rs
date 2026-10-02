//! The dates, the times and the durations, over files written in memory
//! with rust_xlsxwriter in Excel's date system of 1900, and by hand in the
//! system of 1904, which rust_xlsxwriter does not write ("Each cell" of
//! `docs/specs/read.md`).

#![cfg(feature = "xlsx")]

#[cfg(test)]
#[expect(
    dead_code,
    reason = "the dates are numbers, and no cell here is a text written by hand"
)]
mod hand_written;

use std::error::Error;

use rust_xlsxwriter::{Format, Workbook};
use table_io::{SheetCell, read_first_sheet};

use crate::hand_written::xlsx_of_1904_worksheet;

/// `MAX_SHEET_CELLS` of popnei_web, the limit its light worker gives.
const MAX_SHEET_CELLS: u32 = 2_000_000;

/// The one cell of a sheet whose A1 holds `number` with the format
/// `num_format`; an error when the file is not written or not read.
fn cell_of_number_with_format(number: f64, num_format: &str) -> Result<SheetCell, Box<dyn Error>> {
    let mut workbook = Workbook::new();
    let format = Format::new().set_num_format(num_format);
    workbook
        .add_worksheet()
        .write_number_with_format(0, 0, number, &format)?;
    let bytes = workbook.save_to_buffer()?;
    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS)
        .map_err(|read_error| format!("{read_error:?}"))?;
    match sheet.cells.as_slice() {
        [cell] => Ok(cell.clone()),
        cells => Err(format!("one cell expected, got {cells:?}").into()),
    }
}

/// A text cell of `cell_text`.
fn text(cell_text: &str) -> SheetCell {
    SheetCell::Text(cell_text.to_owned())
}

#[test]
fn a_date_with_no_time_is_the_date_alone() {
    assert_eq!(
        cell_of_number_with_format(45425.0, "dd/mm/yyyy").unwrap(),
        text("2024-05-13")
    );
}

#[test]
fn a_date_and_a_time_shown_with_the_time_is_the_date_and_the_time() {
    assert_eq!(
        cell_of_number_with_format(45425.5, "dd/mm/yyyy hh:mm:ss").unwrap(),
        text("2024-05-13 12:00:00")
    );
}

#[test]
fn a_date_and_a_time_shown_without_the_time_is_the_date_and_the_time() {
    assert_eq!(
        cell_of_number_with_format(45425.5, "dd/mm/yyyy").unwrap(),
        text("2024-05-13 12:00:00")
    );
}

#[test]
fn a_date_a_hundredth_of_a_millisecond_before_midnight_is_the_next_day() {
    assert_eq!(
        cell_of_number_with_format(45_425.999_999_999_9, "dd/mm/yyyy").unwrap(),
        text("2024-05-14")
    );
}

#[test]
fn a_date_and_a_time_with_milliseconds_gives_them() {
    // 12:00:00.250 of 13 May 2024: a quarter of a second past midday.
    assert_eq!(
        cell_of_number_with_format(45425.5 + 0.25 / 86_400.0, "dd/mm/yyyy hh:mm:ss.000").unwrap(),
        text("2024-05-13 12:00:00.250")
    );
}

#[test]
fn a_time_below_a_day_is_the_time_alone() {
    assert_eq!(
        cell_of_number_with_format(0.604_166_666, "hh:mm").unwrap(),
        text("14:30:00")
    );
}

#[test]
fn a_duration_of_a_day_and_a_half_has_its_hours_not_wrapped_at_24() {
    assert_eq!(
        cell_of_number_with_format(1.5, "[h]:mm:ss").unwrap(),
        text("36:00:00")
    );
}

#[test]
fn a_negative_duration_has_a_minus() {
    assert_eq!(
        cell_of_number_with_format(-0.5 / 24.0, "[h]:mm:ss").unwrap(),
        text("-0:30:00")
    );
}

#[test]
fn day_60_of_the_1900_system_is_excels_29_february_1900() {
    assert_eq!(
        cell_of_number_with_format(60.0, "dd/mm/yyyy").unwrap(),
        text("1900-02-29")
    );
}

#[test]
fn the_last_day_of_9999_is_the_date() {
    assert_eq!(
        cell_of_number_with_format(2_958_465.0, "dd/mm/yyyy").unwrap(),
        text("9999-12-31")
    );
}

#[test]
fn the_first_day_of_10000_is_the_number() {
    assert_eq!(
        cell_of_number_with_format(2_958_466.0, "dd/mm/yyyy").unwrap(),
        SheetCell::Number(2_958_466.0)
    );
}

#[test]
fn a_date_below_0_is_the_number() {
    assert_eq!(
        cell_of_number_with_format(-3.0, "dd/mm/yyyy").unwrap(),
        SheetCell::Number(-3.0)
    );
}

#[test]
fn a_workbook_of_the_1904_system_gives_its_dates_in_that_system() {
    // Styles 1 to 3 of the file are the three formats, in this order.
    let number_formats = ["dd/mm/yyyy hh:mm:ss", "dd/mm/yyyy", "hh:mm:ss"];
    let bytes = xlsx_of_1904_worksheet(
        r#"<sheetData><row r="1"><c r="A1" s="1"><v>43963.5</v></c></row><row r="2"><c r="A2" s="2"><v>2957003</v></c></row><row r="3"><c r="A3" s="2"><v>2957004</v></c></row><row r="4"><c r="A4" s="3"><v>0</v></c></row><row r="5"><c r="A5" s="2"><v>59</v></c></row></sheetData>"#,
        &number_formats,
    );
    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();
    assert_eq!(
        sheet.cells,
        [
            // 13 May 2024 at 12:00, 45425.5 in the 1900 system.
            text("2024-05-13 12:00:00"),
            text("9999-12-31"),
            SheetCell::Number(2_957_004.0),
            text("00:00:00"),
            // 1904 was a leap year: day 59 is 29 February.
            text("1904-02-29"),
        ]
    );
}

#[test]
fn a_quarter_of_a_day_shown_as_hours_and_minutes_is_six_oclock() {
    assert_eq!(
        cell_of_number_with_format(0.25, "hh:mm").unwrap(),
        text("06:00:00")
    );
}

#[test]
fn a_date_a_little_below_0_is_midnight() {
    assert_eq!(
        cell_of_number_with_format(-1e-10, "dd/mm/yyyy").unwrap(),
        text("00:00:00")
    );
}

#[test]
fn a_duration_a_little_below_0_is_no_time_and_no_minus() {
    assert_eq!(
        cell_of_number_with_format(-1e-10, "[h]:mm:ss").unwrap(),
        text("0:00:00")
    );
}

#[test]
fn a_time_that_rounds_to_a_whole_day_is_the_date_of_day_1() {
    // Excel shows 00:00:00; calamine gives no format, and 1 day is a date.
    assert_eq!(
        cell_of_number_with_format(0.999_999_999_99, "hh:mm:ss").unwrap(),
        text("1900-01-01")
    );
}
