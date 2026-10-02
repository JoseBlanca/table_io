//! Reads a table from the bytes of a CSV, a TSV or an xlsx with
//! [`import_table`], as `docs/specs/import.md` gives it, and gives what
//! `docs/specs/values.md` gives: whether a text is missing, the whole
//! number, the number or the boolean it holds, the text of a float as
//! JavaScript writes it, the four types of a column, and the conversion of
//! a column to another type. It writes a table as a new CSV with
//! [`export_table`], as `docs/specs/export.md` gives it, or refuses a
//! table that would not read back as itself.
//!
//! The reading of an xlsx is behind the cargo feature `xlsx`, with
//! calamine, zip and quick-xml, so that a build without it compiles none
//! of them, and the reading and the writing of a text file behind the
//! feature `csv` (`docs/architecture.md`, section 5); a build without the
//! feature of a file's format refuses the file, and the export of it. The
//! import and the export, their options and their refusals, the values
//! and their types are behind no feature. Both features are on by
//! default.

#![forbid(unsafe_code)]

mod export;
#[cfg_attr(
    not(feature = "csv"),
    expect(dead_code, reason = "a build that writes no CSV writes no cell")
)]
mod export_cells;
#[cfg_attr(
    not(any(feature = "csv", feature = "xlsx")),
    expect(dead_code, reason = "a build that reads no format makes no table")
)]
mod guess;
mod import;
#[cfg_attr(
    not(any(feature = "csv", feature = "xlsx")),
    expect(dead_code, reason = "a build that reads no format makes no table")
)]
mod table;
mod text_options;
mod types;
mod value;

pub use crate::export::{
    CellPlace, CsvEncoding, CsvExport, ExportError, ExportFormat, ExportRefusal, MissingText,
    export_table,
};
pub use crate::import::{
    Column, Format, HowRead, ImportError, ImportOptions, NameColumn, Refusal, Table, import_table,
};
pub use crate::text_options::{Encoding, FoundEncoding, Separator, TextOptions, TextRead};
pub use crate::types::{ColumnType, ColumnValues, ConversionFailure, convert_column};
pub use crate::value::{
    DecimalMark, float_text, is_missing, parse_boolean, parse_float, parse_integer,
};

#[cfg(feature = "xlsx")]
mod attrs;
#[cfg(feature = "xlsx")]
#[cfg(test)]
mod bounds_tests;
#[cfg(feature = "xlsx")]
mod cell;
#[cfg(feature = "csv")]
mod csv;
#[cfg(feature = "xlsx")]
mod date;
#[cfg(feature = "xlsx")]
mod encoding;
#[cfg(feature = "xlsx")]
mod parts;
#[cfg(feature = "xlsx")]
mod rectangle;
#[cfg(feature = "xlsx")]
mod xlsx;
#[cfg(feature = "xlsx")]
#[cfg(test)]
mod xlsx_tests;
