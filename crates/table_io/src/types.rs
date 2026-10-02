//! The four types of a column, its values, and the conversion of a column
//! to another type ("The four types" and "The conversion of a column" of
//! `docs/specs/values.md`).

use crate::value::{DecimalMark, float_text, parse_boolean, parse_float, parse_integer};

/// The four types a column of a table can have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColumnType {
    /// A whole number from −2^63 to 2^63 − 1, Arrow's `Int64`.
    Integer,
    /// A number of 64 bits, Arrow's `Float64`.
    Float,
    /// True or false, Arrow's `Boolean`.
    Boolean,
    /// A text of Unicode, Arrow's `Utf8`.
    Text,
}

/// The values of a column, one for each row of the table, None for a
/// missing one.
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnValues {
    /// The values of an integer column.
    Integer(Vec<Option<i64>>),
    /// The values of a float column, finite in what an import gives.
    Float(Vec<Option<f64>>),
    /// The values of a boolean column.
    Boolean(Vec<Option<bool>>),
    /// The values of a text column.
    Text(Vec<Option<String>>),
}

impl ColumnValues {
    /// The type of the column.
    pub fn column_type(&self) -> ColumnType {
        match self {
            Self::Integer(_) => ColumnType::Integer,
            Self::Float(_) => ColumnType::Float,
            Self::Boolean(_) => ColumnType::Boolean,
            Self::Text(_) => ColumnType::Text,
        }
    }

    /// The number of rows.
    pub fn len(&self) -> usize {
        match self {
            Self::Integer(integers) => integers.len(),
            Self::Float(floats) => floats.len(),
            Self::Boolean(booleans) => booleans.len(),
            Self::Text(texts) => texts.len(),
        }
    }

    /// Whether the column has no row, which clippy asks for beside len.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// What a conversion says when one value or more does not convert.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionFailure {
    /// How many values do not convert, 1 or more.
    pub num_failed: u64,
    /// The row of the first, among the rows of the table, from 1; a first
    /// value past row 4,294,967,295, which no import gives, is named as
    /// row 4,294,967,295.
    pub first_row: u32,
    /// Its text, as "The text of a value" writes it.
    pub first_text: String,
}

/// The values converted to `to`, read with `decimal`, or how many do not
/// convert.
///
/// Each value is read by the rules of a value, a text with `decimal`, and
/// a value written as text takes `decimal` for its point. An integer
/// becomes the nearest float, 2^53 + 1 the float 2^53; a float becomes an
/// integer when it is whole and from −2^63 to 2^63 − 1; a boolean is never
/// a number, nor a number a boolean; and every value converts to text. A
/// missing value stays missing and never fails, so that a column of
/// missing values converts to any type.
///
/// # Errors
///
/// A [`ConversionFailure`] when one value or more does not convert, with
/// how many do not and the first of them, its row counted from 1 among
/// the rows of the table and its text.
pub fn convert_column(
    values: &ColumnValues,
    to: ColumnType,
    decimal: DecimalMark,
) -> Result<ColumnValues, ConversionFailure> {
    match (values, to) {
        (ColumnValues::Integer(_), ColumnType::Integer)
        | (ColumnValues::Float(_), ColumnType::Float)
        | (ColumnValues::Boolean(_), ColumnType::Boolean)
        | (ColumnValues::Text(_), ColumnType::Text) => Ok(values.clone()),
        (ColumnValues::Integer(integers), ColumnType::Float) => {
            Ok(ColumnValues::Float(each_always(integers, |&integer| {
                nearest_float(integer)
            })))
        }
        (ColumnValues::Integer(integers), ColumnType::Boolean) => {
            each_converted(integers, |_| None, i64::to_string).map(ColumnValues::Boolean)
        }
        (ColumnValues::Integer(integers), ColumnType::Text) => {
            Ok(ColumnValues::Text(each_always(integers, i64::to_string)))
        }
        (ColumnValues::Float(floats), ColumnType::Integer) => each_converted(
            floats,
            |&float| integer_of_float(float),
            |&float| float_text(float, decimal),
        )
        .map(ColumnValues::Integer),
        (ColumnValues::Float(floats), ColumnType::Boolean) => {
            each_converted(floats, |_| None, |&float| float_text(float, decimal))
                .map(ColumnValues::Boolean)
        }
        (ColumnValues::Float(floats), ColumnType::Text) => {
            Ok(ColumnValues::Text(each_always(floats, |&float| {
                float_text(float, decimal)
            })))
        }
        (ColumnValues::Boolean(booleans), ColumnType::Integer) => {
            each_converted(booleans, |_| None, |&is_true| boolean_text(is_true))
                .map(ColumnValues::Integer)
        }
        (ColumnValues::Boolean(booleans), ColumnType::Float) => {
            each_converted(booleans, |_| None, |&is_true| boolean_text(is_true))
                .map(ColumnValues::Float)
        }
        (ColumnValues::Boolean(booleans), ColumnType::Text) => {
            Ok(ColumnValues::Text(each_always(booleans, |&is_true| {
                boolean_text(is_true)
            })))
        }
        (ColumnValues::Text(texts), ColumnType::Integer) => {
            each_converted(texts, |text| parse_integer(text), String::clone)
                .map(ColumnValues::Integer)
        }
        (ColumnValues::Text(texts), ColumnType::Float) => {
            each_converted(texts, |text| parse_float(text, decimal), String::clone)
                .map(ColumnValues::Float)
        }
        (ColumnValues::Text(texts), ColumnType::Boolean) => {
            each_converted(texts, |text| parse_boolean(text), String::clone)
                .map(ColumnValues::Boolean)
        }
    }
}

/// 2^63, the first whole number past the integers; a float holds it
/// exactly, as it does not hold 2^63 − 1, which Rust's `i64::MAX as f64`
/// rounds to 2^63.
const TWO_TO_THE_63: f64 = 9_223_372_036_854_775_808.0;

/// The float nearest to an integer, the even one of two as near.
fn nearest_float(integer: i64) -> f64 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "an integer past 2^53 becomes the nearest float, as the owner decided on 2 October 2026"
    )]
    let float = integer as f64;
    float
}

/// The integer a float is, when it is whole and from −2^63 to 2^63 − 1,
/// or None; never an integer made by a cast that saturates.
pub(crate) fn integer_of_float(float: f64) -> Option<i64> {
    let is_integer = float.is_finite()
        && float.fract() == 0.0
        && (-TWO_TO_THE_63..TWO_TO_THE_63).contains(&float);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a whole float from −2^63 to below 2^63 is an i64 exactly, as checked above"
    )]
    let integer = is_integer.then_some(float as i64);
    integer
}

/// The text of a boolean, `TRUE` or `FALSE`, as Excel shows it.
pub(crate) fn boolean_text(is_true: bool) -> String {
    if is_true { "TRUE" } else { "FALSE" }.to_owned()
}

/// Each value converted by a conversion that never fails, a missing one
/// left missing.
fn each_always<From, To>(
    values: &[Option<From>],
    convert: impl Fn(&From) -> To,
) -> Vec<Option<To>> {
    values
        .iter()
        .map(|value_from| value_from.as_ref().map(&convert))
        .collect()
}

/// Each value converted by `convert`, a missing one left missing, or,
/// when `convert` gives None for one value or more, how many do and the
/// first of them, written by `text_of`.
fn each_converted<From, To>(
    values: &[Option<From>],
    convert: impl Fn(&From) -> Option<To>,
    text_of: impl Fn(&From) -> String,
) -> Result<Vec<Option<To>>, ConversionFailure> {
    let mut converted = Vec::with_capacity(values.len());
    let mut first_failed: Option<(usize, &From)> = None;
    let mut num_failed: u64 = 0;
    for (index, value_from) in values.iter().enumerate() {
        match value_from {
            None => converted.push(None),
            Some(present) => match convert(present) {
                Some(value_to) => converted.push(Some(value_to)),
                None => {
                    // A Vec holds fewer than 2^64 values, so the count
                    // never saturates.
                    num_failed = num_failed.saturating_add(1);
                    if first_failed.is_none() {
                        first_failed = Some((index, present));
                    }
                }
            },
        }
    }
    match first_failed {
        None => Ok(converted),
        Some((index, present)) => Err(ConversionFailure {
            num_failed,
            first_row: row_of_index(index),
            first_text: text_of(present),
        }),
    }
}

/// The row of a value at `index` among the rows of the table, from 1, and
/// row 4,294,967,295 for one past it (`docs/specs/values.md`, "The
/// conversion of a column").
fn row_of_index(index: usize) -> u32 {
    u32::try_from(index)
        .ok()
        .and_then(|index| index.checked_add(1))
        .unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::row_of_index;

    #[test]
    fn the_first_value_is_row_1() {
        assert_eq!(row_of_index(0), 1);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn a_value_past_row_4294967295_is_named_as_row_4294967295() {
        assert_eq!(row_of_index(4_294_967_293), 4_294_967_294);
        assert_eq!(row_of_index(4_294_967_294), 4_294_967_295);
        assert_eq!(row_of_index(4_294_967_295), 4_294_967_295);
        assert_eq!(row_of_index(10_000_000_000), 4_294_967_295);
    }
}
