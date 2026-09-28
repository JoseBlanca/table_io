//! Reads the first worksheet that is not hidden of an xlsx into its cells,
//! as `docs/specs/read.md` gives them.

#![forbid(unsafe_code)]

mod cell;
mod rectangle;

use std::io::Cursor;

use calamine::{Reader, SheetType, SheetVisible, Xlsx, XlsxError};

use crate::cell::cell_of_value;
use crate::rectangle::Rectangle;

/// A cell of the sheet: a date, a time, a duration and an error are text
/// by then, as "Each cell" of `docs/specs/read.md` gives them.
#[derive(Debug, Clone, PartialEq)]
pub enum SheetCell {
    /// No value: nothing in the file, or a text with no character.
    Empty,
    /// A text as the file holds it, spaces and line breaks kept.
    Text(String),
    /// A number, always finite; one that is not finite is given as text.
    Number(f64),
    /// `TRUE` or `FALSE`, in any language of Excel.
    Bool(bool),
}

/// The rectangle of the first worksheet that is not hidden, from the
/// first row and column that hold a value to the last ones.
#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    /// The name of the sheet, as its tab shows it.
    pub name: String,
    /// The first row of the rectangle, as Excel numbers the rows, from 1.
    pub first_row: u32,
    /// The first column of the rectangle, column A being 1.
    pub first_column: u32,
    /// The number of rows of the rectangle.
    pub num_rows: u32,
    /// The number of columns of the rectangle.
    pub num_columns: u32,
    /// The cells, row after row, `num_rows` × `num_columns` of them.
    pub cells: Vec<SheetCell>,
}

/// A file xlsx_rs does not read, with what the words of its refusal need.
#[derive(Debug, Clone, PartialEq)]
pub enum Refusal {
    /// A file that is not a zip, as every xlsx is: most often a CSV
    /// saved with the name `.xlsx`, or an empty file.
    NotXlsx,
    /// A file of the old Office, an `.xls` whose name was changed among
    /// them.
    OldExcel,
    /// An xlsx saved with a password to open it.
    Encrypted,
    /// A first worksheet that holds no value.
    EmptySheet {
        /// The name of that worksheet.
        sheet: String,
    },
    /// A cell with an error calamine does not know.
    CellError {
        /// The text of the error, such as `#GETTING_DATA`.
        error: String,
    },
    /// The rectangle of the values read when it passed the limit of cells,
    /// in the numbers of [`Sheet`].
    SheetTooLarge {
        /// The name of the sheet.
        sheet: String,
        /// The first row of the rectangle, from 1.
        first_row: u32,
        /// The first column of the rectangle, column A being 1.
        first_column: u32,
        /// The number of rows the rectangle had reached.
        num_rows: u32,
        /// The number of columns the rectangle had reached.
        num_columns: u32,
    },
}

/// Why a read gave no sheet.
#[derive(Debug, Clone, PartialEq)]
pub enum ReadError {
    /// A file refused, with what its words need.
    Refused(Refusal),
    /// A file calamine cannot read as a workbook, or whose sheet it cannot
    /// read, with calamine's message, for the console; or a workbook with
    /// no worksheet that is not hidden, "no visible worksheet".
    Unreadable(String),
}

/// Reads the first worksheet that is not hidden of the xlsx `bytes`,
/// refusing it at the first cell that makes the rectangle of the values
/// larger than `max_cells` cells.
///
/// # Errors
///
/// [`ReadError::Refused`] for a file of the refusals of
/// `docs/specs/read.md`, and [`ReadError::Unreadable`] for one calamine
/// cannot read.
#[expect(
    unused_variables,
    reason = "the limit of cells, refusal 6, is not built yet: task 2.3 of docs/plans/read.md"
)]
pub fn read_first_sheet(bytes: &[u8], max_cells: u32) -> Result<Sheet, ReadError> {
    if bytes.starts_with(&COMPOUND_FILE_MARK) {
        return Err(ReadError::Refused(refusal_of_compound_file(bytes)));
    }
    if !bytes.starts_with(&ZIP_MARK) {
        return Err(ReadError::Refused(Refusal::NotXlsx));
    }
    let mut workbook = Xlsx::new(Cursor::new(bytes)).map_err(read_error_of)?;
    let sheet_name = workbook
        .sheets_metadata()
        .iter()
        .find(|calamine_sheet| is_visible_worksheet(calamine_sheet))
        .map(|calamine_sheet| calamine_sheet.name.clone())
        .ok_or_else(|| ReadError::Unreadable("no visible worksheet".to_owned()))?;

    let (rectangle, kept_cells) = {
        let mut cells_reader = workbook
            .worksheet_cells_reader(&sheet_name)
            .map_err(read_error_of)?;
        let mut rectangle: Option<Rectangle> = None;
        let mut kept_cells = Vec::new();
        while let Some(calamine_cell) = cells_reader.next_cell().map_err(read_error_of)? {
            let cell = cell_of_value(calamine_cell.get_value());
            if cell == SheetCell::Empty {
                continue;
            }
            let position = calamine_cell.get_position();
            rectangle = Some(match rectangle {
                None => Rectangle::of_position(position),
                Some(so_far) => so_far.extended_to(position),
            });
            kept_cells.push((position, cell));
        }
        (rectangle, kept_cells)
    };

    let Some(rectangle) = rectangle else {
        return Err(ReadError::Refused(Refusal::EmptySheet {
            sheet: sheet_name,
        }));
    };
    let too_large = || ReadError::Unreadable(format!("the sheet {sheet_name} is too large"));
    Ok(Sheet {
        first_row: rectangle.excel_first_row().ok_or_else(too_large)?,
        first_column: rectangle.excel_first_column().ok_or_else(too_large)?,
        num_rows: rectangle.num_rows().ok_or_else(too_large)?,
        num_columns: rectangle.num_columns().ok_or_else(too_large)?,
        cells: rectangle.laid_out(kept_cells).ok_or_else(too_large)?,
        name: sheet_name,
    })
}

/// The eight bytes every compound file of the old Office starts with, an
/// `.xls` and an xlsx saved with a password among them.
const COMPOUND_FILE_MARK: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// The four bytes every zip starts with, `PK` and the bytes 3 and 4.
const ZIP_MARK: [u8; 4] = [b'P', b'K', 3, 4];

/// The refusal of a compound file of Office: `Encrypted` when calamine
/// finds in it the encrypted package of an xlsx saved with a password, and
/// `OldExcel` for any other.
fn refusal_of_compound_file(bytes: &[u8]) -> Refusal {
    if let Err(XlsxError::Password) = Xlsx::new(Cursor::new(bytes)) {
        Refusal::Encrypted
    } else {
        Refusal::OldExcel
    }
}

/// Whether `calamine_sheet` is a worksheet whose tab is shown.
fn is_visible_worksheet(calamine_sheet: &calamine::Sheet) -> bool {
    let is_worksheet = match calamine_sheet.typ {
        SheetType::WorkSheet => true,
        SheetType::DialogSheet | SheetType::MacroSheet | SheetType::ChartSheet | SheetType::Vba => {
            false
        }
    };
    let is_shown = match calamine_sheet.visible {
        SheetVisible::Visible => true,
        SheetVisible::Hidden | SheetVisible::VeryHidden => false,
    };
    is_worksheet && is_shown
}

/// The error of xlsx_rs for an error of calamine: the refusal it means, or
/// calamine's message.
///
/// An error calamine may add in a later version is the message, a file
/// that could not be read, until the spec makes it a refusal; so this is
/// an `if` over the errors that are refusals and not a `match` of every
/// one.
fn read_error_of(calamine_error: XlsxError) -> ReadError {
    if let XlsxError::Password = calamine_error {
        return ReadError::Refused(Refusal::Encrypted);
    }
    ReadError::Unreadable(calamine_error.to_string())
}
