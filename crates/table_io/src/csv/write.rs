//! A table written as a CSV, "A CSV" of `docs/specs/export.md`: each value
//! as its text, a cell in quotes when the import would otherwise split it
//! or remove its spaces, every line ended by `\r\n`, and the text encoded
//! in UTF-8, with its mark or without, or in Windows-1252 by the table the
//! import decodes it with, a character that cannot be carried refused and
//! never replaced.

use super::{BYTE_ORDER_MARK, UTF8_MARK, WINDOWS_1252_80_TO_9F, is_variants_file, separator_byte};
use crate::export_cells::{CellWriter, ExportValue, refused, write_cells};
use crate::types::boolean_text;
use crate::value::float_text;
use crate::{
    CellPlace, Column, CsvEncoding, CsvExport, DecimalMark, ExportError, ExportRefusal,
    MissingText, NameColumn, Separator,
};

/// The bytes of the CSV of the table of `names` and `columns`, whose shape
/// is checked, with the choices `csv_export`.
///
/// # Errors
///
/// [`ExportError::Refused`] for [`ExportRefusal::ReadsAsVariantsFile`],
/// a header whose line starts as a variants file's, then for the first
/// refusal of a cell of [`write_cells`], or of a character the encoding
/// cannot carry, [`ExportRefusal::CannotCarry`], in the order of the
/// file.
pub(crate) fn csv_of_table(
    names: &NameColumn,
    columns: &[Column],
    csv_export: &CsvExport,
) -> Result<Vec<u8>, ExportError> {
    let header_line = header_line(names, columns, csv_export.separator);
    // The import removes the marks at the start of the text before it
    // looks for a variants file.
    if is_variants_file(header_line.trim_start_matches(BYTE_ORDER_MARK)) {
        return Err(refused(ExportRefusal::ReadsAsVariantsFile));
    }
    let mut writer = CsvWriter {
        bytes: match csv_export.encoding {
            CsvEncoding::Utf8WithMark => UTF8_MARK.to_vec(),
            CsvEncoding::Utf8 | CsvEncoding::Windows1252 => Vec::new(),
        },
        csv_export: *csv_export,
        is_row_start: true,
    };
    write_cells(names, columns, &mut writer)?;
    Ok(writer.bytes)
}

/// The line of the header as it is written, its cells quoted, before it is
/// encoded and with no line break.
fn header_line(names: &NameColumn, columns: &[Column], separator: Separator) -> String {
    let separator_character = char::from(separator_byte(separator));
    let mut line = String::new();
    let header_names = std::iter::once(names.header.as_str())
        .chain(columns.iter().map(|column| column.name.as_str()));
    for (index, name) in header_names.enumerate() {
        if index > 0 {
            line.push(separator_character);
        }
        if needs_quotes(name, separator) {
            line.push('"');
            line.push_str(&name.replace('"', "\"\""));
            line.push('"');
        } else {
            line.push_str(name);
        }
    }
    line
}

/// Whether a cell of `text` is written in quotes with `separator`: when it
/// holds the separator, a `"`, `\r` or `\n`, or starts or ends with a
/// space or a tab, which the import removes outside quotes, a tab at the
/// ends being the separator itself when the separator is the tab.
fn needs_quotes(text: &str, separator: Separator) -> bool {
    let separator_character = char::from(separator_byte(separator));
    text.contains([separator_character, '"', '\r', '\n'])
        || text.starts_with([' ', '\t'])
        || text.ends_with([' ', '\t'])
}

/// The text of a float in a CSV: [`float_text`] with `decimal`, followed by
/// the mark and a `0` when it has neither a mark nor an exponent, so that
/// it reads back as a float and not as an integer, `1.0`.
fn float_cell_text(float: f64, decimal: DecimalMark) -> String {
    let mark = decimal.character();
    let mut text = float_text(float, decimal);
    if !text.contains([mark, 'e']) {
        text.push(mark);
        text.push('0');
    }
    text
}

/// The byte of `character` in Windows-1252, by the table the import
/// decodes it with, or None for a character it does not have: a character
/// of U+0000 to U+00FF is the byte of its number but those of U+0080 to
/// U+009F, which are the bytes of `80` to `9F` only where the table gives
/// them, the five controls it keeps; the others of the table, `€` among
/// them, are their bytes of `80` to `9F`.
fn windows_1252_byte(character: char) -> Option<u8> {
    match u8::try_from(u32::from(character)) {
        Ok(byte) if !(0x80..=0x9F).contains(&byte) => Some(byte),
        Ok(_) | Err(_) => WINDOWS_1252_80_TO_9F
            .iter()
            .position(|&table_character| table_character == character)
            .and_then(|offset| u8::try_from(offset).ok())
            .and_then(|offset| offset.checked_add(0x80)),
    }
}

/// The writer of the cells of a CSV into its bytes.
struct CsvWriter {
    /// The bytes written so far, the mark of UTF-8 first when it has one.
    bytes: Vec<u8>,
    /// The choices of the CSV.
    csv_export: CsvExport,
    /// Whether the next cell is the first of its row, with no separator
    /// before it.
    is_row_start: bool,
}

impl CsvWriter {
    /// Writes the separator unless the cell is the first of its row.
    fn start_cell(&mut self) {
        if !self.is_row_start {
            self.bytes.push(separator_byte(self.csv_export.separator));
        }
        self.is_row_start = false;
    }

    /// Writes a cell of ASCII that never needs quotes: the digits of an
    /// integer, a boolean or the text of a missing value.
    fn write_plain(&mut self, text: &str) {
        self.start_cell();
        self.bytes.extend_from_slice(text.as_bytes());
    }

    /// Writes a cell of `text` at `place`, in quotes when it needs them.
    ///
    /// # Errors
    ///
    /// [`ExportError::Refused`] for [`ExportRefusal::CannotCarry`], its
    /// first character the encoding cannot carry, U+0000 in any.
    fn write_text(&mut self, text: &str, place: CellPlace) -> Result<(), ExportError> {
        self.start_cell();
        let is_quoted = needs_quotes(text, self.csv_export.separator);
        if is_quoted {
            self.bytes.push(b'"');
        }
        for character in text.chars() {
            if character == '\0' {
                return Err(refused(ExportRefusal::CannotCarry { place, character }));
            }
            match self.csv_export.encoding {
                CsvEncoding::Utf8 | CsvEncoding::Utf8WithMark => {
                    let mut buffer = [0_u8; 4];
                    self.bytes
                        .extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
                }
                CsvEncoding::Windows1252 => {
                    let byte = windows_1252_byte(character)
                        .ok_or(refused(ExportRefusal::CannotCarry { place, character }))?;
                    self.bytes.push(byte);
                }
            }
            if is_quoted && character == '"' {
                self.bytes.push(b'"');
            }
        }
        if is_quoted {
            self.bytes.push(b'"');
        }
        Ok(())
    }
}

impl CellWriter for CsvWriter {
    fn header_name(&mut self, name: &str, place: CellPlace) -> Result<(), ExportError> {
        // The import removes a mark at the start of the text, which the
        // first name starts.
        if place.column == 1 && name.starts_with(BYTE_ORDER_MARK) {
            return Err(refused(ExportRefusal::CannotCarry {
                place,
                character: BYTE_ORDER_MARK,
            }));
        }
        self.write_text(name, place)
    }

    fn individual_name(&mut self, name: &str, place: CellPlace) -> Result<(), ExportError> {
        self.write_text(name, place)
    }

    fn value(&mut self, value: ExportValue<'_>, place: CellPlace) -> Result<(), ExportError> {
        match value {
            ExportValue::Missing => match self.csv_export.missing {
                MissingText::Empty => self.write_plain(""),
                MissingText::Na => self.write_plain("NA"),
            },
            ExportValue::Integer(integer) => self.write_plain(&integer.to_string()),
            ExportValue::Float(float) => {
                return self.write_text(&float_cell_text(float, self.csv_export.decimal), place);
            }
            ExportValue::Boolean(is_true) => self.write_plain(&boolean_text(is_true)),
            ExportValue::Text(text) => return self.write_text(text, place),
        }
        Ok(())
    }

    fn row_end(&mut self) {
        self.bytes.extend_from_slice(b"\r\n");
        self.is_row_start = true;
    }
}

#[cfg(test)]
mod tests {
    use super::windows_1252_byte;
    use crate::csv::windows_1252_character;

    #[test]
    fn every_byte_of_windows_1252_but_0_is_written_back_from_the_character_it_is_read_as() {
        for byte in 1..=u8::MAX {
            assert_eq!(
                windows_1252_byte(windows_1252_character(byte)),
                Some(byte),
                "{byte:02X}"
            );
        }
    }

    #[test]
    fn a_control_of_80_to_9f_that_windows_1252_does_not_keep_has_no_byte() {
        assert_eq!(windows_1252_byte('\u{80}'), None);
        assert_eq!(windows_1252_byte('\u{9F}'), None);
        assert_eq!(windows_1252_byte('ő'), None);
        assert_eq!(windows_1252_byte('😀'), None);
    }
}
