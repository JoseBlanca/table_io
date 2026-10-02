//! The table of the rows of a file, by the rules both applications share,
//! "The cells of an xlsx, as the table takes them" and "The rows" of
//! `docs/specs/import.md`: the blank rows skipped, the header, the columns
//! with no name dropped or refused, the duplicate columns, the names of the
//! individuals, the missing cells, and each column typed by
//! `docs/specs/values.md`, with the refusals of the rows in the order of
//! "The refusals". It is behind no feature: the module of each format gives
//! it the rows of its file.

use std::collections::HashMap;

use crate::guess::{CellValue, guessed_column};
use crate::types::boolean_text;
use crate::value::{DecimalMark, float_text, is_missing};
use crate::{Column, Format, HowRead, ImportError, NameColumn, Refusal, Table};

/// A cell of a row, as the module of its format gives it.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum Cell {
    /// No value.
    #[default]
    Empty,
    /// A text, as the file holds it.
    Text(String),
    /// The number of a number cell of an xlsx, finite.
    Number(f64),
    /// The value of a boolean cell of an xlsx.
    Boolean(bool),
}

/// A row of a file, with its place.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Row {
    /// The line of a text file where the row starts, or the row of the
    /// sheet of an xlsx, from 1.
    pub(crate) place: u32,
    /// Its cells, the first in the column [`Rows::first_column`] and each
    /// next one in the column after; a row of a text file may have fewer
    /// or more cells than its header.
    pub(crate) cells: Vec<Cell>,
}

/// The format the rows come from, which sets the rules that are the
/// format's: the spaces at the ends of a cell, the errors of Excel, and
/// the decimal mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Origin {
    /// The rectangle of the sheet of an xlsx: every row as long as the
    /// rectangle, a text cell with its spaces and tabs at the ends, the
    /// seven errors of Excel missing values and refused in the header, and
    /// the numbers read with the point.
    Xlsx,
}

impl Origin {
    /// The format of the file, which a refusal is given with.
    fn format(self) -> Format {
        match self {
            Self::Xlsx => Format::Xlsx,
        }
    }

    /// The decimal mark the texts of the values are read with.
    fn decimal(self) -> DecimalMark {
        match self {
            Self::Xlsx => DecimalMark::Point,
        }
    }
}

/// The rows of a file, in its order, blank ones among them.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Rows {
    /// The format they come from.
    pub(crate) origin: Origin,
    /// The column of the first cell of every row: in an xlsx the first
    /// column of the rectangle of the sheet, column A being 1; in a text
    /// file 1.
    pub(crate) first_column: u32,
    /// The rows.
    pub(crate) rows: Vec<Row>,
}

/// The seven errors of Excel that calamine knows, as `docs/specs/read.md`
/// gives an error cell, its text: missing in a column of an xlsx and
/// refused in its header.
const EXCEL_ERRORS: [&str; 7] = [
    "#N/A", "#DIV/0!", "#NAME?", "#NULL!", "#NUM!", "#REF!", "#VALUE!",
];

/// The message of [`ImportError::Unreadable`] for a column whose number
/// passes 4,294,967,295, which no sheet of Excel, of 16,384 columns, and
/// no text file within the limit of bytes of an application has.
const COLUMN_PAST_U32: &str = "a column past column 4,294,967,295";

/// The table of `rows`, read as `read` says.
///
/// # Errors
///
/// [`ImportError::Refused`], with the format of `rows`, for the first in
/// this order: a header error, an error of Excel in the header of an xlsx,
/// the first by position; no row below the header, or no header, empty; a
/// column with no name and a value in the run of empty cells at the end of
/// the header, the first by position; a column with no name and a value
/// elsewhere, the first by position; two columns of one name, the first
/// name that repeats one before it; then, row by row, an empty individual
/// or a duplicate individual. [`ImportError::Unreadable`] for a column past
/// column 4,294,967,295.
pub(crate) fn table_of_rows(rows: Rows, read: HowRead) -> Result<Table, ImportError> {
    let Rows {
        origin,
        first_column,
        rows,
    } = rows;
    let refused = |refusal| ImportError::Refused {
        format: origin.format(),
        refusal,
    };
    let column_number = |index: usize| {
        u32::try_from(index)
            .ok()
            .and_then(|index| first_column.checked_add(index))
            .ok_or_else(|| ImportError::Unreadable(COLUMN_PAST_U32.to_owned()))
    };
    let mut non_blank_rows = rows
        .into_iter()
        .map(|row| row_as_taken(row, origin))
        .filter(|row| !row.cells.iter().all(|cell| *cell == Cell::Empty));
    let Some(header) = non_blank_rows.next() else {
        return Err(refused(Refusal::Empty));
    };
    let individuals: Vec<Row> = non_blank_rows.collect();
    if let Some((index, error)) = first_header_error(&header, origin) {
        return Err(refused(Refusal::HeaderError {
            row: header.place,
            column: column_number(index)?,
            error: error.to_owned(),
        }));
    }
    if individuals.is_empty() {
        return Err(refused(Refusal::Empty));
    }
    let header_names: Vec<String> = header.cells.iter().map(name_of).collect();
    let num_counted = num_counted_columns(&header_names, &individuals, origin);
    if let Some(index) = unnamed_in_run_at_end(&header_names, num_counted, &individuals, origin) {
        return Err(refused(Refusal::UnnamedColumn {
            column: column_number(index)?,
        }));
    }
    // A row of another length than the header is refused here, before the
    // columns with no name elsewhere: a row of an xlsx is always as long as
    // the rectangle, and only a text file has them.
    match origin {
        Origin::Xlsx => {}
    }
    let kept = match kept_columns(&header_names, num_counted, &individuals, origin) {
        Ok(kept) => kept,
        Err(index) => {
            return Err(refused(Refusal::UnnamedColumn {
                column: column_number(index)?,
            }));
        }
    };
    if let Some((first_index, second_index)) = first_repeated(&header_names, &kept) {
        return Err(refused(Refusal::DuplicateColumn {
            name: header_names.get(second_index).cloned().unwrap_or_default(),
            first_column: column_number(first_index)?,
            second_column: column_number(second_index)?,
        }));
    }
    let names = names_of_individuals(&individuals).map_err(refused)?;
    let mut columns_values: Vec<Vec<Option<CellValue>>> = kept
        .iter()
        .map(|_| Vec::with_capacity(individuals.len()))
        .collect();
    for row in individuals {
        let mut cells = row.cells;
        for (&index, column_values) in kept.iter().zip(&mut columns_values) {
            let cell = cells.get_mut(index).map(std::mem::take).unwrap_or_default();
            column_values.push(value_of(cell, origin));
        }
    }
    let mut header_names = header_names;
    let columns = kept
        .iter()
        .zip(columns_values)
        .map(|(&index, column_values)| {
            Ok(Column {
                name: header_names
                    .get_mut(index)
                    .map(std::mem::take)
                    .unwrap_or_default(),
                number: column_number(index)?,
                values: guessed_column(column_values, origin.decimal()),
            })
        })
        .collect::<Result<Vec<Column>, ImportError>>()?;
    Ok(Table {
        names: NameColumn {
            header: header_names.into_iter().next().unwrap_or_default(),
            number: first_column,
            names,
        },
        columns,
        read,
    })
}

/// A row with each cell as the table takes it: in an xlsx, a text cell
/// with its spaces and tabs at the ends removed, and empty when nothing is
/// left.
fn row_as_taken(row: Row, origin: Origin) -> Row {
    match origin {
        Origin::Xlsx => Row {
            place: row.place,
            cells: row.cells.into_iter().map(trimmed_cell).collect(),
        },
    }
}

/// A cell with the spaces and tabs at the ends of its text removed, empty
/// when nothing is left.
fn trimmed_cell(cell: Cell) -> Cell {
    match cell {
        Cell::Text(cell_text) => {
            let trimmed = cell_text.trim_matches([' ', '\t']);
            if trimmed.is_empty() {
                Cell::Empty
            } else if trimmed.len() == cell_text.len() {
                Cell::Text(cell_text)
            } else {
                Cell::Text(trimmed.to_owned())
            }
        }
        Cell::Empty | Cell::Number(_) | Cell::Boolean(_) => cell,
    }
}

/// Whether a text is one of the seven errors of Excel.
fn is_excel_error(cell_text: &str) -> bool {
    EXCEL_ERRORS.contains(&cell_text)
}

/// The index in the header and the text of its first cell that is an
/// error of Excel, in an xlsx.
fn first_header_error(header: &Row, origin: Origin) -> Option<(usize, &str)> {
    match origin {
        Origin::Xlsx => header
            .cells
            .iter()
            .enumerate()
            .find_map(|(index, cell)| match cell {
                Cell::Text(cell_text) if is_excel_error(cell_text) => {
                    Some((index, cell_text.as_str()))
                }
                Cell::Empty | Cell::Text(_) | Cell::Number(_) | Cell::Boolean(_) => None,
            }),
    }
}

/// The name a cell of the header or of the first column gives: a text as
/// it is, a number as JavaScript writes it with the point, a boolean
/// `TRUE` or `FALSE`, and an empty cell the empty name.
fn name_of(cell: &Cell) -> String {
    match cell {
        Cell::Empty => String::new(),
        Cell::Text(cell_text) => cell_text.clone(),
        Cell::Number(number) => float_text(*number, DecimalMark::Point),
        Cell::Boolean(is_true) => boolean_text(*is_true),
    }
}

/// Whether a cell outside the first column is missing: empty, `NA` or
/// `-`, and in an xlsx one of the seven errors of Excel.
fn is_missing_cell(cell: &Cell, origin: Origin) -> bool {
    match cell {
        Cell::Empty => true,
        Cell::Text(cell_text) => match origin {
            Origin::Xlsx => is_missing(cell_text) || is_excel_error(cell_text),
        },
        Cell::Number(_) | Cell::Boolean(_) => false,
    }
}

/// Whether the column at `index` holds a value in some row, a cell that
/// is not missing.
fn has_value(rows: &[Row], index: usize, origin: Origin) -> bool {
    rows.iter().any(|row| {
        row.cells
            .get(index)
            .is_some_and(|cell| !is_missing_cell(cell, origin))
    })
}

/// The number of cells of the header without the longest run at its end
/// of empty names whose columns hold no value in any row; the first cell
/// is always counted.
fn num_counted_columns(header_names: &[String], rows: &[Row], origin: Origin) -> usize {
    let num_dropped = header_names
        .iter()
        .enumerate()
        .skip(1)
        .rev()
        .take_while(|(index, name)| name.is_empty() && !has_value(rows, *index, origin))
        .count();
    header_names.len().saturating_sub(num_dropped)
}

/// The index of the first column, among the run of empty names at the end
/// of the first `num_counted` cells of the header, that holds a value in
/// some row; the run is not dropped only when there is one.
fn unnamed_in_run_at_end(
    header_names: &[String],
    num_counted: usize,
    rows: &[Row],
    origin: Origin,
) -> Option<usize> {
    let counted = header_names.get(..num_counted).unwrap_or(header_names);
    let num_in_run = counted
        .iter()
        .skip(1)
        .rev()
        .take_while(|name| name.is_empty())
        .count();
    let run_start = num_counted.saturating_sub(num_in_run);
    (run_start..num_counted).find(|&index| has_value(rows, index, origin))
}

/// The indices of the columns of the table other than the first, among the
/// first `num_counted` cells of the header: each with a name, and none
/// with an empty name whose cells are all missing; or the index of the
/// first column with an empty name and a value.
fn kept_columns(
    header_names: &[String],
    num_counted: usize,
    rows: &[Row],
    origin: Origin,
) -> Result<Vec<usize>, usize> {
    let mut kept = Vec::new();
    for (index, name) in header_names.iter().enumerate().take(num_counted).skip(1) {
        if !name.is_empty() {
            kept.push(index);
        } else if has_value(rows, index, origin) {
            return Err(index);
        }
    }
    Ok(kept)
}

/// The indices of the first name, among those of the first column and the
/// columns `kept`, that repeats one before it, and of the name it repeats.
fn first_repeated(header_names: &[String], kept: &[usize]) -> Option<(usize, usize)> {
    let mut first_index_of: HashMap<&str, usize> = HashMap::new();
    for index in std::iter::once(0).chain(kept.iter().copied()) {
        let name = header_names.get(index).map_or("", String::as_str);
        if let Some(&first_index) = first_index_of.get(name) {
            return Some((first_index, index));
        }
        first_index_of.insert(name, index);
    }
    None
}

/// The name of the individual of each row, its first cell, or, row by
/// row, the first that is empty or that repeats the name of a row before.
fn names_of_individuals(rows: &[Row]) -> Result<Vec<String>, Refusal> {
    let names: Vec<String> = rows
        .iter()
        .map(|row| row.cells.first().map(name_of).unwrap_or_default())
        .collect();
    let mut first_row_of: HashMap<&str, u32> = HashMap::with_capacity(names.len());
    for (name, row) in names.iter().zip(rows) {
        if name.is_empty() {
            return Err(Refusal::EmptyIndividual { row: row.place });
        }
        if let Some(&first_row) = first_row_of.get(name.as_str()) {
            return Err(Refusal::DuplicateIndividual {
                name: name.clone(),
                first_row,
                second_row: row.place,
            });
        }
        first_row_of.insert(name, row.place);
    }
    Ok(names)
}

/// The value of a cell of a column other than the first, None for a
/// missing one.
fn value_of(cell: Cell, origin: Origin) -> Option<CellValue> {
    if is_missing_cell(&cell, origin) {
        return None;
    }
    match cell {
        Cell::Empty => None,
        Cell::Text(cell_text) => Some(CellValue::Text(cell_text)),
        Cell::Number(number) => Some(CellValue::Number(number)),
        Cell::Boolean(is_true) => Some(CellValue::Boolean(is_true)),
    }
}
