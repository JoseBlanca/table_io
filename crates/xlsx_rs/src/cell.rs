//! The cell of one value of calamine ("Each cell" of `docs/specs/read.md`).

use calamine::DataRef;

use crate::SheetCell;

/// The cell of `calamine_value`: empty, text, number or boolean.
///
/// The empty cell, the texts, the numbers and the booleans are as the spec
/// gives them. The other values are provisional until the rest of "Each
/// cell" is built, and no test reaches them yet: a whole number as the
/// number, a date or a duration as the number Excel stores, an ISO 8601
/// text as it is, and an error as calamine writes it.
pub(crate) fn cell_of_value(calamine_value: &DataRef<'_>) -> SheetCell {
    match calamine_value {
        DataRef::Empty => SheetCell::Empty,
        DataRef::SharedString(calamine_text) => cell_of_text(calamine_text),
        DataRef::String(calamine_text) => cell_of_text(calamine_text),
        DataRef::Float(number) => cell_of_number(*number),
        DataRef::Bool(is_true) => SheetCell::Bool(*is_true),
        DataRef::Int(whole_number) => cell_of_number(*whole_number as f64),
        DataRef::DateTime(date_time) => cell_of_number(date_time.as_f64()),
        DataRef::DateTimeIso(calamine_text) | DataRef::DurationIso(calamine_text) => {
            cell_of_text(calamine_text)
        }
        DataRef::Error(calamine_error) => SheetCell::Text(calamine_error.to_string()),
    }
}

/// A text as it is, or an empty cell for a text with no character.
fn cell_of_text(cell_text: &str) -> SheetCell {
    if cell_text.is_empty() {
        SheetCell::Empty
    } else {
        SheetCell::Text(cell_text.to_owned())
    }
}

/// A finite number as it is, and one that is not finite as the text
/// JavaScript writes for it, since the number of a cell is finite.
fn cell_of_number(number: f64) -> SheetCell {
    if number.is_finite() {
        SheetCell::Number(number)
    } else if number.is_nan() {
        SheetCell::Text("NaN".to_owned())
    } else if number.is_sign_positive() {
        SheetCell::Text("Infinity".to_owned())
    } else {
        SheetCell::Text("-Infinity".to_owned())
    }
}
