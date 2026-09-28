//! Reads the first worksheet that is not hidden of an xlsx into its cells,
//! as `docs/specs/read.md` gives them.

#![forbid(unsafe_code)]

mod cell;
mod rectangle;

use std::io::Cursor;

use calamine::{Reader, SheetType, SheetVisible, Xlsx, XlsxError};

use crate::cell::cell_of_value;
use crate::rectangle::{LayoutError, Rectangle};

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
    let sheet_error =
        |cause: String| ReadError::Unreadable(format!("the sheet {sheet_name} {cause}"));
    let first_row = rectangle.excel_first_row().ok_or_else(|| {
        sheet_error("has its first row past row 4,294,967,295, the last a u32 holds".to_owned())
    })?;
    let first_column = rectangle.excel_first_column().ok_or_else(|| {
        sheet_error(
            "has its first column past column 4,294,967,295, the last a u32 holds".to_owned(),
        )
    })?;
    let num_rows = rectangle
        .num_rows()
        .ok_or_else(|| sheet_error("has 4,294,967,296 rows, more than a u32 holds".to_owned()))?;
    let num_columns = rectangle.num_columns().ok_or_else(|| {
        sheet_error("has 4,294,967,296 columns, more than a u32 holds".to_owned())
    })?;
    let cells = rectangle
        .laid_out(kept_cells)
        .map_err(|layout_error| match layout_error {
            LayoutError::TooManyCells { num_cells } => sheet_error(format!(
                "has a rectangle of {num_cells} cells, more than the memory can hold"
            )),
            LayoutError::PositionOutside {
                position: (row, column),
            } => sheet_error(format!(
                "has a cell at row {row} and column {column}, counted from 0, outside its rectangle"
            )),
        })?;
    Ok(Sheet {
        name: sheet_name,
        first_row,
        first_column,
        num_rows,
        num_columns,
        cells,
    })
}

/// The eight bytes every compound file of the old Office starts with, an
/// `.xls` and an xlsx saved with a password among them.
const COMPOUND_FILE_MARK: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// The four bytes every zip starts with, `PK` and the bytes 3 and 4.
const ZIP_MARK: [u8; 4] = [b'P', b'K', 3, 4];

/// The name of the part an xlsx saved with a password holds its workbook
/// in, `EncryptedPackage`, as a compound file writes the names of its
/// parts: in UTF-16, little-endian.
const ENCRYPTED_PACKAGE_NAME: &[u8; 32] = b"E\0n\0c\0r\0y\0p\0t\0e\0d\0P\0a\0c\0k\0a\0g\0e\0";

/// The refusal of a compound file of Office: `Encrypted` when its bytes
/// hold the name [`ENCRYPTED_PACKAGE_NAME`], and `OldExcel` for any other.
///
/// calamine is not asked, since its reader of compound files panics on one
/// cut short, which in the wasm ends the worker (refusal 1 of
/// `docs/specs/read.md`).
fn refusal_of_compound_file(bytes: &[u8]) -> Refusal {
    let holds_encrypted_package = bytes
        .windows(ENCRYPTED_PACKAGE_NAME.len())
        .any(|window| window == ENCRYPTED_PACKAGE_NAME);
    if holds_encrypted_package {
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
/// Every error of calamine is named, so that one its next version adds
/// stops the build here, where whether it is a refusal is decided.
fn read_error_of(calamine_error: XlsxError) -> ReadError {
    match calamine_error {
        XlsxError::Password => ReadError::Refused(Refusal::Encrypted),
        // An error of a cell calamine does not know is to be refusal 5,
        // `CellError`, in task 2.1 of docs/plans/read.md; the message until
        // then.
        XlsxError::CellError(_)
        | XlsxError::Io(_)
        | XlsxError::Zip(_)
        | XlsxError::Vba(_)
        | XlsxError::Xml(_)
        | XlsxError::XmlAttr(_)
        | XlsxError::Parse(_)
        | XlsxError::ParseFloat(_)
        | XlsxError::ParseInt(_)
        | XlsxError::XmlEof(_)
        | XlsxError::UnexpectedNode(_)
        | XlsxError::FileNotFound(_)
        | XlsxError::RelationshipNotFound
        | XlsxError::Alphanumeric(_)
        | XlsxError::NumericColumn(_)
        | XlsxError::RangeWithoutColumnComponent
        | XlsxError::RangeWithoutRowComponent
        | XlsxError::ColumnNumberOverflow
        | XlsxError::RowNumberOverflow
        | XlsxError::DimensionCount(_)
        | XlsxError::CellTAttribute(_)
        | XlsxError::Unexpected(_)
        | XlsxError::Unrecognized { .. }
        | XlsxError::WorksheetNotFound(_)
        | XlsxError::TableNotFound(_)
        | XlsxError::NotAWorksheet(_)
        | XlsxError::Encoding(_)
        | XlsxError::PivotTableNotFound(_) => ReadError::Unreadable(calamine_error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use calamine::XlsxError;

    use crate::{ReadError, Refusal, read_error_of};

    #[test]
    fn calamines_error_of_a_password_is_the_refusal_encrypted() {
        assert_eq!(
            read_error_of(XlsxError::Password),
            ReadError::Refused(Refusal::Encrypted)
        );
    }

    #[test]
    fn another_error_of_calamine_is_unreadable_with_its_message() {
        assert_eq!(
            read_error_of(XlsxError::FileNotFound("xl/workbook.xml".to_owned())),
            ReadError::Unreadable("File not found 'xl/workbook.xml'".to_owned())
        );
    }
}
