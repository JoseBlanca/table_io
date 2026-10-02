//! The import of a table, `docs/specs/import.md`: the types an application
//! gives and gets, and `import_table`, which finds the format of a file
//! from its first bytes, refuses a file too large or of a format the build
//! does not read, and reads the rest with the module of its format. It is
//! behind no feature.

use crate::{ColumnValues, DecimalMark, Separator, TextOptions, TextRead};

/// What the caller accepts and sets for one import.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportOptions {
    /// The largest file, in bytes; a larger one is refused as
    /// [`Refusal::TooLarge`] before anything of it is read but its first
    /// bytes.
    pub max_bytes: u64,
    /// The largest number of cells of the rectangle of the values of an
    /// xlsx, from the first row and column that hold a value to the last
    /// ones; a text file is bounded by `max_bytes` alone.
    pub max_cells: u32,
    /// The options of a text file; ignored for an xlsx.
    pub text: TextOptions,
}

/// A table read from a file: the column of the names of the individuals,
/// the other columns, each typed, and how the file was read.
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    /// The first column of the file, the names of the individuals.
    pub names: NameColumn,
    /// The other columns, in the order of the file.
    pub columns: Vec<Column>,
    /// The format the file was read as, and with what.
    pub read: HowRead,
}

/// The first column of a table, which names the individuals, one for each
/// row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameColumn {
    /// Its name in the header, which may be empty.
    pub header: String,
    /// Its column in the file, from 1: in an xlsx, its column of the sheet,
    /// column A being 1.
    pub number: u32,
    /// One name for each row, none empty, no two the same.
    pub names: Vec<String>,
}

/// A column of a table other than the first, with its type and values.
#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    /// Its name in the header, not empty, no two the same in a table.
    pub name: String,
    /// Its column in the file, from 1: in an xlsx, its column of the sheet,
    /// column A being 1.
    pub number: u32,
    /// One value for each row, as many as the names.
    pub values: ColumnValues,
}

/// How a file was read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HowRead {
    /// As a text file, with the encoding, the separator and the decimal
    /// mark set or found.
    Text(TextRead),
    /// As an xlsx, from its first worksheet that is not hidden.
    Xlsx {
        /// The name of that worksheet, as its tab shows it.
        sheet: String,
    },
}

impl HowRead {
    /// The decimal mark the values were read with, which a conversion of
    /// a column of this table takes: the one of a text file, and the point
    /// for an xlsx.
    pub fn decimal(&self) -> DecimalMark {
        match self {
            Self::Text(text_read) => text_read.decimal,
            Self::Xlsx { .. } => DecimalMark::Point,
        }
    }
}

/// Why an import gave no table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportError {
    /// A file refused, with the format it was found to be, so that the
    /// application writes a place of the refusal as the user will look for
    /// it: a line of a text file, a row and a column of the sheet of an
    /// xlsx.
    Refused {
        /// The format found from the first bytes of the file.
        format: Format,
        /// The refusal, with what its words need.
        refusal: Refusal,
    },
    /// A file that cannot be read, a damaged one or one past a bound
    /// table_io checks, with the message of the zip crate, of calamine or
    /// of table_io, for whoever reports the problem.
    Unreadable(String),
}

/// A refusal or an unreadable file, written for whoever reports the
/// problem, in English and with its fields, such as "an xlsx refused: the
/// error #VALUE! in the header, at row 6 and column 5 of the sheet"; the
/// words a user reads are the application's.
impl std::fmt::Display for ImportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Refused { format, refusal } => {
                let format_text = match format {
                    Format::Text => "a text file",
                    Format::Xlsx => "an xlsx",
                };
                write!(formatter, "{format_text} refused: ")?;
                write_refusal(formatter, refusal)
            }
            Self::Unreadable(message) => write!(formatter, "an unreadable file: {message}"),
        }
    }
}

impl std::error::Error for ImportError {}

/// Writes `refusal` with its fields, for [`ImportError`]'s `Display`.
fn write_refusal(formatter: &mut std::fmt::Formatter<'_>, refusal: &Refusal) -> std::fmt::Result {
    match refusal {
        Refusal::TooLarge { size, max_bytes } => write!(
            formatter,
            "too large, {size} bytes, past the limit of {max_bytes} bytes"
        ),
        Refusal::FormatNotBuilt => {
            write!(formatter, "a format this build of table_io does not read")
        }
        Refusal::OldExcel => write!(formatter, "a workbook of Excel 97-2003, an .xls"),
        Refusal::Encrypted => write!(formatter, "a workbook saved with a password"),
        Refusal::NotWorkbook => write!(formatter, "a zip that holds no workbook"),
        Refusal::EmptySheet { sheet } => write!(formatter, "the sheet '{sheet}' holds no value"),
        Refusal::CellError { error } => {
            write!(
                formatter,
                "a cell holds the error {error}, which calamine does not know"
            )
        }
        Refusal::SheetTooLarge {
            sheet,
            first_row,
            first_column,
            num_rows,
            num_columns,
        } => write!(
            formatter,
            "the sheet '{sheet}' passes the limit of cells, its rectangle from row \
             {first_row} and column {first_column} having reached {num_rows} rows of \
             {num_columns} columns"
        ),
        Refusal::CutShort => write!(formatter, "a file in UTF-16 cut short in a character"),
        Refusal::NotText => write!(formatter, "a byte 0 in a file that is not in UTF-16"),
        Refusal::VariantsFile => write!(formatter, "a file of variants, a VCF"),
        Refusal::UnclosedQuote { line, separator } => write!(
            formatter,
            "a quote opened in the cell that starts at line {line} is never closed, split \
             at {}",
            separator_text(*separator)
        ),
        Refusal::HeaderError { row, column, error } => write!(
            formatter,
            "the error {error} in the header, at row {row} and column {column} of the sheet"
        ),
        Refusal::Empty => write!(formatter, "no row below the header"),
        Refusal::UnnamedColumn { column } => {
            write!(formatter, "column {column} has values and no name")
        }
        Refusal::RaggedRow {
            line,
            expected,
            found,
            separator,
        } => write!(
            formatter,
            "line {line} has {found} cells where the header has {expected}, split at {}",
            separator_text(*separator)
        ),
        Refusal::DuplicateColumn {
            name,
            first_column,
            second_column,
        } => write!(
            formatter,
            "the name '{name}' is used by the columns {first_column} and {second_column}"
        ),
        Refusal::EmptyIndividual { row } => {
            write!(formatter, "the row {row} has no name of an individual")
        }
        Refusal::DuplicateIndividual {
            name,
            first_row,
            second_row,
        } => write!(
            formatter,
            "the individual '{name}' is in the rows {first_row} and {second_row}"
        ),
    }
}

/// The separator as it is written in a file, quoted, the tab as `\t`.
fn separator_text(separator: Separator) -> &'static str {
    match separator {
        Separator::Tab => "'\\t'",
        Separator::Semicolon => "';'",
        Separator::Comma => "','",
    }
}

/// The format of a file, found from its first bytes and not from its name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// A text file, a CSV or a TSV: any file that is not of the format
    /// xlsx.
    Text,
    /// A zip, which every xlsx is, or a compound file of the old Office,
    /// an `.xls` or an xlsx saved with a password.
    Xlsx,
}

/// A file the import does not read, with what the words of its refusal
/// need. A line is a line of a text file, counted from 1, and a column of
/// a text file its place in the row, from 1; a row and a column of an
/// xlsx are those of the sheet, from 1, column A being 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// A file larger than the caller's limit of bytes.
    TooLarge {
        /// The size of the file, in bytes.
        size: u64,
        /// The caller's limit, [`ImportOptions::max_bytes`].
        max_bytes: u64,
    },
    /// A file of a format whose cargo feature the build leaves out, the
    /// format being that of [`ImportError::Refused`].
    FormatNotBuilt,
    /// A compound file of the old Office that is not an xlsx saved with a
    /// password: a workbook of Excel 97–2003, an `.xls`.
    OldExcel,
    /// An xlsx saved with a password to open it.
    Encrypted,
    /// A zip that holds no workbook, such as a `.docx` or a `.zip` of other
    /// files.
    NotWorkbook,
    /// An xlsx whose first worksheet that is not hidden holds no value.
    EmptySheet {
        /// The name of that worksheet.
        sheet: String,
    },
    /// An xlsx with a cell holding an error calamine does not know.
    CellError {
        /// The text of the error, such as `#GETTING_DATA`.
        error: String,
    },
    /// An xlsx whose rectangle of the values passed the caller's limit of
    /// cells, given as it was when it passed.
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
    /// A text file in UTF-16 that ends in the middle of a character.
    CutShort,
    /// A file with a byte 0 that is not in UTF-16.
    NotText,
    /// A file of variants, a VCF, picked as a table.
    VariantsFile,
    /// A text file with a quote that is never closed.
    UnclosedQuote {
        /// The line where the cell of the quote starts.
        line: u32,
        /// The separator the file was split with.
        separator: Separator,
    },
    /// A cell of the header of an xlsx holding one of the seven errors of
    /// Excel, such as `#VALUE!`.
    HeaderError {
        /// The row of the cell in the sheet.
        row: u32,
        /// The column of the cell in the sheet.
        column: u32,
        /// The error, its spaces at the ends removed.
        error: String,
    },
    /// A file with no row below its header.
    Empty,
    /// A column with no name in the header and a value in some row.
    UnnamedColumn {
        /// The column: its place in the row of a text file, its column of
        /// the sheet in an xlsx.
        column: u32,
    },
    /// A row of a text file with another number of cells than the header.
    RaggedRow {
        /// The line of the row.
        line: u32,
        /// The number of cells of the header.
        expected: u32,
        /// The number of cells of the row.
        found: u32,
        /// The separator the file was split with.
        separator: Separator,
    },
    /// Two columns of one name, the first name that repeats one before it
    /// in the header, read from left to right.
    DuplicateColumn {
        /// The name.
        name: String,
        /// The column of its first use.
        first_column: u32,
        /// The column that repeats it.
        second_column: u32,
    },
    /// A row whose first cell, the name of its individual, is empty.
    EmptyIndividual {
        /// The line of a text file, or the row of the sheet of an xlsx.
        row: u32,
    },
    /// An individual named in two rows.
    DuplicateIndividual {
        /// The name of the individual.
        name: String,
        /// The line or the row of the sheet of its first row.
        first_row: u32,
        /// The line or the row of the sheet of the row that repeats it.
        second_row: u32,
    },
}

/// Reads the table of the file `bytes`, a CSV, a TSV or an xlsx, with the
/// limits and the options of a text file of `options`.
///
/// The format is found from the first bytes, not from the name of the
/// file: one that starts with the bytes of the first part of a zip,
/// `50 4B 03 04`, or of a compound file of the old Office,
/// `D0 CF 11 E0 A1 B1 1A E1`, is an xlsx, and any other a text file, the
/// empty file and a CSV whose header starts with `PK` among them.
///
/// # Errors
///
/// [`ImportError::Refused`], with the format found, for the first of the
/// refusals of `docs/specs/import.md`, "The refusals", in this order:
///
/// 1. a file larger than `options.max_bytes`, [`Refusal::TooLarge`];
/// 2. a file of a format whose feature the build leaves out,
///    [`Refusal::FormatNotBuilt`];
/// 3. the refusals of an xlsx before its rows: [`Refusal::OldExcel`],
///    [`Refusal::Encrypted`], [`Refusal::NotWorkbook`] for a zip whose
///    package relationships, read before any other part, are missing or
///    name no workbook in it, then the first in the order of the file of
///    [`Refusal::CellError`] and [`Refusal::SheetTooLarge`], and
///    [`Refusal::EmptySheet`] once every cell is read; or those of a text
///    file before its rows, [`Refusal::CutShort`], [`Refusal::NotText`],
///    [`Refusal::VariantsFile`], then [`Refusal::UnclosedQuote`];
/// 4. [`Refusal::HeaderError`], an error of Excel in the header of an
///    xlsx, the first by position;
/// 5. no row below the header, [`Refusal::Empty`];
/// 6. [`Refusal::UnnamedColumn`] for a column with no name and a value in
///    the run of empty cells at the end of the header, the first by
///    position;
/// 7. [`Refusal::RaggedRow`], the first by line;
/// 8. [`Refusal::UnnamedColumn`] for a column with no name and a value
///    elsewhere, then [`Refusal::DuplicateColumn`], the first by position;
/// 9. then, row by row in the order of the file,
///    [`Refusal::EmptyIndividual`] or [`Refusal::DuplicateIndividual`].
///
/// [`ImportError::Unreadable`] for an xlsx that cannot be read, with the
/// messages of `docs/specs/read.md`; and, in this version, for every text
/// file that is not refused before its rows, since it is not read yet:
/// "the reading of a text file is not built yet".
pub fn import_table(bytes: &[u8], options: &ImportOptions) -> Result<Table, ImportError> {
    let format = format_of(bytes);
    // A slice holds at most isize::MAX bytes, which a u64 holds on every
    // platform Rust builds for; the saturation is never reached.
    let size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if size > options.max_bytes {
        return Err(ImportError::Refused {
            format,
            refusal: Refusal::TooLarge {
                size,
                max_bytes: options.max_bytes,
            },
        });
    }
    match format {
        Format::Xlsx => table_of_xlsx(bytes, options),
        Format::Text => table_of_text(bytes, options),
    }
}

/// The eight bytes every compound file of the old Office starts with, an
/// `.xls` and an xlsx saved with a password among them.
pub(crate) const COMPOUND_FILE_MARK: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// The four bytes that start a zip with a part, `PK` and the bytes 3 and
/// 4, which open the record of its first part. An empty zip starts with
/// `PK` and the bytes 5 and 6, and is a text file for the import.
pub(crate) const ZIP_MARK: [u8; 4] = [b'P', b'K', 3, 4];

/// The format of `bytes` by their first bytes ("The format" of
/// `docs/specs/import.md`).
fn format_of(bytes: &[u8]) -> Format {
    if bytes.starts_with(&ZIP_MARK) || bytes.starts_with(&COMPOUND_FILE_MARK) {
        Format::Xlsx
    } else {
        Format::Text
    }
}

/// The table of the xlsx `bytes`, a zip or a compound file of the old
/// Office, within `options.max_cells`.
#[cfg(feature = "xlsx")]
fn table_of_xlsx(bytes: &[u8], options: &ImportOptions) -> Result<Table, ImportError> {
    let sheet = crate::xlsx::import_sheet(bytes, options.max_cells)?;
    let crate::xlsx::Sheet {
        name,
        first_row,
        first_column,
        num_rows,
        num_columns,
        cells,
    } = sheet;
    let row_ends = row_ends_of_rectangle(first_row, num_rows, num_columns, cells.len())?;
    // Each cell of the sheet made a cell of the table in the same buffer,
    // the two types being of one size and alignment, so that the rows take
    // no second copy of the slots of the sheet (docs/architecture.md,
    // section 6).
    let cells: Vec<crate::table::Cell<'static>> =
        cells.into_iter().map(cell_of_sheet_cell).collect();
    crate::table::table_of_rows(
        crate::table::Rows {
            origin: crate::table::Origin::Xlsx,
            first_column,
            cells,
            row_ends,
        },
        HowRead::Xlsx { sheet: name },
    )
}

/// The message of [`ImportError::Unreadable`] for a sheet whose rectangle
/// does not hold its cells, or passes row 4,294,967,295, which no sheet
/// read by `docs/specs/read.md` does.
#[cfg(feature = "xlsx")]
const RECTANGLE_NOT_ITS_CELLS: &str = "the rectangle of the sheet does not hold its cells";

/// Where each row of the rectangle of a sheet ends among its `num_cells`
/// cells, `num_rows` rows of `num_columns` from the row `first_row` of the
/// sheet, each with its row.
///
/// # Errors
///
/// [`ImportError::Unreadable`] when `num_cells` is not `num_rows` ×
/// `num_columns`, or a row passes 4,294,967,295.
#[cfg(feature = "xlsx")]
fn row_ends_of_rectangle(
    first_row: u32,
    num_rows: u32,
    num_columns: u32,
    num_cells: usize,
) -> Result<Vec<crate::table::RowEnd>, ImportError> {
    let not_its_cells = || ImportError::Unreadable(RECTANGLE_NOT_ITS_CELLS.to_owned());
    if u64::from(num_rows).checked_mul(u64::from(num_columns)) != u64::try_from(num_cells).ok() {
        return Err(not_its_cells());
    }
    let row_length = usize::try_from(num_columns).map_err(|_| not_its_cells())?;
    let mut row_ends = Vec::new();
    let mut end: usize = 0;
    for row_offset in 0..num_rows {
        end = end.checked_add(row_length).ok_or_else(not_its_cells)?;
        row_ends.push(crate::table::RowEnd {
            place: first_row
                .checked_add(row_offset)
                .ok_or_else(not_its_cells)?,
            end,
        });
    }
    Ok(row_ends)
}

/// The cell of the table of a cell of the sheet.
#[cfg(feature = "xlsx")]
fn cell_of_sheet_cell(sheet_cell: crate::xlsx::SheetCell) -> crate::table::Cell<'static> {
    match sheet_cell {
        crate::xlsx::SheetCell::Empty => crate::table::Cell::Empty,
        crate::xlsx::SheetCell::Text(cell_text) => {
            crate::table::Cell::Text(std::borrow::Cow::Owned(cell_text))
        }
        crate::xlsx::SheetCell::Number(number) => crate::table::Cell::Number(number),
        crate::xlsx::SheetCell::Bool(is_true) => crate::table::Cell::Boolean(is_true),
    }
}

/// The refusal of an xlsx in a build without the feature `xlsx`.
#[cfg(not(feature = "xlsx"))]
fn table_of_xlsx(_bytes: &[u8], _options: &ImportOptions) -> Result<Table, ImportError> {
    Err(ImportError::Refused {
        format: Format::Xlsx,
        refusal: Refusal::FormatNotBuilt,
    })
}

/// The message of [`ImportError::Unreadable`] for a text file, until the
/// module `csv` reads it.
#[cfg(feature = "csv")]
const TEXT_NOT_BUILT: &str = "the reading of a text file is not built yet";

/// The table of the text file `bytes`, with `options.text`.
#[cfg(feature = "csv")]
fn table_of_text(_bytes: &[u8], _options: &ImportOptions) -> Result<Table, ImportError> {
    Err(ImportError::Unreadable(TEXT_NOT_BUILT.to_owned()))
}

/// The refusal of a text file in a build without the feature `csv`.
#[cfg(not(feature = "csv"))]
fn table_of_text(_bytes: &[u8], _options: &ImportOptions) -> Result<Table, ImportError> {
    Err(ImportError::Refused {
        format: Format::Text,
        refusal: Refusal::FormatNotBuilt,
    })
}

#[cfg(all(test, feature = "xlsx"))]
mod tests {
    use crate::table::Cell;
    use crate::xlsx::SheetCell;

    // The cells of a sheet are made the cells of the table in their own
    // buffer, which Rust's collect into a Vec does only for two types of
    // one size and alignment; were they to differ, the rows would take a
    // second copy of the slots of the sheet.
    #[test]
    fn a_cell_of_the_table_has_the_size_and_alignment_of_a_cell_of_the_sheet() {
        assert_eq!(
            (size_of::<Cell<'static>>(), align_of::<Cell<'static>>()),
            (size_of::<SheetCell>(), align_of::<SheetCell>())
        );
    }
}
