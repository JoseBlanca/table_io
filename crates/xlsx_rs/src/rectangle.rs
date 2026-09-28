//! The rectangle of the values of a sheet, and its cells laid out row
//! after row ("The sheet read" of `docs/specs/read.md`).

use crate::SheetCell;
use crate::cell::TextCount;

/// A position as calamine gives it: the row and the column, both from 0.
pub(crate) type Position = (u32, u32);

/// A range of merged cells, from its first cell, at its top left, to its
/// last, at its bottom right, both included.
#[derive(Debug, Clone, Copy)]
pub(crate) struct MergedRange {
    /// The position of the first cell, which holds the value of the range.
    first: Position,
    /// The position of the last cell.
    last: Position,
}

impl MergedRange {
    /// The range between two opposite corners, `corner` and
    /// `opposite_corner`, in either order: a range the file writes from its
    /// last cell to its first, `B4:B2`, is the range `B2:B4` ("Merged
    /// cells" of `docs/specs/read.md`).
    pub(crate) fn of_corners(corner: Position, opposite_corner: Position) -> Self {
        let (corner_row, corner_column) = corner;
        let (opposite_row, opposite_column) = opposite_corner;
        Self {
            first: (
                corner_row.min(opposite_row),
                corner_column.min(opposite_column),
            ),
            last: (
                corner_row.max(opposite_row),
                corner_column.max(opposite_column),
            ),
        }
    }
}

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

    /// The number of cells, rows times columns, `u64::MAX` for the one
    /// rectangle whose count does not fit in a `u64`, of 2^32 rows and 2^32
    /// columns, which no memory holds either.
    pub(crate) fn num_cells(&self) -> u64 {
        // The last row and column are never before the first ones, as
        // `of_position` and `extended_to` build them.
        let rows = u64::from(self.last_row)
            .saturating_sub(u64::from(self.first_row))
            .saturating_add(1);
        let columns = u64::from(self.last_column)
            .saturating_sub(u64::from(self.first_column))
            .saturating_add(1);
        rows.saturating_mul(columns)
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
    /// position, and the others empty; then every cell of each of
    /// `merged_ranges` inside the rectangle with the value of the range's
    /// first cell, whatever `kept_cells` held there, which is empty when
    /// that cell is outside the rectangle, since every value is inside it.
    ///
    /// # Errors
    ///
    /// [`LayoutError::TooManyCells`] when the memory cannot hold the cells
    /// of the rectangle, [`LayoutError::PositionOutside`] for a cell of
    /// `kept_cells` outside it,
    /// [`LayoutError::OverlappingMergedRanges`] when two of
    /// `merged_ranges` share a cell inside it, and
    /// [`LayoutError::TooMuchText`] when the text a range copies into its
    /// cells, counted in `text_count`, passes its bound.
    pub(crate) fn laid_out(
        &self,
        kept_cells: Vec<(Position, SheetCell)>,
        merged_ranges: &[MergedRange],
        text_count: &mut TextCount,
    ) -> Result<Vec<SheetCell>, LayoutError> {
        let num_cells = self.num_cells();
        let too_many_cells = || LayoutError::TooManyCells { num_cells };
        let capacity = usize::try_from(num_cells).map_err(|_| too_many_cells())?;
        let cells = {
            let mut cells = Vec::new();
            // A failed allocation is an error here, where `vec!` would abort,
            // which in the wasm is a trap.
            cells
                .try_reserve_exact(capacity)
                .map_err(|_| too_many_cells())?;
            cells.resize(capacity, SheetCell::Empty);
            for (position, cell) in kept_cells {
                let slot = self
                    .index_of(position)
                    .and_then(|index| cells.get_mut(index))
                    .ok_or(LayoutError::PositionOutside { position })?;
                *slot = cell;
            }
            if !merged_ranges.is_empty() {
                // One mark for each cell of the rectangle a range has filled,
                // so that an overlap is found in as many steps as the cells
                // filled, and not by comparing every pair of ranges.
                let mut is_merged = Vec::new();
                is_merged
                    .try_reserve_exact(capacity)
                    .map_err(|_| too_many_cells())?;
                is_merged.resize(capacity, false);
                for merged_range in merged_ranges {
                    self.fill_merged_range(&mut cells, &mut is_merged, text_count, *merged_range)?;
                }
            }
            cells
        };
        Ok(cells)
    }

    /// Gives every cell of `merged_range` inside the rectangle, among
    /// `cells` laid out row after row, the value of the range's first cell,
    /// and marks it in `is_merged`, laid out the same way. The value copied
    /// into each cell but the first, which was counted as it was read, is
    /// counted in `text_count` before it is copied.
    ///
    /// # Errors
    ///
    /// [`LayoutError::OverlappingMergedRanges`] at a cell `is_merged`
    /// already marks, one of another range, and [`LayoutError::TooMuchText`]
    /// when the count passes its bound.
    fn fill_merged_range(
        &self,
        cells: &mut [SheetCell],
        is_merged: &mut [bool],
        text_count: &mut TextCount,
        merged_range: MergedRange,
    ) -> Result<(), LayoutError> {
        let MergedRange {
            first: (first_row, first_column),
            last: (last_row, last_column),
        } = merged_range;
        let first_value = self
            .index_of(merged_range.first)
            .and_then(|index| cells.get(index))
            .cloned()
            .unwrap_or(SheetCell::Empty);
        let rows_inside = first_row.max(self.first_row)..=last_row.min(self.last_row);
        let columns_inside =
            first_column.max(self.first_column)..=last_column.min(self.last_column);
        for row in rows_inside {
            for column in columns_inside.clone() {
                let Some(index) = self.index_of((row, column)) else {
                    continue;
                };
                if let Some(mark) = is_merged.get_mut(index) {
                    if *mark {
                        return Err(LayoutError::OverlappingMergedRanges);
                    }
                    *mark = true;
                }
                if (row, column) != merged_range.first {
                    text_count
                        .count(&first_value)
                        .map_err(|_| LayoutError::TooMuchText)?;
                }
                if let Some(slot) = cells.get_mut(index) {
                    slot.clone_from(&first_value);
                }
            }
        }
        Ok(())
    }
}

/// Why the cells of a rectangle could not be laid out.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LayoutError {
    /// The memory cannot hold the cells of the rectangle, `num_cells` of
    /// them.
    TooManyCells {
        /// The number of cells of the rectangle.
        num_cells: u64,
    },
    /// A cell kept at a position outside the rectangle.
    PositionOutside {
        /// The position of the cell, as calamine gives it.
        position: Position,
    },
    /// Two merged ranges that share a cell of the rectangle.
    OverlappingMergedRanges,
    /// The texts of the cells passed the bound of their count.
    TooMuchText,
}

#[cfg(test)]
mod tests {
    use crate::SheetCell;
    use crate::cell::TextCount;
    use crate::rectangle::{LayoutError, MergedRange, Rectangle};

    #[test]
    fn a_rectangle_of_more_cells_than_memory_holds_is_an_error() {
        // (2^32 - 1)^2 cells of 16 bytes: past what any memory can address,
        // so nothing is allocated before the error.
        let rectangle = Rectangle::of_position((0, 0)).extended_to((u32::MAX - 1, u32::MAX - 1));

        assert_eq!(
            rectangle.laid_out(Vec::new(), &[], &mut TextCount::with_bound(0)),
            Err(LayoutError::TooManyCells {
                num_cells: 18_446_744_065_119_617_025
            })
        );
    }

    #[test]
    fn a_cell_outside_the_rectangle_is_an_error() {
        let rectangle = Rectangle::of_position((1, 1));
        let kept_cells = vec![((0, 0), SheetCell::Bool(true))];

        assert_eq!(
            rectangle.laid_out(kept_cells, &[], &mut TextCount::with_bound(0)),
            Err(LayoutError::PositionOutside { position: (0, 0) })
        );
    }

    #[test]
    fn a_merged_range_whose_first_cell_is_outside_the_rectangle_empties_its_cells_inside() {
        // The first cell of a range holds its value, and every value is
        // inside the rectangle, so a first cell outside it has none. A file
        // Excel writes has nothing in the other cells; this one does.
        let rectangle = Rectangle::of_position((1, 1)).extended_to((2, 2));
        let kept_cells = vec![
            ((1, 1), SheetCell::Bool(true)),
            ((2, 2), SheetCell::Number(7.0)),
        ];
        let merged_range = MergedRange::of_corners((0, 0), (1, 1));

        assert_eq!(
            rectangle.laid_out(kept_cells, &[merged_range], &mut TextCount::with_bound(0)),
            Ok(vec![
                SheetCell::Empty,
                SheetCell::Empty,
                SheetCell::Empty,
                SheetCell::Number(7.0),
            ])
        );
    }
}
