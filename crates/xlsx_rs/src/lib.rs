//! Reads the first worksheet that is not hidden of an xlsx into its cells,
//! as `docs/specs/read.md` gives them.

#![forbid(unsafe_code)]

mod attrs;
mod cell;
mod date;
mod parts;
mod rectangle;

use std::io::Cursor;

use calamine::{Reader, SheetType, SheetVisible, Xlsx, XlsxError};

use crate::cell::{TextCount, cell_of_value};
use crate::parts::PartBounds;
use crate::rectangle::{LayoutError, MergedRange, Rectangle};

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
    /// A file xlsx_rs cannot read, with a message for the console: the zip
    /// crate's, for a file that is not a zip past its first bytes or a part
    /// damaged; calamine's, for a zip it cannot read as a workbook or whose
    /// sheet it cannot read; or xlsx_rs's, for a workbook with no worksheet
    /// that is not hidden, a sheet whose cells cannot be laid out, two
    /// merged ranges that overlap, and a file that passes one of the bounds
    /// of "The refusals" and "What xlsx_rs reads before calamine" of
    /// `docs/specs/read.md`, as [`read_first_sheet`] lists them.
    Unreadable(String),
}

/// The most bytes of UTF-8 the texts of the cells of a read may hold,
/// 200,000,000, a text merged over several cells counted in each: past it
/// the read is [`ReadError::Unreadable`], "too much text".
///
/// The value is that of "The refusals" of `docs/specs/read.md`: 200 MB of
/// UTF-8 are 400 MB or more as the strings of JavaScript, the most a tab
/// can be asked to hold, where a table of individuals of 20 MB of CSV
/// holds about 20 MB of text.
pub const MAX_TEXT_BYTES: u64 = 200_000_000;

/// The most bytes the parts of a file may hold together once unzipped,
/// 1,000,000,000: past it the read is [`ReadError::Unreadable`], "the file
/// unzips to more than 1,000,000,000 bytes".
///
/// The value is that of "What xlsx_rs reads before calamine" of
/// `docs/specs/read.md`: `individuals_10000.xlsx` unzips to 6.6 times its
/// size, so a zip of 20 MB, popnei_web's limit, of the same kind unzips to
/// about 133 MB, while a zip can be written to unzip to many GB.
pub const MAX_UNZIPPED_BYTES: u64 = 1_000_000_000;

/// The most bytes each settings part may hold unzipped, 50,000,000: past
/// it the read is [`ReadError::Unreadable`], "a part of the file is too
/// large". A settings part is one calamine reads whole when it opens the
/// file, other than a table of texts, found by the end of its name in any
/// folder: `_rels/.rels`, the package relationships, and a part whose name
/// ends in `workbook.xml`, the workbook, in `workbook.xml.rels`, its
/// relationships, or in `styles.xml`, the styles.
///
/// The value is that of "What xlsx_rs reads before calamine" of
/// `docs/specs/read.md`, point 3. One Excel saves is a few KB, but a
/// workbook that gathers tens of thousands of styles or defined names can
/// pass 10 MB; calamine holds of a workbook of 50 MB of defined names
/// about 175 MB natively, estimated there from a workbook listing
/// 20,000,000 of them, 3.2 MB zipped, which took calamine 1.61 GB.
pub const MAX_SETTINGS_PART_BYTES: u64 = 50_000_000;

/// The most bytes each table of texts, the part that holds once each text
/// the cells hold, may hold unzipped, 400,000,000, twice
/// [`MAX_TEXT_BYTES`]: past it the read is [`ReadError::Unreadable`], "too
/// much text". A table of texts is a part whose name ends in
/// `sharedStrings.xml`, `\` read as `/` and ignoring case, in any folder.
pub const MAX_TEXT_TABLE_BYTES: u64 = MAX_TEXT_BYTES.saturating_mul(2);

/// The most texts, elements of local name `si`, each table of texts may
/// hold, and the largest number of texts, its attribute `uniqueCount`, it
/// may say it holds, 10,000,000: past either the read is
/// [`ReadError::Unreadable`], "too many texts". Every `si` of the part is
/// counted, and every `uniqueCount` of every element of local name `sst`
/// read, since calamine reserves room for as many texts as the first says
/// before it reads one.
///
/// The value is that of "What xlsx_rs reads before calamine" of
/// `docs/specs/read.md`, point 3: calamine holds 12 bytes in the wasm for
/// each text before its characters, 120 MB for these, where a table of
/// individuals has at most as many texts as cells, 2,000,000.
pub const MAX_TEXTS: u64 = 10_000_000;

/// The most bytes the paths of the sheets calamine keeps may be counted
/// at, 100,000,000: past it the read is [`ReadError::Unreadable`], "the
/// workbook lists too many sheets".
///
/// calamine keeps, for each sheet the workbook lists, the path of its part,
/// the folder of the workbook followed by the target of the sheet's
/// relationship, and every sheet may name the same relationship: 2,000
/// sheets naming one target of 500 KB, a zip of 8,586 bytes, took 1.0 GB
/// natively in the review of `docs/specs/read.md` of 28 September 2026.
/// The paths are counted as the largest count, in a part whose name ends in
/// `workbook.xml`, of the bytes `sheet` right after `<` or `:`, which every
/// element `sheet` has at the start of its name, times the sum of the two
/// longest tags, from `<` to `>`, of the parts named `_rels/.rels` or
/// ending in `workbook.xml.rels`, since a target and the folder calamine
/// takes from one are inside one tag ("What xlsx_rs reads before calamine",
/// point 3). A workbook of 1,000 sheets as Excel writes it counts 1,001,
/// with tags of about 150 bytes: 300 KB.
pub const MAX_SHEET_PATH_BYTES: u64 = 100_000_000;

/// Reads the first worksheet that is not hidden of the xlsx `bytes`,
/// refusing it at the first cell that makes the rectangle of the values
/// larger than `max_cells` cells.
///
/// # Errors
///
/// [`ReadError::Refused`] for a file of the refusals of
/// `docs/specs/read.md`, with what the words of the refusal need.
///
/// [`ReadError::Unreadable`] with one of these messages:
///
/// - the zip crate's, for a file that is not a zip past its first bytes, a
///   file cut short among them, and for a part of the zip whose bytes are
///   not those it was saved with, "Invalid checksum";
/// - "the file names no workbook", for a file whose package relationships,
///   `_rels/.rels`, are missing or name no workbook;
/// - "the file unzips to more than 1,000,000,000 bytes", for parts that
///   hold more than [`MAX_UNZIPPED_BYTES`] together;
/// - "a part of the file is too large", for a settings part past
///   [`MAX_SETTINGS_PART_BYTES`], in any folder;
/// - "a part of the file is not in UTF-8", for a settings part or a table
///   of texts that quick-xml decodes with another encoding, from a byte
///   order mark or from the `encoding` of its declaration;
/// - "the workbook lists too many sheets", for paths of the sheets counted
///   past [`MAX_SHEET_PATH_BYTES`];
/// - "too much text", for a part whose name ends in `sharedStrings.xml`, a
///   table of texts, past [`MAX_TEXT_TABLE_BYTES`], in any folder; and
///   "too many texts", for one of more texts than [`MAX_TEXTS`], or whose
///   attribute `uniqueCount` passes it;
/// - "the part … cannot be read as XML: …", with the message of quick-xml,
///   for package relationships or a workbook whose XML xlsx_rs cannot read
///   up to what it takes of it;
/// - calamine's, for a zip it cannot read as a workbook, or whose sheet it
///   cannot read;
/// - "no visible worksheet", for a workbook whose worksheets are all
///   hidden;
/// - "the sheet … has a rectangle of … cells, more than the memory can
///   hold", when the cells of a rectangle within `max_cells` cannot be
///   allocated;
/// - "the sheet … has a cell at row … and column …, counted from 0,
///   outside its rectangle", and the four messages of a first row or
///   column, or a number of rows or columns, past what a `u32` holds,
///   which no file calamine reads can give and which are there so that
///   the read has no panic;
/// - "too many merged ranges", for a part of the file with more merged
///   ranges than `max_cells`, counted as the bytes `mergeCell` right after
///   `<` or `:`, any sheet's and not only the one read;
/// - "overlapping merged ranges", for two merged ranges that share a cell;
/// - "cells written more than once", for a file that gives more cells
///   with a value than `max_cells`, a cell written again counted again;
/// - "too much text", for texts of the cells past [`MAX_TEXT_BYTES`].
pub fn read_first_sheet(bytes: &[u8], max_cells: u32) -> Result<Sheet, ReadError> {
    read_first_sheet_within(bytes, max_cells, PART_BOUNDS)
}

/// The bounds of the parts of [`read_first_sheet`].
const PART_BOUNDS: PartBounds = PartBounds {
    max_unzipped_bytes: MAX_UNZIPPED_BYTES,
    max_settings_part_bytes: MAX_SETTINGS_PART_BYTES,
    max_text_table_bytes: MAX_TEXT_TABLE_BYTES,
    max_texts: MAX_TEXTS,
    max_sheet_path_bytes: MAX_SHEET_PATH_BYTES,
};

/// Reads as [`read_first_sheet`] does, with `max_unzipped_bytes` in the
/// place of [`MAX_UNZIPPED_BYTES`], so that a test can pass the bound with
/// a file of a few KB instead of unzipping a GB.
///
/// # Errors
///
/// Those of [`read_first_sheet`], the bound of the unzipped bytes being
/// `max_unzipped_bytes`: "the file unzips to more than … bytes", with
/// `max_unzipped_bytes`.
pub fn read_first_sheet_within_unzipped_bytes(
    bytes: &[u8],
    max_cells: u32,
    max_unzipped_bytes: u64,
) -> Result<Sheet, ReadError> {
    let bounds = PartBounds {
        max_unzipped_bytes,
        max_settings_part_bytes: MAX_SETTINGS_PART_BYTES,
        max_text_table_bytes: MAX_TEXT_TABLE_BYTES,
        max_texts: MAX_TEXTS,
        max_sheet_path_bytes: MAX_SHEET_PATH_BYTES,
    };
    read_first_sheet_within(bytes, max_cells, bounds)
}

/// Reads as [`read_first_sheet`] does, with `max_text_table_bytes` in the
/// place of [`MAX_TEXT_TABLE_BYTES`], so that a test can pass the bound
/// with a file of a few KB instead of 400 MB of text.
///
/// # Errors
///
/// Those of [`read_first_sheet`], the bound of the bytes of the table of
/// texts being `max_text_table_bytes`.
pub fn read_first_sheet_within_text_table_bytes(
    bytes: &[u8],
    max_cells: u32,
    max_text_table_bytes: u64,
) -> Result<Sheet, ReadError> {
    let bounds = PartBounds {
        max_unzipped_bytes: MAX_UNZIPPED_BYTES,
        max_settings_part_bytes: MAX_SETTINGS_PART_BYTES,
        max_text_table_bytes,
        max_texts: MAX_TEXTS,
        max_sheet_path_bytes: MAX_SHEET_PATH_BYTES,
    };
    read_first_sheet_within(bytes, max_cells, bounds)
}

/// Reads as [`read_first_sheet`] does, within `bounds`.
fn read_first_sheet_within(
    bytes: &[u8],
    max_cells: u32,
    bounds: PartBounds,
) -> Result<Sheet, ReadError> {
    if bytes.starts_with(&COMPOUND_FILE_MARK) {
        return Err(ReadError::Refused(refusal_of_compound_file(bytes)));
    }
    if !bytes.starts_with(&ZIP_MARK) {
        return Err(ReadError::Refused(Refusal::NotXlsx));
    }
    // Read before calamine, which takes the date system from an element of
    // another namespace in a workbook Excel 365 saves, reads four parts
    // whole, with no bound, when it opens the file, and holds every merged
    // range of the sheet ("What xlsx_rs reads before calamine" of
    // docs/specs/read.md, points 2 to 4).
    let date_system = parts::read_parts(bytes, max_cells, bounds)?;
    let mut workbook = Xlsx::new(Cursor::new(bytes)).map_err(read_error_of)?;
    let sheet_name = workbook
        .sheets_metadata()
        .iter()
        .find(|calamine_sheet| is_visible_worksheet(calamine_sheet))
        .map(|calamine_sheet| calamine_sheet.name.clone())
        .ok_or_else(|| ReadError::Unreadable("no visible worksheet".to_owned()))?;

    let (rectangle, kept_cells, mut text_count) = {
        let mut cells_reader = workbook
            .worksheet_cells_reader(&sheet_name)
            .map_err(read_error_of)?;
        let mut rectangle: Option<Rectangle> = None;
        let mut kept_cells = Vec::new();
        let mut text_count = TextCount::with_bound(MAX_TEXT_BYTES);
        while let Some(calamine_cell) = cells_reader.next_cell().map_err(read_error_of)? {
            let cell = cell_of_value(calamine_cell.get_value(), date_system);
            if cell == SheetCell::Empty {
                continue;
            }
            let position = calamine_cell.get_position();
            let extended_rectangle = match rectangle {
                None => Rectangle::of_position(position),
                Some(rectangle_so_far) => rectangle_so_far.extended_to(position),
            };
            // Checked at each cell, so that the read stops at the first cell
            // past the limit and never holds more cells than it.
            if extended_rectangle.num_cells() > u64::from(max_cells) {
                let excel_rectangle = excel_rectangle_of(&extended_rectangle, &sheet_name)?;
                return Err(ReadError::Refused(Refusal::SheetTooLarge {
                    sheet: sheet_name.clone(),
                    first_row: excel_rectangle.first_row,
                    first_column: excel_rectangle.first_column,
                    num_rows: excel_rectangle.num_rows,
                    num_columns: excel_rectangle.num_columns,
                }));
            }
            // A cell written again is kept again, so a file that writes one
            // cell many times would hold every copy in a rectangle within the
            // limit; the cells kept, as written, are held to it too.
            let num_kept_cells = u64::try_from(kept_cells.len()).unwrap_or(u64::MAX);
            if num_kept_cells >= u64::from(max_cells) {
                return Err(ReadError::Unreadable(
                    "cells written more than once".to_owned(),
                ));
            }
            text_count.count(&cell).map_err(|_| too_much_text())?;
            rectangle = Some(extended_rectangle);
            kept_cells.push((position, cell));
        }
        (rectangle, kept_cells, text_count)
    };

    let Some(rectangle) = rectangle else {
        return Err(ReadError::Refused(Refusal::EmptySheet {
            sheet: sheet_name,
        }));
    };
    // Read once the cells are, so that a sheet refused as it is read is not
    // read a second time for its merged ranges; their number is bounded by
    // parts::read_parts, since calamine holds them all ("What xlsx_rs reads
    // before calamine" of docs/specs/read.md, point 4).
    let merged_ranges: Vec<MergedRange> = workbook
        .merge_cells_by_sheet_name(&sheet_name)
        .map_err(read_error_of)?
        .into_iter()
        .map(|calamine_range| MergedRange::of_corners(calamine_range.start, calamine_range.end))
        .collect();
    let excel_rectangle = excel_rectangle_of(&rectangle, &sheet_name)?;
    let cells = rectangle
        .laid_out(kept_cells, &merged_ranges, &mut text_count)
        .map_err(|layout_error| match layout_error {
            LayoutError::TooManyCells { num_cells } => unreadable_error_of(
                &sheet_name,
                &format!("has a rectangle of {num_cells} cells, more than the memory can hold"),
            ),
            LayoutError::PositionOutside {
                position: (row, column),
            } => unreadable_error_of(
                &sheet_name,
                &format!(
                    "has a cell at row {row} and column {column}, counted from 0, outside its rectangle"
                ),
            ),
            LayoutError::OverlappingMergedRanges => {
                ReadError::Unreadable("overlapping merged ranges".to_owned())
            }
            LayoutError::TooMuchText => too_much_text(),
        })?;
    Ok(Sheet {
        name: sheet_name,
        first_row: excel_rectangle.first_row,
        first_column: excel_rectangle.first_column,
        num_rows: excel_rectangle.num_rows,
        num_columns: excel_rectangle.num_columns,
        cells,
    })
}

/// The rectangle of the values, from the first row and column that hold a
/// value to the last ones ("The sheet read" of `docs/specs/read.md`), in
/// the numbers [`Sheet`] and [`Refusal::SheetTooLarge`] give: its first row
/// and column as Excel numbers them, from 1, and its numbers of rows and
/// columns.
struct ExcelRectangle {
    first_row: u32,
    first_column: u32,
    num_rows: u32,
    num_columns: u32,
}

/// The [`ExcelRectangle`] of `rectangle`, or [`ReadError::Unreadable`] when
/// one of them does not fit in a `u32`.
fn excel_rectangle_of(
    rectangle: &Rectangle,
    sheet_name: &str,
) -> Result<ExcelRectangle, ReadError> {
    let first_row = rectangle.excel_first_row().ok_or_else(|| {
        unreadable_error_of(
            sheet_name,
            "has its first row past row 4,294,967,295, the last a u32 holds",
        )
    })?;
    let first_column = rectangle.excel_first_column().ok_or_else(|| {
        unreadable_error_of(
            sheet_name,
            "has its first column past column 4,294,967,295, the last a u32 holds",
        )
    })?;
    let num_rows = rectangle.num_rows().ok_or_else(|| {
        unreadable_error_of(sheet_name, "has 4,294,967,296 rows, more than a u32 holds")
    })?;
    let num_columns = rectangle.num_columns().ok_or_else(|| {
        unreadable_error_of(
            sheet_name,
            "has 4,294,967,296 columns, more than a u32 holds",
        )
    })?;
    Ok(ExcelRectangle {
        first_row,
        first_column,
        num_rows,
        num_columns,
    })
}

/// The error of a read whose texts passed [`MAX_TEXT_BYTES`].
fn too_much_text() -> ReadError {
    ReadError::Unreadable("too much text".to_owned())
}

/// The error of a sheet xlsx_rs cannot give, with `cause` after its name.
fn unreadable_error_of(sheet_name: &str, cause: &str) -> ReadError {
    ReadError::Unreadable(format!("the sheet {sheet_name} {cause}"))
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
        XlsxError::CellError(error) => ReadError::Refused(Refusal::CellError { error }),
        XlsxError::Io(_)
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
    fn calamines_error_of_a_cell_is_the_refusal_cell_error_with_its_text() {
        assert_eq!(
            read_error_of(XlsxError::CellError("#SPILL!".to_owned())),
            ReadError::Refused(Refusal::CellError {
                error: "#SPILL!".to_owned()
            })
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
