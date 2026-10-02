//! The export of a table, `docs/specs/export.md`: the types an application
//! gives and gets, and `export_table`, which refuses a table that would not
//! read back as itself and writes the others with the module of their
//! format. It is behind no feature.

use crate::{Column, DecimalMark, NameColumn, Separator};

/// The format of the file an export writes, with the choices of a CSV.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// A CSV, or a TSV when the separator is the tab.
    Csv(CsvExport),
    /// An xlsx of one sheet, `Sheet1`.
    Xlsx,
}

/// The choices of a CSV, each set by the caller; every combination is
/// accepted, the comma as both the separator and the decimal mark among
/// them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CsvExport {
    /// The character between the cells of a row.
    pub separator: Separator,
    /// The mark between the whole part and the decimals of a float.
    pub decimal: DecimalMark,
    /// The encoding of the bytes.
    pub encoding: CsvEncoding,
    /// How a missing value is written.
    pub missing: MissingText,
}

/// The encoding a CSV is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsvEncoding {
    /// UTF-8, with no mark.
    Utf8,
    /// UTF-8 after its mark, the bytes `EF BB BF`, as Excel writes "CSV
    /// UTF-8".
    Utf8WithMark,
    /// Windows-1252, what Excel on Windows writes for "CSV".
    Windows1252,
}

/// How a missing value is written in a CSV.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingText {
    /// As an empty cell.
    Empty,
    /// As `NA`.
    Na,
}

/// A cell of the file written: the header is row 0, the first individual
/// row 1, the name at index `i` of the names' column row `i` + 1; the
/// names' column is column 1, and the column at index `j` of the other
/// columns given to [`export_table`] column `j` + 2.
/// A row or a column past 4,294,967,295 is given as 4,294,967,295.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellPlace {
    /// The column, from 1.
    pub column: u32,
    /// The row, the header being 0.
    pub row: u32,
}

/// A table the export does not write, because it would not read back as
/// itself, with what the words of the refusal need. A row and a column
/// are those of [`CellPlace`], and a count past 4,294,967,295 is given as
/// 4,294,967,295.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportRefusal {
    /// A format whose cargo feature the build leaves out; in this version
    /// an xlsx in every build, since its writer is not there yet.
    FormatNotBuilt,
    /// A table with no individual, a header alone, which the import
    /// refuses as empty.
    NoIndividual,
    /// A column with another number of values than there are names of
    /// individuals, a defect of the caller.
    WrongLength {
        /// The column, of [`CellPlace`]: the column at index `j` of the
        /// other columns is column `j` + 2.
        column: u32,
        /// The number of names of individuals.
        expected: u32,
        /// The number of values of the column.
        found: u32,
    },
    /// An empty name of a column other than the names', or of the names'
    /// column of a table with no other column.
    EmptyName {
        /// The column, of [`CellPlace`]: 1 for the names' column, `j` + 2
        /// for the column at index `j` of the other columns.
        column: u32,
    },
    /// Two columns of one name, the names' column among them: the first
    /// name, from left to right, that repeats one before it.
    DuplicateName {
        /// The name.
        name: String,
        /// The column of its first use.
        first_column: u32,
        /// The column that repeats it.
        second_column: u32,
    },
    /// An empty name of an individual.
    EmptyIndividual {
        /// The row.
        row: u32,
    },
    /// An individual named in two rows.
    DuplicateIndividual {
        /// The name.
        name: String,
        /// The row of its first use.
        first_row: u32,
        /// The row that repeats it.
        second_row: u32,
    },
    /// A text value that the import takes for a missing value: empty, `NA`
    /// or `-`, whatever the text of a missing value of the file is.
    ReadsAsMissing {
        /// The cell.
        place: CellPlace,
    },
    /// In an xlsx, a name that is one of the seven errors of Excel.
    ErrorAsName {
        /// The cell.
        place: CellPlace,
    },
    /// In an xlsx, a name or a text value with a space or a tab at its
    /// start or its end, which the import of an xlsx removes.
    SpacesAtEnds {
        /// The cell.
        place: CellPlace,
    },
    /// In an xlsx, an integer beyond −2^53 to 2^53, which a number of
    /// Excel does not hold exactly.
    IntegerTooLarge {
        /// The cell.
        place: CellPlace,
    },
    /// A float that is infinite or not a number.
    NotFinite {
        /// The cell.
        place: CellPlace,
    },
    /// A character the file cannot carry, never replaced: in a CSV, U+0000,
    /// a U+FEFF at the start of the name of the names' column, and in
    /// Windows-1252 a character it does not have.
    CannotCarry {
        /// The cell.
        place: CellPlace,
        /// The first such character of the cell.
        character: char,
    },
    /// In an xlsx, a text longer than the 32,767 units of UTF-16 a cell of
    /// Excel holds.
    TextTooLong {
        /// The cell.
        place: CellPlace,
        /// Its length in units of UTF-16.
        length: u32,
    },
    /// In an xlsx, more than 1,048,575 rows below the header or more than
    /// 16,384 columns, the names' among them.
    TooLargeForSheet {
        /// The number of rows below the header.
        rows: u32,
        /// The number of columns, the names' among them.
        columns: u32,
    },
    /// In a CSV, a header whose line, as written, after its spaces and tabs
    /// at the start, starts as a variants file's, `##fileformat=VCF` or
    /// `#CHROM`, which the import refuses.
    ReadsAsVariantsFile,
}

/// Why an export gave no file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportError {
    /// A table refused, with what the words of the refusal need.
    Refused(ExportRefusal),
    /// rust_xlsxwriter's message, for whoever reports the problem.
    Failed(String),
}

/// A refusal or a failed export, written for whoever reports the problem,
/// in English and with its fields, such as "an export refused: the text at
/// column 3 and row 2 reads back as a missing value"; the words a user
/// reads are the application's.
impl std::fmt::Display for ExportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Refused(refusal) => {
                write!(formatter, "an export refused: ")?;
                write_refusal(formatter, refusal)
            }
            Self::Failed(message) => write!(formatter, "an export failed: {message}"),
        }
    }
}

impl std::error::Error for ExportError {}

/// Writes `refusal` with its fields, for [`ExportError`]'s `Display`.
fn write_refusal(
    formatter: &mut std::fmt::Formatter<'_>,
    refusal: &ExportRefusal,
) -> std::fmt::Result {
    let place_text =
        |place: &CellPlace| format!("at column {} and row {}", place.column, place.row);
    match refusal {
        ExportRefusal::FormatNotBuilt => {
            write!(formatter, "a format this build of table_io does not write")
        }
        ExportRefusal::NoIndividual => write!(formatter, "a table with no individual"),
        ExportRefusal::WrongLength {
            column,
            expected,
            found,
        } => write!(
            formatter,
            "column {column} has {found} values where there are {expected} individuals"
        ),
        ExportRefusal::EmptyName { column } => {
            write!(formatter, "column {column} has an empty name")
        }
        ExportRefusal::DuplicateName {
            name,
            first_column,
            second_column,
        } => write!(
            formatter,
            "the name '{name}' is used by the columns {first_column} and {second_column}"
        ),
        ExportRefusal::EmptyIndividual { row } => {
            write!(formatter, "the row {row} has no name of an individual")
        }
        ExportRefusal::DuplicateIndividual {
            name,
            first_row,
            second_row,
        } => write!(
            formatter,
            "the individual '{name}' is in the rows {first_row} and {second_row}"
        ),
        ExportRefusal::ReadsAsMissing { place } => write!(
            formatter,
            "the text {} reads back as a missing value",
            place_text(place)
        ),
        ExportRefusal::ErrorAsName { place } => write!(
            formatter,
            "the name {} is an error of Excel",
            place_text(place)
        ),
        ExportRefusal::SpacesAtEnds { place } => write!(
            formatter,
            "the text {} has a space or a tab at its start or its end",
            place_text(place)
        ),
        ExportRefusal::IntegerTooLarge { place } => write!(
            formatter,
            "the integer {} is beyond -2^53 to 2^53, which a number of Excel holds exactly",
            place_text(place)
        ),
        ExportRefusal::NotFinite { place } => write!(
            formatter,
            "the float {} is infinite or not a number",
            place_text(place)
        ),
        ExportRefusal::CannotCarry { place, character } => write!(
            formatter,
            "the character '{character}' (U+{:04X}) {} cannot be carried by the file",
            u32::from(*character),
            place_text(place)
        ),
        ExportRefusal::TextTooLong { place, length } => write!(
            formatter,
            "the text {} is {length} units of UTF-16 long, past the 32,767 of a cell of Excel",
            place_text(place)
        ),
        ExportRefusal::TooLargeForSheet { rows, columns } => write!(
            formatter,
            "{rows} rows below the header and {columns} columns, past the 1,048,575 rows and \
             16,384 columns of a sheet of Excel"
        ),
        ExportRefusal::ReadsAsVariantsFile => {
            write!(formatter, "the header starts as a file of variants, a VCF")
        }
    }
}

/// The bytes of a new file holding the table of the column of the names
/// of the individuals `names` and the other columns `columns`, in
/// `format`, by `docs/specs/export.md`.
///
/// The file has the header, the name of the names' column and then the
/// name of each column, and a row for each individual, its name and then
/// its values; the numbers of the columns in a file they were imported
/// from are not used. A CSV writes an integer as its digits, a float as
/// [`crate::float_text`] writes it with the decimal mark chosen, followed
/// by the mark and a `0` when that text has neither a mark nor an
/// exponent, `1.0`, a boolean `TRUE` or `FALSE`, a text as it is and a
/// missing value as the text chosen; a cell in quotes, a `"` inside
/// doubled, when it holds the separator, a `"`, `\r` or `\n`, or a space
/// or a tab at its start or its end; every line ended by `\r\n`, the last
/// one included.
///
/// # Errors
///
/// [`ExportError::Refused`] for the first of the refusals of
/// `docs/specs/export.md`, "The refusals", in this order; those marked
/// xlsx only an xlsx gives:
///
/// 1. [`ExportRefusal::FormatNotBuilt`]: a CSV in a build without the
///    feature `csv`, and an xlsx in every build of this version;
/// 2. [`ExportRefusal::WrongLength`], the first such column from left to
///    right;
/// 3. [`ExportRefusal::NoIndividual`];
/// 4. [`ExportRefusal::TooLargeForSheet`], xlsx;
/// 5. [`ExportRefusal::ReadsAsVariantsFile`], a CSV only;
/// 6. then the first refusal of a cell in the order of the file, the
///    header and then row by row, each from left to right; of a name of
///    the header, [`ExportRefusal::EmptyName`],
///    [`ExportRefusal::DuplicateName`], [`ExportRefusal::ErrorAsName`]
///    (xlsx); of a name of an individual,
///    [`ExportRefusal::EmptyIndividual`],
///    [`ExportRefusal::DuplicateIndividual`]; of a value,
///    [`ExportRefusal::ReadsAsMissing`], [`ExportRefusal::NotFinite`],
///    [`ExportRefusal::IntegerTooLarge`] (xlsx); and of any text,
///    [`ExportRefusal::SpacesAtEnds`] (xlsx),
///    [`ExportRefusal::TextTooLong`] (xlsx) and
///    [`ExportRefusal::CannotCarry`].
///
/// [`ExportError::Failed`] for an error of rust_xlsxwriter, which the
/// checks should leave none of; a CSV never gives it.
pub fn export_table(
    names: &NameColumn,
    columns: &[Column],
    format: &ExportFormat,
) -> Result<Vec<u8>, ExportError> {
    let is_built = match format {
        ExportFormat::Csv(_) => cfg!(feature = "csv"),
        ExportFormat::Xlsx => false,
    };
    if !is_built {
        return Err(ExportError::Refused(ExportRefusal::FormatNotBuilt));
    }
    // The shape is checked here, for every format, so that no writer
    // meets a column shorter than the names, whose values past its end
    // would be written as missing.
    if let Some(refusal) = crate::export_cells::shape_refusal(names, columns) {
        return Err(ExportError::Refused(refusal));
    }
    match format {
        ExportFormat::Csv(csv_export) => csv_file(names, columns, csv_export),
        ExportFormat::Xlsx => Err(ExportError::Refused(ExportRefusal::FormatNotBuilt)),
    }
}

/// The bytes of the CSV of the table, whose shape is checked, with the
/// choices `csv_export`.
#[cfg(feature = "csv")]
fn csv_file(
    names: &NameColumn,
    columns: &[Column],
    csv_export: &CsvExport,
) -> Result<Vec<u8>, ExportError> {
    crate::csv::csv_of_table(names, columns, csv_export)
}

/// The refusal of a CSV in a build without the feature `csv`, which
/// [`export_table`] gives before it calls this.
#[cfg(not(feature = "csv"))]
fn csv_file(
    _names: &NameColumn,
    _columns: &[Column],
    _csv_export: &CsvExport,
) -> Result<Vec<u8>, ExportError> {
    Err(ExportError::Refused(ExportRefusal::FormatNotBuilt))
}
