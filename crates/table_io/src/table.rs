//! The table of the rows of a file, by the rules both applications share,
//! "The cells of an xlsx, as the table takes them" and "The rows" of
//! `docs/specs/import.md`: the blank rows skipped, the header, the columns
//! with no name dropped or refused, the duplicate columns, the names of the
//! individuals, the missing cells, and each column typed by
//! `docs/specs/values.md`, with the refusals of the rows in the order of
//! "The refusals". It is behind no feature: the module of each format gives
//! it the rows of its file.
//!
//! The cells of every row are held in one `Vec`, and a text cell may
//! borrow from the text of its file, so that the rows hold no more than one
//! slot for each cell and a `String` only for a text that needed one
//! (`docs/architecture.md`, section 6). The table step moves the names and
//! the values out of the cells, one column at a time, and copies none of
//! them.

use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Range;

use crate::guess::{CellValue, guessed_column};
use crate::types::boolean_text;
use crate::value::{DecimalMark, float_text, is_missing};
use crate::{Column, Format, HowRead, ImportError, NameColumn, Refusal, Table};

/// A cell of a row, as the module of its format gives it.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum Cell<'text> {
    /// No value.
    #[default]
    Empty,
    /// A text, as the file holds it: borrowed from the text of a text file
    /// where it needs no change, or a `String` of its own.
    Text(Cow<'text, str>),
    /// The number of a number cell of an xlsx, finite.
    Number(f64),
    /// The value of a boolean cell of an xlsx.
    Boolean(bool),
}

/// Where a row of [`Rows`] ends, and its place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RowEnd {
    /// The line of a text file where the row starts, or the row of the
    /// sheet of an xlsx, from 1.
    pub(crate) place: u32,
    /// The index in [`Rows::cells`] past its last cell; its first cell is
    /// at the end of the row before, or at 0 for the first row.
    pub(crate) end: usize,
}

/// The format the rows come from, which sets the rules that are the
/// format's: the spaces at the ends of a cell and the errors of Excel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Origin {
    /// The rectangle of the sheet of an xlsx: every row as long as the
    /// rectangle, a text cell with its spaces and tabs at the ends, and
    /// the seven errors of Excel missing values and refused in the header.
    Xlsx,
}

impl Origin {
    /// The format of the file, which a refusal is given with.
    fn format(self) -> Format {
        match self {
            Self::Xlsx => Format::Xlsx,
        }
    }
}

/// The rows of a file, in its order, blank ones among them: their cells
/// one after the other in one `Vec`, and where each row ends.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Rows<'text> {
    /// The format they come from.
    pub(crate) origin: Origin,
    /// The column of the first cell of every row: in an xlsx the first
    /// column of the rectangle of the sheet, column A being 1; in a text
    /// file 1.
    pub(crate) first_column: u32,
    /// The cells of every row, row after row; the cells of a row are in the
    /// columns from `first_column` on, and a row of a text file may have
    /// fewer or more cells than its header.
    pub(crate) cells: Vec<Cell<'text>>,
    /// Where each row ends in `cells`, in the order of the file, each end
    /// at least that of the row before and at most the number of cells.
    pub(crate) row_ends: Vec<RowEnd>,
}

/// A row of [`Rows`] that is not blank: its place and the range of its
/// cells.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RowSpan {
    /// The line or the row of the sheet, from 1.
    place: u32,
    /// The indices of its cells in [`Rows::cells`].
    cells: Range<usize>,
}

impl RowSpan {
    /// The index in [`Rows::cells`] of the cell of this row at `index`, or
    /// None when the row has no cell there.
    fn cell_index(&self, index: usize) -> Option<usize> {
        self.cells
            .start
            .checked_add(index)
            .filter(|cell_index| *cell_index < self.cells.end)
    }
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

/// The message of [`ImportError::Unreadable`] for rows whose ends are not
/// in order or pass their cells, which no module of a format gives.
const ROWS_NOT_THEIR_CELLS: &str = "the rows do not hold their cells";

/// The table of `rows`, read as `read` says, its values read with the
/// decimal mark of `read`.
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
/// column 4,294,967,295, or rows whose ends do not fit their cells.
pub(crate) fn table_of_rows(rows: Rows<'_>, read: HowRead) -> Result<Table, ImportError> {
    let Rows {
        origin,
        first_column,
        mut cells,
        row_ends,
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
    take_as_the_table_takes_them(&mut cells, origin);
    let spans = non_blank_rows(&cells, &row_ends)?;
    drop(row_ends);
    let Some((header, individuals)) = spans.split_first() else {
        return Err(refused(Refusal::Empty));
    };
    let header_cells = cells.get_mut(header.cells.clone()).unwrap_or_default();
    if let Some((index, error)) = first_header_error(header_cells, origin) {
        return Err(refused(Refusal::HeaderError {
            row: header.place,
            column: column_number(index)?,
            error: error.to_owned(),
        }));
    }
    if individuals.is_empty() {
        return Err(refused(Refusal::Empty));
    }
    let mut header_names: Vec<String> = header_cells
        .iter_mut()
        .map(|cell| name_of(std::mem::take(cell)))
        .collect();
    let has_value = |index: usize| {
        individuals.iter().any(|row| {
            row.cell_index(index)
                .and_then(|cell_index| cells.get(cell_index))
                .is_some_and(|cell| !is_missing_cell(cell, origin))
        })
    };
    let num_counted = num_counted_columns(&header_names, has_value);
    if let Some(index) = unnamed_in_run_at_end(&header_names, num_counted, has_value) {
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
    let kept = match kept_columns(&header_names, num_counted, has_value) {
        Ok(kept) => kept,
        Err(index) => {
            return Err(refused(Refusal::UnnamedColumn {
                column: column_number(index)?,
            }));
        }
    };
    if let Some((first_index, second_index)) = first_repeated(&header_names, &kept) {
        return Err(refused(Refusal::DuplicateColumn {
            name: header_names
                .get_mut(second_index)
                .map(std::mem::take)
                .unwrap_or_default(),
            first_column: column_number(first_index)?,
            second_column: column_number(second_index)?,
        }));
    }
    let names = names_of_individuals(&mut cells, individuals).map_err(refused)?;
    let mut columns = Vec::with_capacity(kept.len());
    for index in kept {
        let column_values: Vec<Option<CellValue<'_>>> = individuals
            .iter()
            .map(|row| {
                let cell = row
                    .cell_index(index)
                    .and_then(|cell_index| cells.get_mut(cell_index))
                    .map(std::mem::take)
                    .unwrap_or_default();
                value_of(cell, origin)
            })
            .collect();
        columns.push(Column {
            name: header_names
                .get_mut(index)
                .map(std::mem::take)
                .unwrap_or_default(),
            number: column_number(index)?,
            values: guessed_column(column_values, read.decimal()),
        });
    }
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

/// Each cell as the table takes it: in an xlsx, a text cell with its
/// spaces and tabs at the ends removed, and empty when nothing is left.
fn take_as_the_table_takes_them(cells: &mut [Cell<'_>], origin: Origin) {
    match origin {
        Origin::Xlsx => {
            for cell in cells {
                trim(cell);
            }
        }
    }
}

/// Removes the spaces and tabs at the ends of the text of `cell`, in place,
/// and makes it empty when nothing is left.
fn trim(cell: &mut Cell<'_>) {
    let Cell::Text(cell_text) = cell else {
        return;
    };
    let is_space_or_tab = |character: char| character == ' ' || character == '\t';
    let num_trailing = cell_text
        .len()
        .saturating_sub(cell_text.trim_end_matches(is_space_or_tab).len());
    let num_leading = cell_text
        .len()
        .saturating_sub(cell_text.trim_start_matches(is_space_or_tab).len());
    if num_leading == cell_text.len() {
        *cell = Cell::Empty;
        return;
    }
    if num_leading == 0 && num_trailing == 0 {
        return;
    }
    let kept_end = cell_text.len().saturating_sub(num_trailing);
    match cell_text {
        Cow::Borrowed(borrowed) => {
            *borrowed = borrowed.get(num_leading..kept_end).unwrap_or_default();
        }
        Cow::Owned(owned) => {
            owned.truncate(kept_end);
            owned.drain(..num_leading);
        }
    }
}

/// The rows of `row_ends` that are not blank, one whose cells are all
/// empty, each with the range of its cells.
///
/// # Errors
///
/// [`ImportError::Unreadable`] when the ends are not in order or pass the
/// cells.
fn non_blank_rows(cells: &[Cell<'_>], row_ends: &[RowEnd]) -> Result<Vec<RowSpan>, ImportError> {
    let mut spans = Vec::new();
    let mut start = 0;
    for row_end in row_ends {
        let row_cells = cells
            .get(start..row_end.end)
            .ok_or_else(|| ImportError::Unreadable(ROWS_NOT_THEIR_CELLS.to_owned()))?;
        if row_cells.iter().any(|cell| *cell != Cell::Empty) {
            spans.push(RowSpan {
                place: row_end.place,
                cells: start..row_end.end,
            });
        }
        start = row_end.end;
    }
    Ok(spans)
}

/// Whether a text is one of the seven errors of Excel.
fn is_excel_error(cell_text: &str) -> bool {
    EXCEL_ERRORS.contains(&cell_text)
}

/// The index in the header and the text of its first cell that is an
/// error of Excel, in an xlsx.
fn first_header_error<'cells>(
    header_cells: &'cells [Cell<'_>],
    origin: Origin,
) -> Option<(usize, &'cells str)> {
    match origin {
        Origin::Xlsx => header_cells
            .iter()
            .enumerate()
            .find_map(|(index, cell)| match cell {
                Cell::Text(cell_text) if is_excel_error(cell_text) => Some((index, &**cell_text)),
                Cell::Empty | Cell::Text(_) | Cell::Number(_) | Cell::Boolean(_) => None,
            }),
    }
}

/// The name a cell of the header or of the first column gives: a text as
/// it is, a number as JavaScript writes it with the point, a boolean
/// `TRUE` or `FALSE`, and an empty cell the empty name.
fn name_of(cell: Cell<'_>) -> String {
    match cell {
        Cell::Empty => String::new(),
        Cell::Text(cell_text) => cell_text.into_owned(),
        Cell::Number(number) => float_text(number, DecimalMark::Point),
        Cell::Boolean(is_true) => boolean_text(is_true),
    }
}

/// Whether a cell outside the first column is missing: empty, `NA` or
/// `-`, and in an xlsx one of the seven errors of Excel.
fn is_missing_cell(cell: &Cell<'_>, origin: Origin) -> bool {
    match cell {
        Cell::Empty => true,
        Cell::Text(cell_text) => match origin {
            Origin::Xlsx => is_missing(cell_text) || is_excel_error(cell_text),
        },
        Cell::Number(_) | Cell::Boolean(_) => false,
    }
}

/// The number of cells of the header without the longest run at its end
/// of empty names whose columns hold no value in any row, by `has_value`;
/// the first cell is always counted.
fn num_counted_columns(header_names: &[String], has_value: impl Fn(usize) -> bool) -> usize {
    let num_dropped = header_names
        .iter()
        .enumerate()
        .skip(1)
        .rev()
        .take_while(|(index, name)| name.is_empty() && !has_value(*index))
        .count();
    header_names.len().saturating_sub(num_dropped)
}

/// The index of the first column, among the run of empty names at the end
/// of the first `num_counted` cells of the header, that holds a value in
/// some row, by `has_value`; the run is not dropped only when there is one.
fn unnamed_in_run_at_end(
    header_names: &[String],
    num_counted: usize,
    has_value: impl Fn(usize) -> bool,
) -> Option<usize> {
    let counted = header_names.get(..num_counted).unwrap_or(header_names);
    let num_in_run = counted
        .iter()
        .skip(1)
        .rev()
        .take_while(|name| name.is_empty())
        .count();
    let run_start = num_counted.saturating_sub(num_in_run);
    (run_start..num_counted).find(|&index| has_value(index))
}

/// The indices of the columns of the table other than the first, among the
/// first `num_counted` cells of the header: each with a name, and none
/// with an empty name whose cells are all missing, by `has_value`; or the
/// index of the first column with an empty name and a value.
fn kept_columns(
    header_names: &[String],
    num_counted: usize,
    has_value: impl Fn(usize) -> bool,
) -> Result<Vec<usize>, usize> {
    let mut kept = Vec::new();
    for (index, name) in header_names.iter().enumerate().take(num_counted).skip(1) {
        if !name.is_empty() {
            kept.push(index);
        } else if has_value(index) {
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

/// The name of the individual of each row of `individuals`, its first
/// cell, moved out of `cells`; or, row by row, the first that is empty or
/// that repeats the name of a row before.
fn names_of_individuals(
    cells: &mut [Cell<'_>],
    individuals: &[RowSpan],
) -> Result<Vec<String>, Refusal> {
    let names: Vec<String> = individuals
        .iter()
        .map(|row| {
            cells
                .get_mut(row.cells.start..row.cells.end)
                .and_then(<[Cell<'_>]>::first_mut)
                .map(|cell| name_of(std::mem::take(cell)))
                .unwrap_or_default()
        })
        .collect();
    let mut first_row_of: HashMap<&str, u32> = HashMap::with_capacity(names.len());
    for (name, row) in names.iter().zip(individuals) {
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
fn value_of(cell: Cell<'_>, origin: Origin) -> Option<CellValue<'_>> {
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
