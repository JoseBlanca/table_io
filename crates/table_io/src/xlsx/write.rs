//! A table written as an xlsx with rust_xlsxwriter, "An xlsx" of
//! `docs/specs/export.md`: one sheet, `Sheet1`, the names in the first
//! row from A1, a row for each individual below, a float and an integer
//! as a number cell, a boolean as a boolean cell, a text as a text cell
//! and a missing value as no cell; with the refusals only an xlsx gives,
//! of what its sheet or its cells cannot hold or what its import reads as
//! another value.
//!
//! rust_xlsxwriter holds the whole sheet in memory until it is saved: its
//! mode of constant memory writes the sheet to a file as it goes, and the
//! library uses no file system.

use rust_xlsxwriter::{ColNum, RowNum, Workbook, Worksheet, XlsxError};

use crate::export_cells::{CellWriter, ExportValue, count_of, refused, write_cells};
use crate::table::is_excel_error;
use crate::{CellPlace, Column, ExportError, ExportRefusal, NameColumn};

/// The rows of a sheet of Excel below the header, its 1,048,576 rows less
/// the header's.
const MAX_ROWS: u32 = 1_048_575;

/// The columns of a sheet of Excel, A to XFD.
const MAX_COLUMNS: u32 = 16_384;

/// The most units of UTF-16 a cell of Excel holds, by the limit of Excel's
/// specifications, 32,767 characters, counted as the units of UTF-16, the
/// stricter count (`docs/specs/export.md`, "The refusals").
const MAX_TEXT_UNITS: usize = 32_767;

/// 2^53, up to which a float holds every whole number exactly.
const LARGEST_EXACT_INTEGER: i64 = 9_007_199_254_740_992;

/// The message of [`ExportError::Failed`] for a place past the sheet,
/// which the refusal of a table larger than a sheet leaves none of.
const PLACE_PAST_SHEET: &str = "a cell past the last row or column of a sheet";

/// The bytes of the xlsx of the table of `names` and `columns`, whose
/// shape is checked.
///
/// # Errors
///
/// [`ExportError::Refused`] for [`ExportRefusal::TooLargeForSheet`], more
/// rows or columns than a sheet has, then for the first refusal of a cell
/// of [`write_cells`] or of an xlsx, in the order of the file;
/// [`ExportError::Failed`] for an error of rust_xlsxwriter.
pub(crate) fn xlsx_of_table(
    names: &NameColumn,
    columns: &[Column],
) -> Result<Vec<u8>, ExportError> {
    let rows = count_of(names.names.len());
    let num_columns = count_of(columns.len()).saturating_add(1);
    if rows > MAX_ROWS || num_columns > MAX_COLUMNS {
        return Err(refused(ExportRefusal::TooLargeForSheet {
            rows,
            columns: num_columns,
        }));
    }
    let mut workbook = Workbook::new();
    // rust_xlsxwriter names its first sheet `Sheet1`.
    let worksheet = workbook.add_worksheet();
    write_cells(names, columns, &mut XlsxWriter { worksheet })?;
    workbook.save_to_buffer().map_err(failed)
}

/// The error of an export for an error of rust_xlsxwriter, its message
/// kept as text.
fn failed(error: XlsxError) -> ExportError {
    ExportError::Failed(error.to_string())
}

/// The row and the column of rust_xlsxwriter, from 0, of `place`.
///
/// # Errors
///
/// [`ExportError::Failed`] for a column past the sheet's.
fn sheet_place(place: CellPlace) -> Result<(RowNum, ColNum), ExportError> {
    place
        .column
        .checked_sub(1)
        .and_then(|column| ColNum::try_from(column).ok())
        .map(|column| (place.row, column))
        .ok_or_else(|| ExportError::Failed(PLACE_PAST_SHEET.to_owned()))
}

/// The refusal of a text, a name or a value, that an xlsx does not hold
/// as itself, or None, the first of: a space or a tab at its start or its
/// end, which the import removes, [`ExportRefusal::SpacesAtEnds`]; more
/// units of UTF-16 than a cell holds, [`ExportRefusal::TextTooLong`]; and
/// a character [`uncarried_character`] finds, [`ExportRefusal::CannotCarry`].
fn text_refusal(text: &str, place: CellPlace) -> Option<ExportRefusal> {
    let is_space_or_tab = |character: char| character == ' ' || character == '\t';
    if text.starts_with(is_space_or_tab) || text.ends_with(is_space_or_tab) {
        return Some(ExportRefusal::SpacesAtEnds { place });
    }
    // A text has no more units of UTF-16 than bytes of UTF-8.
    if text.len() > MAX_TEXT_UNITS {
        let length = text.encode_utf16().count();
        if length > MAX_TEXT_UNITS {
            return Some(ExportRefusal::TextTooLong {
                place,
                length: count_of(length),
            });
        }
    }
    uncarried_character(text).map(|character| ExportRefusal::CannotCarry { place, character })
}

/// The first character of `text` that an xlsx does not carry, or None:
/// U+FFFE or U+FFFF, which rust_xlsxwriter writes as `_xFFFE_` and
/// calamine reads back as those seven characters; and a control
/// rust_xlsxwriter writes as `_xHHHH_`, [`is_escaped_control`], right
/// after `_x` and four digits of hexadecimal, which the `_` of its escape
/// would make an escape too, `_x0041` and `\r` read back as `Ax000D_`.
/// The second is refused until the owner decides
/// (`docs/specs/export.md`, "The refusals").
fn uncarried_character(text: &str) -> Option<char> {
    text.char_indices().find_map(|(index, character)| {
        let is_uncarried = character == '\u{FFFE}'
            || character == '\u{FFFF}'
            || (is_escaped_control(character)
                && ends_with_escape_digits(text.get(..index).unwrap_or_default()));
        is_uncarried.then_some(character)
    })
}

/// Whether rust_xlsxwriter 0.99.1 writes `character` as `_xHHHH_`: the
/// controls U+0000 to U+0008 and U+000B to U+001F, its `match_xml_char`;
/// U+FFFE and U+FFFF, which it writes so too, are refused whatever comes
/// before them.
fn is_escaped_control(character: char) -> bool {
    matches!(character, '\u{0}'..='\u{8}' | '\u{B}'..='\u{1F}')
}

/// Whether `text` ends with `_x` and four digits of hexadecimal, in either
/// case.
fn ends_with_escape_digits(text: &str) -> bool {
    match text.as_bytes().last_chunk::<6>() {
        Some([b'_', b'x', digits @ ..]) => digits.iter().all(u8::is_ascii_hexdigit),
        Some(_) | None => false,
    }
}

/// The float of `integer`, or None for one beyond −2^53 to 2^53, which a
/// number of Excel does not hold exactly.
fn exact_float(integer: i64) -> Option<f64> {
    if !(-LARGEST_EXACT_INTEGER..=LARGEST_EXACT_INTEGER).contains(&integer) {
        return None;
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "an integer from -2^53 to 2^53, checked above, is a float exactly"
    )]
    let float = integer as f64;
    Some(float)
}

/// The writer of the cells of an xlsx into the sheet of rust_xlsxwriter.
struct XlsxWriter<'book> {
    /// The sheet, `Sheet1`.
    worksheet: &'book mut Worksheet,
}

impl XlsxWriter<'_> {
    /// Writes `text`, not empty, as a text cell at `place`, after the
    /// refusals of [`text_refusal`].
    ///
    /// # Errors
    ///
    /// The refusal of the text, or a failure of rust_xlsxwriter.
    fn write_text(&mut self, text: &str, place: CellPlace) -> Result<(), ExportError> {
        if let Some(refusal) = text_refusal(text, place) {
            return Err(refused(refusal));
        }
        let (row, column) = sheet_place(place)?;
        self.worksheet
            .write_string(row, column, text)
            .map_err(failed)?;
        Ok(())
    }
}

impl CellWriter for XlsxWriter<'_> {
    fn header_name(&mut self, name: &str, place: CellPlace) -> Result<(), ExportError> {
        if is_excel_error(name) {
            return Err(refused(ExportRefusal::ErrorAsName { place }));
        }
        // An empty name of the names' column is no cell.
        if name.is_empty() {
            return Ok(());
        }
        self.write_text(name, place)
    }

    fn individual_name(&mut self, name: &str, place: CellPlace) -> Result<(), ExportError> {
        self.write_text(name, place)
    }

    fn value(&mut self, value: ExportValue<'_>, place: CellPlace) -> Result<(), ExportError> {
        let (row, column) = sheet_place(place)?;
        match value {
            ExportValue::Missing => return Ok(()),
            ExportValue::Integer(integer) => {
                let float = exact_float(integer)
                    .ok_or(refused(ExportRefusal::IntegerTooLarge { place }))?;
                self.worksheet
                    .write_number(row, column, float)
                    .map_err(failed)?;
            }
            ExportValue::Float(float) => {
                self.worksheet
                    .write_number(row, column, float)
                    .map_err(failed)?;
            }
            ExportValue::Boolean(is_true) => {
                self.worksheet
                    .write_boolean(row, column, is_true)
                    .map_err(failed)?;
            }
            ExportValue::Text(text) => {
                // The import of an xlsx takes an error of Excel in a
                // column for a missing value.
                if is_excel_error(text) {
                    return Err(refused(ExportRefusal::ReadsAsMissing { place }));
                }
                return self.write_text(text, place);
            }
        }
        Ok(())
    }

    fn row_end(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::{exact_float, sheet_place};
    use crate::CellPlace;

    #[test]
    fn an_integer_is_its_float_from_minus_2_to_the_53_to_2_to_the_53_and_none_beyond() {
        assert_eq!(
            exact_float(9_007_199_254_740_992),
            Some(9_007_199_254_740_992.0)
        );
        assert_eq!(
            exact_float(-9_007_199_254_740_992),
            Some(-9_007_199_254_740_992.0)
        );
        assert_eq!(exact_float(-3), Some(-3.0));
        assert_eq!(exact_float(9_007_199_254_740_993), None);
        assert_eq!(exact_float(-9_007_199_254_740_993), None);
    }

    #[test]
    fn a_place_is_the_row_as_it_is_and_the_column_less_1_and_none_past_the_sheet() {
        assert_eq!(sheet_place(CellPlace { column: 1, row: 0 }), Ok((0, 0)));
        assert_eq!(sheet_place(CellPlace { column: 3, row: 7 }), Ok((7, 2)));
        assert!(sheet_place(CellPlace { column: 0, row: 0 }).is_err());
        assert!(
            sheet_place(CellPlace {
                column: 65_537,
                row: 0
            })
            .is_err()
        );
    }
}
