//! The cells of a table to export, given to the writer of its format in
//! the order of the file, the header and then row by row, each from left
//! to right, with the refusals of `docs/specs/export.md` that every format
//! shares: the shape of the table, the empty and the repeated names of the
//! columns and of the individuals, a text that reads back as missing and a
//! float that is not finite. The writer refuses what its own format cannot
//! hold. It is behind no feature.

use std::collections::HashMap;

use crate::value::is_missing;
use crate::{CellPlace, Column, ColumnValues, ExportRefusal, NameColumn};

/// A value of a column other than the names', as the writer of a format
/// is given it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ExportValue<'table> {
    /// A missing value.
    Missing,
    /// An integer.
    Integer(i64),
    /// A float, finite.
    Float(f64),
    /// A boolean.
    Boolean(bool),
    /// A text, none that reads back as missing.
    Text(&'table str),
}

/// The writer of a format, given each cell in the order of the file.
pub(crate) trait CellWriter {
    /// Writes the name of a column of the header at `place`, the names'
    /// first, which may be empty when the table has other columns.
    ///
    /// # Errors
    ///
    /// The refusal of a name the format cannot hold.
    fn header_name(&mut self, name: &str, place: CellPlace) -> Result<(), ExportRefusal>;

    /// Writes the name of the individual of a row, not empty, at `place`,
    /// in the names' column.
    ///
    /// # Errors
    ///
    /// The refusal of a name the format cannot hold.
    fn individual_name(&mut self, name: &str, place: CellPlace) -> Result<(), ExportRefusal>;

    /// Writes a value of a column other than the names' at `place`.
    ///
    /// # Errors
    ///
    /// The refusal of a value the format cannot hold.
    fn value(&mut self, value: ExportValue<'_>, place: CellPlace) -> Result<(), ExportRefusal>;

    /// Ends the row, the header or that of an individual.
    fn row_end(&mut self);
}

/// The refusal of the shape of a table, or None: the first column, from
/// left to right, with another number of values than there are names,
/// [`ExportRefusal::WrongLength`], then a table with no individual,
/// [`ExportRefusal::NoIndividual`].
pub(crate) fn shape_refusal(names: &NameColumn, columns: &[Column]) -> Option<ExportRefusal> {
    let num_names = names.names.len();
    let wrong_length = columns
        .iter()
        .enumerate()
        .find(|(_, column)| column.values.len() != num_names)
        .map(|(index, column)| ExportRefusal::WrongLength {
            column: column_of_index(index),
            expected: count_of(num_names),
            found: count_of(column.values.len()),
        });
    if wrong_length.is_some() {
        return wrong_length;
    }
    names
        .names
        .is_empty()
        .then_some(ExportRefusal::NoIndividual)
}

/// Gives every cell of the table to `writer`, in the order of the file,
/// and stops at the first refusal, of the cell or of the writer.
///
/// # Errors
///
/// The first refusal in the order of the file: an empty name,
/// [`ExportRefusal::EmptyName`], or one that repeats a name before it,
/// [`ExportRefusal::DuplicateName`]; an empty name of an individual,
/// [`ExportRefusal::EmptyIndividual`], or one that repeats an individual
/// before it, [`ExportRefusal::DuplicateIndividual`]; a text value that is
/// a missing value, [`ExportRefusal::ReadsAsMissing`]; a float that is not
/// finite, [`ExportRefusal::NotFinite`]; or the writer's refusal of the
/// cell, which comes after these.
pub(crate) fn write_cells(
    names: &NameColumn,
    columns: &[Column],
    writer: &mut impl CellWriter,
) -> Result<(), ExportRefusal> {
    let header_names = std::iter::once(names.header.as_str())
        .chain(columns.iter().map(|column| column.name.as_str()));
    let mut first_column_of: HashMap<&str, u32> = HashMap::with_capacity(columns.len());
    for (index, name) in header_names.enumerate() {
        let column = number_from(index, 1);
        let may_be_empty = index == 0 && !columns.is_empty();
        if name.is_empty() && !may_be_empty {
            return Err(ExportRefusal::EmptyName { column });
        }
        if let Some(&first_column) = first_column_of.get(name) {
            return Err(ExportRefusal::DuplicateName {
                name: name.to_owned(),
                first_column,
                second_column: column,
            });
        }
        first_column_of.insert(name, column);
        writer.header_name(name, CellPlace { column, row: 0 })?;
    }
    writer.row_end();
    let mut first_row_of: HashMap<&str, u32> = HashMap::with_capacity(names.names.len());
    for (index, name) in names.names.iter().enumerate() {
        let row = number_from(index, 1);
        if name.is_empty() {
            return Err(ExportRefusal::EmptyIndividual { row });
        }
        if let Some(&first_row) = first_row_of.get(name.as_str()) {
            return Err(ExportRefusal::DuplicateIndividual {
                name: name.clone(),
                first_row,
                second_row: row,
            });
        }
        first_row_of.insert(name, row);
        writer.individual_name(name, CellPlace { column: 1, row })?;
        for (column_index, column) in columns.iter().enumerate() {
            let place = CellPlace {
                column: column_of_index(column_index),
                row,
            };
            let value = value_at(&column.values, index);
            match value {
                ExportValue::Text(text) if is_missing(text) => {
                    return Err(ExportRefusal::ReadsAsMissing { place });
                }
                ExportValue::Float(float) if !float.is_finite() => {
                    return Err(ExportRefusal::NotFinite { place });
                }
                ExportValue::Missing
                | ExportValue::Integer(_)
                | ExportValue::Float(_)
                | ExportValue::Boolean(_)
                | ExportValue::Text(_) => {}
            }
            writer.value(value, place)?;
        }
        writer.row_end();
    }
    Ok(())
}

/// The value of `values` at `index`, missing past its end, which
/// [`shape_refusal`] leaves no column with.
fn value_at(values: &ColumnValues, index: usize) -> ExportValue<'_> {
    let present = match values {
        ColumnValues::Integer(integers) => integers
            .get(index)
            .copied()
            .flatten()
            .map(ExportValue::Integer),
        ColumnValues::Float(floats) => floats.get(index).copied().flatten().map(ExportValue::Float),
        ColumnValues::Boolean(booleans) => booleans
            .get(index)
            .copied()
            .flatten()
            .map(ExportValue::Boolean),
        ColumnValues::Text(texts) => texts
            .get(index)
            .and_then(Option::as_deref)
            .map(ExportValue::Text),
    };
    present.unwrap_or(ExportValue::Missing)
}

/// The column of the file of the column at `index` of the other columns,
/// the first being column 2.
fn column_of_index(index: usize) -> u32 {
    number_from(index, 2)
}

/// The number of the thing at `index` when the first is numbered `first`,
/// 4,294,967,295 past it (`docs/specs/export.md`, "The table it takes").
fn number_from(index: usize, first: u32) -> u32 {
    u32::try_from(index)
        .ok()
        .and_then(|index| index.checked_add(first))
        .unwrap_or(u32::MAX)
}

/// A count of names or of values, 4,294,967,295 past it.
fn count_of(length: usize) -> u32 {
    u32::try_from(length).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::{count_of, number_from};

    #[test]
    fn the_first_column_is_numbered_from_the_number_given_and_a_count_is_the_length() {
        assert_eq!(number_from(0, 1), 1);
        assert_eq!(number_from(0, 2), 2);
        assert_eq!(number_from(3, 2), 5);
        assert_eq!(count_of(3), 3);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn a_number_or_a_count_past_4294967295_is_given_as_4294967295() {
        assert_eq!(number_from(4_294_967_293, 2), 4_294_967_295);
        assert_eq!(number_from(4_294_967_294, 2), 4_294_967_295);
        assert_eq!(number_from(4_294_967_295, 1), 4_294_967_295);
        assert_eq!(count_of(4_294_967_295), 4_294_967_295);
        assert_eq!(count_of(10_000_000_000), 4_294_967_295);
    }
}
