//! The cell of one value of calamine ("Each cell" of `docs/specs/read.md`).

use calamine::{CellErrorType, DataRef};

use crate::SheetCell;

/// The cell of `calamine_value`, as "Each cell" gives it.
///
/// A date, a date with its time, a time alone and a duration are the
/// number Excel stores until work package 3 of `docs/plans/read.md` builds
/// them. calamine's reader of an xlsx never gives a whole number, `Int`,
/// nor a duration as ISO 8601 text, `DurationIso`; they are given all the
/// same, as [`cell_of_whole_number`] says and as their text.
pub(crate) fn cell_of_value(calamine_value: &DataRef<'_>) -> SheetCell {
    match calamine_value {
        DataRef::Empty => SheetCell::Empty,
        DataRef::SharedString(calamine_text) => cell_of_text(calamine_text),
        DataRef::String(calamine_text) => cell_of_text(calamine_text),
        DataRef::Float(number) => cell_of_number(*number),
        DataRef::Bool(is_true) => SheetCell::Bool(*is_true),
        DataRef::Int(whole_number) => cell_of_whole_number(*whole_number),
        DataRef::DateTime(date_time) => cell_of_number(date_time.as_f64()),
        DataRef::DateTimeIso(calamine_text) | DataRef::DurationIso(calamine_text) => {
            cell_of_text(calamine_text)
        }
        DataRef::Error(calamine_error) => SheetCell::Text(text_of_error(calamine_error).to_owned()),
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

/// The largest whole number every smaller one of which a float holds
/// exactly, 2^53.
const LARGEST_EXACT_WHOLE_NUMBER: u64 = 1 << 53;

/// A whole number as the number when a float holds it exactly, up to 2^53
/// either side of 0, and as its digits, a text, past that, so that no
/// cell is a number other than the one in the file.
fn cell_of_whole_number(whole_number: i64) -> SheetCell {
    if whole_number.unsigned_abs() <= LARGEST_EXACT_WHOLE_NUMBER {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a whole number up to 2^53 either side of 0 is a float exactly"
        )]
        let number = whole_number as f64;
        SheetCell::Number(number)
    } else {
        SheetCell::Text(whole_number.to_string())
    }
}

/// The text of an error as Excel writes it in English, in the file and in
/// the cell.
fn text_of_error(calamine_error: &CellErrorType) -> &'static str {
    match calamine_error {
        CellErrorType::Div0 => "#DIV/0!",
        CellErrorType::NA => "#N/A",
        CellErrorType::Name => "#NAME?",
        CellErrorType::Null => "#NULL!",
        CellErrorType::Num => "#NUM!",
        CellErrorType::Ref => "#REF!",
        CellErrorType::Value => "#VALUE!",
        // calamine writes it `#DATA!`; its reader of an xlsx refuses the
        // sheet at it, refusal 5, rather than give it.
        CellErrorType::GettingData => "#GETTING_DATA",
    }
}

#[cfg(test)]
mod tests {
    use calamine::{CellErrorType, DataRef};

    use crate::SheetCell;
    use crate::cell::cell_of_value;

    #[test]
    fn a_shared_text_with_no_character_is_an_empty_cell() {
        assert_eq!(cell_of_value(&DataRef::SharedString("")), SheetCell::Empty);
    }

    #[test]
    fn a_text_of_the_cell_with_no_character_is_an_empty_cell() {
        assert_eq!(
            cell_of_value(&DataRef::String(String::new())),
            SheetCell::Empty
        );
    }

    // calamine's reader of an xlsx gives every number as a float, and a
    // whole number only for other formats, so these reach no sheet.

    #[test]
    fn a_whole_number_a_float_holds_exactly_is_the_number() {
        assert_eq!(cell_of_value(&DataRef::Int(-7)), SheetCell::Number(-7.0));
        assert_eq!(
            cell_of_value(&DataRef::Int(9_007_199_254_740_992)),
            SheetCell::Number(9_007_199_254_740_992.0)
        );
    }

    #[test]
    fn a_whole_number_past_two_to_the_53_is_its_digits_as_text() {
        assert_eq!(
            cell_of_value(&DataRef::Int(9_007_199_254_740_993)),
            SheetCell::Text("9007199254740993".to_owned())
        );
        assert_eq!(
            cell_of_value(&DataRef::Int(i64::MIN)),
            SheetCell::Text("-9223372036854775808".to_owned())
        );
    }

    #[test]
    fn an_error_of_getting_data_is_the_text_excel_writes() {
        // calamine writes it #DATA!, and its reader of an xlsx refuses the
        // sheet at it rather than give it.
        assert_eq!(
            cell_of_value(&DataRef::Error(CellErrorType::GettingData)),
            SheetCell::Text("#GETTING_DATA".to_owned())
        );
    }
}
