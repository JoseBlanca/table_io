//! Reads a table from the bytes of a CSV, a TSV or an xlsx with
//! [`import_table`], as `docs/specs/import.md` gives it, and gives what
//! `docs/specs/values.md` gives: whether a text is missing, the whole
//! number, the number or the boolean it holds, the text of a float as
//! JavaScript writes it, the four types of a column, and the conversion of
//! a column to another type.
//!
//! The reading of an xlsx is behind the cargo feature `xlsx`, with
//! calamine, zip and quick-xml, so that a build without it compiles none
//! of them, and that of a text file behind the feature `csv`
//! (`docs/architecture.md`, section 5); a build without the feature of a
//! file's format refuses the file. The import, its options and its
//! refusals, the values and their types are behind no feature. Both
//! features are on by default.

#![forbid(unsafe_code)]

#[cfg_attr(
    not(feature = "xlsx"),
    expect(
        dead_code,
        reason = "the table of a text file is made in work package 4 of docs/plans/table-io.md; until then only an xlsx is"
    )
)]
mod guess;
mod import;
#[cfg_attr(
    not(feature = "xlsx"),
    expect(
        dead_code,
        reason = "the table of a text file is made in work package 4 of docs/plans/table-io.md; until then only an xlsx is"
    )
)]
mod table;
mod text_options;
mod types;
mod value;

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
