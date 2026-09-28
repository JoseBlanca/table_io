//! The rectangle of the values of a sheet, and its cells laid out row
//! after row ("The sheet read" of `docs/specs/read.md`).

use crate::SheetCell;

/// A position as calamine gives it: the row and the column, both from 0.
pub(crate) type Position = (u32, u32);

/// The smallest rectangle that holds every position seen, its rows and
/// columns counted from 0 as calamine counts them, the last ones included.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Rectangle {
    first_row: u32,
    first_column: u32,
    last_row: u32,
    last_column: u32,
}

impl Rectangle {
    /// The rectangle of the one cell at `position`.
    pub(crate) fn of_position(position: Position) -> Self {
        let (row, column) = position;
        Self {
            first_row: row,
            first_column: column,
            last_row: row,
            last_column: column,
        }
    }

    /// The smallest rectangle that holds this one and `position`.
    pub(crate) fn extended_to(self, position: Position) -> Self {
        let (row, column) = position;
        Self {
            first_row: self.first_row.min(row),
            first_column: self.first_column.min(column),
            last_row: self.last_row.max(row),
            last_column: self.last_column.max(column),
        }
    }

    /// The first row as Excel numbers the rows, from 1; `None` past `u32`.
    pub(crate) fn excel_first_row(&self) -> Option<u32> {
        self.first_row.checked_add(1)
    }

    /// The first column as Excel numbers the columns, A being 1; `None`
    /// past `u32`.
    pub(crate) fn excel_first_column(&self) -> Option<u32> {
        self.first_column.checked_add(1)
    }

    /// The number of rows; `None` past `u32`.
    pub(crate) fn num_rows(&self) -> Option<u32> {
        self.last_row.checked_sub(self.first_row)?.checked_add(1)
    }

    /// The number of columns; `None` past `u32`.
    pub(crate) fn num_columns(&self) -> Option<u32> {
        self.last_column
            .checked_sub(self.first_column)?
            .checked_add(1)
    }

    /// The number of cells, rows times columns; `None` when the rows or the
    /// columns are past `u32`.
    fn num_cells(&self) -> Option<u64> {
        u64::from(self.num_rows()?).checked_mul(u64::from(self.num_columns()?))
    }

    /// Where the cell at `position` is among the cells laid out row after
    /// row; `None` for a position outside the rectangle.
    fn index_of(&self, position: Position) -> Option<usize> {
        let (row, column) = position;
        let rows_before = u64::from(row.checked_sub(self.first_row)?);
        let columns_before = u64::from(column.checked_sub(self.first_column)?);
        if row > self.last_row || column > self.last_column {
            return None;
        }
        let index = rows_before
            .checked_mul(u64::from(self.num_columns()?))?
            .checked_add(columns_before)?;
        usize::try_from(index).ok()
    }

    /// The cells of the rectangle, row after row: those of `kept_cells` at
    /// their positions, a later one over an earlier one at the same
    /// position, and the others empty. `None` when the rectangle does not
    /// fit in memory or a position is outside it.
    pub(crate) fn laid_out(
        &self,
        kept_cells: Vec<(Position, SheetCell)>,
    ) -> Option<Vec<SheetCell>> {
        let num_cells = usize::try_from(self.num_cells()?).ok()?;
        let mut cells = vec![SheetCell::Empty; num_cells];
        for (position, cell) in kept_cells {
            let slot = cells.get_mut(self.index_of(position)?)?;
            *slot = cell;
        }
        Some(cells)
    }
}
