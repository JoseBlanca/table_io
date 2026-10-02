//! The guess of the type of a column of an import, "The type of a column"
//! of `docs/specs/values.md`: the narrowest of the four types that holds
//! every value of the column, and the values made of that type. It is not
//! public: an application asks for a type by converting a column to it.

use std::borrow::Cow;

use crate::types::{ColumnValues, boolean_text, integer_of_float};
use crate::value::{DecimalMark, float_text, parse_boolean, parse_float, parse_integer};

/// A value of a column of an import that is not missing, before the type
/// of its column is guessed: a text, or a number or a boolean cell of an
/// xlsx. A text borrows from the text of its file where the module of
/// the format made it no `String`, and becomes one only in a text column.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CellValue<'text> {
    /// A text, its spaces at the ends removed.
    Text(Cow<'text, str>),
    /// The number of a number cell of an xlsx, finite.
    Number(f64),
    /// The value of a boolean cell of an xlsx.
    Boolean(bool),
}

/// The values of a column of an import, None for a missing one, as the
/// narrowest of the four types that holds every value, read with
/// `decimal` ("The type of a column" of `docs/specs/values.md`).
///
/// The column is integer when every value is a whole number or a number
/// cell whose value is whole and from −2^63 to 2^63 − 1; else float when
/// every value is a number with `decimal` or a number cell; else boolean
/// when every value is a boolean or a boolean cell; else text, and text
/// too when every value is missing. In a text column a number cell is
/// written as JavaScript writes it, with `decimal`, and a boolean cell as
/// `TRUE` or `FALSE`. The guess depends on the values and not on their
/// order.
pub(crate) fn guessed_column(
    values: Vec<Option<CellValue<'_>>>,
    decimal: DecimalMark,
) -> ColumnValues {
    if values.iter().all(Option::is_none) {
        return ColumnValues::Text(vec![None; values.len()]);
    }
    if let Some(integers) = each_of_type(&values, integer_of_cell_value) {
        return ColumnValues::Integer(integers);
    }
    if let Some(floats) = each_of_type(&values, |cell_value| {
        float_of_cell_value(cell_value, decimal)
    }) {
        return ColumnValues::Float(floats);
    }
    if let Some(booleans) = each_of_type(&values, boolean_of_cell_value) {
        return ColumnValues::Boolean(booleans);
    }
    ColumnValues::Text(
        values
            .into_iter()
            .map(|cell_value| cell_value.map(|present| text_of_cell_value(present, decimal)))
            .collect(),
    )
}

/// Each value read by `read`, a missing one left missing, or None when
/// `read` gives None for one value.
fn each_of_type<To>(
    values: &[Option<CellValue<'_>>],
    read: impl Fn(&CellValue<'_>) -> Option<To>,
) -> Option<Vec<Option<To>>> {
    values
        .iter()
        .map(|cell_value| match cell_value {
            None => Some(None),
            Some(present) => read(present).map(Some),
        })
        .collect()
}

/// The integer a value is: a text that is a whole number, or a number
/// cell whose value is whole and from −2^63 to 2^63 − 1.
fn integer_of_cell_value(cell_value: &CellValue<'_>) -> Option<i64> {
    match cell_value {
        CellValue::Text(cell_text) => parse_integer(cell_text),
        CellValue::Number(number) => integer_of_float(*number),
        CellValue::Boolean(_) => None,
    }
}

/// The float a value is: a text that is a number with `decimal`, or a
/// number cell that is finite.
fn float_of_cell_value(cell_value: &CellValue<'_>, decimal: DecimalMark) -> Option<f64> {
    match cell_value {
        CellValue::Text(cell_text) => parse_float(cell_text, decimal),
        CellValue::Number(number) => number.is_finite().then_some(*number),
        CellValue::Boolean(_) => None,
    }
}

/// The boolean a value is: a text that is a boolean, or a boolean cell.
fn boolean_of_cell_value(cell_value: &CellValue<'_>) -> Option<bool> {
    match cell_value {
        CellValue::Text(cell_text) => parse_boolean(cell_text),
        CellValue::Boolean(is_true) => Some(*is_true),
        CellValue::Number(_) => None,
    }
}

/// The text of a value in a text column: a text as it is, a number cell as
/// JavaScript writes it with `decimal`, a boolean cell `TRUE` or `FALSE`.
fn text_of_cell_value(cell_value: CellValue<'_>, decimal: DecimalMark) -> String {
    match cell_value {
        CellValue::Text(cell_text) => cell_text.into_owned(),
        CellValue::Number(number) => float_text(number, decimal),
        CellValue::Boolean(is_true) => boolean_text(is_true),
    }
}
