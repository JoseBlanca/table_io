//! The binding of table_io to JavaScript, `docs/specs/package.md`: the
//! import of a table, `importTable`, the conversion of a column,
//! `convertColumn`, and the rules of a value, each a thin wrapper over the
//! function of the library crate of the same name.
//!
//! It is compiled for `wasm32-unknown-unknown`, and the `wasm-bindgen`
//! command line writes from it the JavaScript and the declarations of the
//! package `js/table_io`. It holds no rule about a value: it turns the
//! strings of the options into the library's types, throwing an `Error` for
//! one that is not among them, and copies what the library gives into
//! [`TableRead`] and [`Conversion`], a refusal and a file that cannot be
//! read as the code of their kind and the fields its words need.
//!
//! wasm-bindgen copies the doc comments of the exported items into the
//! declarations, `wasm/table_io.d.ts`, which the test of the package
//! compares with `js/table_io/test/table_io.d.ts`: a change of one of them
//! is a change of that file too.

use table_io::{
    ColumnType, ColumnValues, ConversionFailure, DecimalMark, Encoding, Format, FoundEncoding,
    HowRead, ImportError, ImportOptions, Refusal, Separator, Table, TextOptions,
};
use wasm_bindgen::prelude::{JsError, wasm_bindgen};

/// The largest limit of bytes JavaScript can give exactly, 2^53: a number
/// of JavaScript holds every whole number up to it.
const MAX_JS_WHOLE_NUMBER: f64 = 9_007_199_254_740_992.0;

/// What `importTable` gives: a table, or the kind of a refusal with the
/// fields its words need. It stays in the memory of the wasm, each read of
/// a field or of a column copies it out, and `free()` releases it, once.
#[wasm_bindgen(getter_with_clone)]
pub struct TableRead {
    /// "" for a table; otherwise the kind of the refusal: "unreadable",
    /// "tooLarge", "formatNotBuilt", "oldExcel", "encrypted",
    /// "notWorkbook", "emptySheet", "cellError", "sheetTooLarge",
    /// "cutShort", "notText", "variantsFile", "unclosedQuote",
    /// "headerError", "empty", "unnamedColumn", "raggedRow",
    /// "duplicateColumn", "emptyIndividual" or "duplicateIndividual".
    #[wasm_bindgen(readonly)]
    pub refusal: String,
    /// The format found from the first bytes, "text" or "xlsx", with a
    /// table and with every refusal but "unreadable", for which it is "".
    #[wasm_bindgen(readonly)]
    pub format: String,
    /// For "unreadable", the message of the zip crate, of calamine or of
    /// table_io; for "cellError" and "headerError", the error; for
    /// "duplicateColumn" and "duplicateIndividual", the name; "" otherwise.
    #[wasm_bindgen(readonly)]
    pub text: String,
    /// For "tooLarge", the size of the file in bytes; 0 otherwise.
    #[wasm_bindgen(readonly)]
    pub size: f64,
    /// The line of a text file, from 1, for "unclosedQuote", "raggedRow",
    /// "emptyIndividual" and the first of "duplicateIndividual"; 0
    /// otherwise.
    #[wasm_bindgen(readonly)]
    pub line: u32,
    /// The line of a text file that repeats the name, for
    /// "duplicateIndividual"; 0 otherwise.
    #[wasm_bindgen(readonly, js_name = secondLine)]
    pub second_line: u32,
    /// The row of the sheet of an xlsx, from 1, for "headerError",
    /// "emptyIndividual", the first of "duplicateIndividual", and the first
    /// row of the rectangle of "sheetTooLarge"; 0 otherwise.
    #[wasm_bindgen(readonly)]
    pub row: u32,
    /// The row of the sheet that repeats the name, for
    /// "duplicateIndividual" of an xlsx; 0 otherwise.
    #[wasm_bindgen(readonly, js_name = secondRow)]
    pub second_row: u32,
    /// The column, from 1, its place in the row of a text file or its
    /// column of the sheet of an xlsx, column A being 1, for
    /// "headerError", "unnamedColumn", the first of "duplicateColumn", and
    /// the first column of the rectangle of "sheetTooLarge"; 0 otherwise.
    #[wasm_bindgen(readonly)]
    pub column: u32,
    /// The column that repeats the name, for "duplicateColumn"; 0
    /// otherwise.
    #[wasm_bindgen(readonly, js_name = secondColumn)]
    pub second_column: u32,
    /// The number of cells of the header, for "raggedRow"; 0 otherwise.
    #[wasm_bindgen(readonly)]
    pub expected: u32,
    /// The number of cells of the row, for "raggedRow"; 0 otherwise.
    #[wasm_bindgen(readonly)]
    pub found: u32,
    /// The number of rows the rectangle had reached, for "sheetTooLarge";
    /// 0 otherwise.
    #[wasm_bindgen(readonly, js_name = sheetRows)]
    pub sheet_rows: u32,
    /// The number of columns the rectangle had reached, for
    /// "sheetTooLarge"; 0 otherwise.
    #[wasm_bindgen(readonly, js_name = sheetColumns)]
    pub sheet_columns: u32,
    /// The encoding a text file was read with, "utf-8", "utf-16" or
    /// "windows-1252"; "" for an xlsx and for a refusal.
    #[wasm_bindgen(readonly)]
    pub encoding: String,
    /// The separator, "tab", "semicolon" or "comma": the one a text file
    /// was read with, or split with for "unclosedQuote" and "raggedRow";
    /// "" for an xlsx and for the other refusals.
    #[wasm_bindgen(readonly)]
    pub separator: String,
    /// The decimal mark the numbers in text were read with, "point" or
    /// "comma", "point" for an xlsx; "" for a refusal.
    #[wasm_bindgen(readonly)]
    pub decimal: String,
    /// The line of a text file, from 1, of the first character that could
    /// not be decoded and stands as U+FFFD; undefined when every character
    /// was decoded, for an xlsx and for a refusal.
    #[wasm_bindgen(readonly, js_name = undecodedLine)]
    pub undecoded_line: Option<u32>,
    /// The name of the sheet of an xlsx, as its tab shows it, with a table
    /// and for "emptySheet" and "sheetTooLarge"; "" otherwise.
    #[wasm_bindgen(readonly)]
    pub sheet: String,
    /// The header of the column of the names, the first column of the
    /// file, which may be ""; "" for a refusal.
    #[wasm_bindgen(readonly, js_name = namesHeader)]
    pub names_header: String,
    /// The column of the names in the file, from 1, its column of the
    /// sheet in an xlsx; 0 for a refusal.
    #[wasm_bindgen(readonly, js_name = namesNumber)]
    pub names_number: u32,
    /// The names of the individuals, one for each row, none empty, no two
    /// the same; empty for a refusal. Each read makes a new array.
    #[wasm_bindgen(readonly)]
    pub names: Vec<String>,
    /// The number of columns other than the names, which the methods of a
    /// column take by their index, from 0; 0 for a refusal.
    #[wasm_bindgen(readonly, js_name = numColumns)]
    pub num_columns: u32,
    /// The columns other than the names, in the order of the file.
    columns: Vec<ColumnRead>,
}

/// A column of a table other than the names, as its methods give it.
struct ColumnRead {
    /// Its name in the header.
    name: String,
    /// Its column in the file, from 1.
    number: u32,
    /// Its type.
    column_type: ColumnType,
    /// Its values, as five arrays.
    arrays: ColumnArrays,
}

/// The values of a column as JavaScript reads them: one entry of `missing`
/// for each row, 1 for a missing value, and the array of the column's type
/// with one entry for each row, 0, 0, 0 or "" for a missing one; the other
/// three are empty.
struct ColumnArrays {
    /// 1 for a missing value, 0 for a value.
    missing: Vec<u8>,
    /// The values of an integer column.
    integers: Vec<i64>,
    /// The values of a float column.
    floats: Vec<f64>,
    /// The values of a boolean column, 1 for true and 0 for false.
    booleans: Vec<u8>,
    /// The values of a text column.
    texts: Vec<String>,
}

#[wasm_bindgen]
impl TableRead {
    /// The name in the header of the column `index`, from 0.
    ///
    /// # Errors
    ///
    /// Throws an `Error` for an index that is not below `numColumns`.
    #[wasm_bindgen(js_name = columnName)]
    pub fn column_name(&self, index: u32) -> Result<String, JsError> {
        Ok(self.column(index)?.name.clone())
    }

    /// The column in the file of the column `index`, from 1: its column of
    /// the sheet in an xlsx, column A being 1.
    ///
    /// # Errors
    ///
    /// Throws an `Error` for an index that is not below `numColumns`.
    #[wasm_bindgen(js_name = columnNumber)]
    pub fn column_number(&self, index: u32) -> Result<u32, JsError> {
        Ok(self.column(index)?.number)
    }

    /// The type of the column `index`: "integer", "float", "boolean" or
    /// "text".
    ///
    /// # Errors
    ///
    /// Throws an `Error` for an index that is not below `numColumns`.
    #[wasm_bindgen(js_name = columnType)]
    pub fn column_type(&self, index: u32) -> Result<String, JsError> {
        Ok(column_type_code(self.column(index)?.column_type).to_owned())
    }

    /// For each row of the column `index`, 1 when its value is missing and
    /// 0 otherwise. Each call makes a new array.
    ///
    /// # Errors
    ///
    /// Throws an `Error` for an index that is not below `numColumns`.
    #[wasm_bindgen(js_name = columnMissing)]
    pub fn column_missing(&self, index: u32) -> Result<Vec<u8>, JsError> {
        Ok(self.column(index)?.arrays.missing.clone())
    }

    /// The values of the column `index` when it is an integer column, one
    /// for each row, 0 for a missing one; empty for a column of another
    /// type. Each call makes a new array.
    ///
    /// # Errors
    ///
    /// Throws an `Error` for an index that is not below `numColumns`.
    #[wasm_bindgen(js_name = columnIntegers)]
    pub fn column_integers(&self, index: u32) -> Result<Vec<i64>, JsError> {
        Ok(self.column(index)?.arrays.integers.clone())
    }

    /// The values of the column `index` when it is a float column, one for
    /// each row, 0 for a missing one; empty for a column of another type.
    /// Each call makes a new array.
    ///
    /// # Errors
    ///
    /// Throws an `Error` for an index that is not below `numColumns`.
    #[wasm_bindgen(js_name = columnFloats)]
    pub fn column_floats(&self, index: u32) -> Result<Vec<f64>, JsError> {
        Ok(self.column(index)?.arrays.floats.clone())
    }

    /// The values of the column `index` when it is a boolean column, one
    /// for each row, 1 for true and 0 for false or a missing one; empty for
    /// a column of another type. Each call makes a new array.
    ///
    /// # Errors
    ///
    /// Throws an `Error` for an index that is not below `numColumns`.
    #[wasm_bindgen(js_name = columnBooleans)]
    pub fn column_booleans(&self, index: u32) -> Result<Vec<u8>, JsError> {
        Ok(self.column(index)?.arrays.booleans.clone())
    }

    /// The values of the column `index` when it is a text column, one for
    /// each row, "" for a missing one; empty for a column of another type.
    /// Each call makes a new array.
    ///
    /// # Errors
    ///
    /// Throws an `Error` for an index that is not below `numColumns`.
    #[wasm_bindgen(js_name = columnTexts)]
    pub fn column_texts(&self, index: u32) -> Result<Vec<String>, JsError> {
        Ok(self.column(index)?.arrays.texts.clone())
    }
}

impl TableRead {
    /// The column `index`, from 0, or the `Error` of an index out of range.
    fn column(&self, index: u32) -> Result<&ColumnRead, JsError> {
        usize::try_from(index)
            .ok()
            .and_then(|position| self.columns.get(position))
            .ok_or_else(|| {
                JsError::new(&format!(
                    "no column {index}: the table has {} columns",
                    self.num_columns
                ))
            })
    }
}

/// Reads the table of the file `bytes`, a CSV, a TSV or an xlsx, found from
/// its first bytes. `max_bytes` is the largest file accepted, in bytes, and
/// `max_cells` the largest rectangle of the values of an xlsx, in cells.
/// `encoding` is "" to find it, "utf-8" or "windows-1252"; `separator` ""
/// to find it, "tab", "semicolon" or "comma"; `decimal` "" to find it,
/// "point" or "comma"; the three are for a text file and ignored for an
/// xlsx. A file refused, or one that cannot be read, is a value: its kind
/// is in `refusal`.
///
/// # Errors
///
/// Throws an `Error` for a `max_bytes` that is not a whole number from 0 to
/// 2^53, a `max_cells` that is not one from 0 to 4,294,967,295, and an
/// option that is not one of its strings.
#[wasm_bindgen(js_name = importTable)]
pub fn import_table(
    bytes: &[u8],
    max_bytes: f64,
    max_cells: f64,
    encoding: &str,
    separator: &str,
    decimal: &str,
) -> Result<TableRead, JsError> {
    let options = ImportOptions {
        max_bytes: max_bytes_of(max_bytes)?,
        max_cells: max_cells_of(max_cells)?,
        text: TextOptions {
            encoding: encoding_set(encoding)?,
            separator: separator_set(separator)?,
            decimal: decimal_set(decimal)?,
        },
    };
    Ok(match table_io::import_table(bytes, &options) {
        Ok(table) => read_of_table(table),
        Err(ImportError::Refused { format, refusal }) => read_of_refusal(format, refusal),
        Err(ImportError::Unreadable(message)) => read_of_unreadable(message),
    })
}

/// What `convertColumn` gives: the column converted, as its five arrays,
/// or, with `numFailed` 1 or more, how many values do not convert and the
/// first of them. It stays in the memory of the wasm, each read of a field
/// copies it out, and `free()` releases it, once.
#[wasm_bindgen(getter_with_clone)]
pub struct Conversion {
    /// How many values do not convert; 0 when the column converted.
    #[wasm_bindgen(readonly, js_name = numFailed)]
    pub num_failed: f64,
    /// The row of the first value that does not convert, among the rows
    /// of the table, from 1; 0 when the column converted.
    #[wasm_bindgen(readonly, js_name = firstRow)]
    pub first_row: u32,
    /// The text of the first value that does not convert; "" when the
    /// column converted.
    #[wasm_bindgen(readonly, js_name = firstText)]
    pub first_text: String,
    /// For each row, 1 when its value is missing and 0 otherwise; empty
    /// when a value does not convert.
    #[wasm_bindgen(readonly)]
    pub missing: Vec<u8>,
    /// The values, 0 for a missing one, when converted to "integer";
    /// empty otherwise.
    #[wasm_bindgen(readonly)]
    pub integers: Vec<i64>,
    /// The values, 0 for a missing one, when converted to "float"; empty
    /// otherwise.
    #[wasm_bindgen(readonly)]
    pub floats: Vec<f64>,
    /// The values, 1 for true and 0 for false or a missing one, when
    /// converted to "boolean"; empty otherwise.
    #[wasm_bindgen(readonly)]
    pub booleans: Vec<u8>,
    /// The values, "" for a missing one, when converted to "text"; empty
    /// otherwise.
    #[wasm_bindgen(readonly)]
    pub texts: Vec<String>,
}

/// Converts a column to the type `to`, "integer", "float", "boolean" or
/// "text", its texts read and its floats written with `decimal`, "point"
/// or "comma". The column is given as `TableRead` gives it: its type,
/// `column_type`, `missing`, with 1 for a missing value and 0 for a value,
/// and the array of its type, of as many entries; the arrays of the other
/// types are not read.
///
/// # Errors
///
/// Throws an `Error` for a type or a decimal mark that is not one of its
/// strings, a `missing` or a `booleans` with an entry other than 0 and 1,
/// and an array of the column's type of another length than `missing`.
#[wasm_bindgen(js_name = convertColumn)]
#[expect(
    clippy::too_many_arguments,
    reason = "the column crosses as its type and its five arrays, as specs/package.md declares"
)]
pub fn convert_column(
    column_type: &str,
    missing: Vec<u8>,
    integers: Vec<i64>,
    floats: Vec<f64>,
    booleans: Vec<u8>,
    texts: Vec<String>,
    to: &str,
    decimal: &str,
) -> Result<Conversion, JsError> {
    let arrays = ColumnArrays {
        missing,
        integers,
        floats,
        booleans,
        texts,
    };
    let values = values_of_arrays(column_type_of(column_type)?, arrays)?;
    let to = column_type_of(to)?;
    let decimal = decimal_of(decimal)?;
    Ok(match table_io::convert_column(&values, to, decimal) {
        Ok(converted) => conversion_of_values(converted),
        Err(failure) => conversion_of_failure(failure),
    })
}

/// Whether `text` is a missing value: "", "NA" or "-", exactly.
#[wasm_bindgen(js_name = isMissing)]
pub fn is_missing(text: &str) -> bool {
    table_io::is_missing(text)
}

/// The whole number `text` holds, from −2^63 to 2^63 − 1, or undefined.
#[wasm_bindgen(js_name = parseInteger)]
pub fn parse_integer(text: &str) -> Option<i64> {
    table_io::parse_integer(text)
}

/// The finite number `text` holds with the decimal mark `decimal`, "point"
/// or "comma", or undefined.
///
/// # Errors
///
/// Throws an `Error` for a decimal mark that is not one of its strings.
#[wasm_bindgen(js_name = parseFloat)]
pub fn parse_float(text: &str, decimal: &str) -> Result<Option<f64>, JsError> {
    Ok(table_io::parse_float(text, decimal_of(decimal)?))
}

/// The boolean `text` holds, TRUE or FALSE in any case, or undefined.
#[wasm_bindgen(js_name = parseBoolean)]
pub fn parse_boolean(text: &str) -> Option<bool> {
    table_io::parse_boolean(text)
}

/// `number` as JavaScript's `String` writes it, with the decimal mark
/// `decimal`, "point" or "comma".
///
/// # Errors
///
/// Throws an `Error` for a decimal mark that is not one of its strings.
#[wasm_bindgen(js_name = floatText)]
pub fn float_text(number: f64, decimal: &str) -> Result<String, JsError> {
    Ok(table_io::float_text(number, decimal_of(decimal)?))
}

/// The limit of bytes `max_bytes`, a whole number from 0 to 2^53.
fn max_bytes_of(max_bytes: f64) -> Result<u64, JsError> {
    if is_whole_from_zero_to(max_bytes, MAX_JS_WHOLE_NUMBER) {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a whole number from 0 to 2^53, which a u64 holds exactly"
        )]
        Ok(max_bytes as u64)
    } else {
        Err(JsError::new(&format!(
            "max_bytes is {max_bytes}, not a whole number from 0 to 2^53"
        )))
    }
}

/// The limit of cells `max_cells`, a whole number from 0 to 4,294,967,295.
fn max_cells_of(max_cells: f64) -> Result<u32, JsError> {
    if is_whole_from_zero_to(max_cells, f64::from(u32::MAX)) {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a whole number from 0 to 4,294,967,295, which a u32 holds exactly"
        )]
        Ok(max_cells as u32)
    } else {
        Err(JsError::new(&format!(
            "max_cells is {max_cells}, not a whole number from 0 to 4294967295"
        )))
    }
}

/// Whether `number` is a whole number from 0 to `largest`; NaN and the
/// infinities are not, their fractional part being NaN.
fn is_whole_from_zero_to(number: f64, largest: f64) -> bool {
    number.fract() == 0.0 && (0.0..=largest).contains(&number)
}

/// The encoding set by the string `encoding`, `None` for "" to find it.
fn encoding_set(encoding: &str) -> Result<Option<Encoding>, JsError> {
    match encoding {
        "" => Ok(None),
        "utf-8" => Ok(Some(Encoding::Utf8)),
        "windows-1252" => Ok(Some(Encoding::Windows1252)),
        _ => Err(JsError::new(&format!(
            "the encoding is \"{encoding}\", not \"\", \"utf-8\" or \"windows-1252\""
        ))),
    }
}

/// The separator set by the string `separator`, `None` for "" to find it.
fn separator_set(separator: &str) -> Result<Option<Separator>, JsError> {
    match separator {
        "" => Ok(None),
        "tab" => Ok(Some(Separator::Tab)),
        "semicolon" => Ok(Some(Separator::Semicolon)),
        "comma" => Ok(Some(Separator::Comma)),
        _ => Err(JsError::new(&format!(
            "the separator is \"{separator}\", not \"\", \"tab\", \"semicolon\" or \"comma\""
        ))),
    }
}

/// The decimal mark set by the string `decimal`, `None` for "" to find it.
fn decimal_set(decimal: &str) -> Result<Option<DecimalMark>, JsError> {
    match decimal {
        "" => Ok(None),
        "point" => Ok(Some(DecimalMark::Point)),
        "comma" => Ok(Some(DecimalMark::Comma)),
        _ => Err(JsError::new(&format!(
            "the decimal mark is \"{decimal}\", not \"\", \"point\" or \"comma\""
        ))),
    }
}

/// The decimal mark of the string `decimal`, "point" or "comma".
fn decimal_of(decimal: &str) -> Result<DecimalMark, JsError> {
    match decimal {
        "point" => Ok(DecimalMark::Point),
        "comma" => Ok(DecimalMark::Comma),
        _ => Err(JsError::new(&format!(
            "the decimal mark is \"{decimal}\", not \"point\" or \"comma\""
        ))),
    }
}

/// The type of the string `column_type`.
fn column_type_of(column_type: &str) -> Result<ColumnType, JsError> {
    match column_type {
        "integer" => Ok(ColumnType::Integer),
        "float" => Ok(ColumnType::Float),
        "boolean" => Ok(ColumnType::Boolean),
        "text" => Ok(ColumnType::Text),
        _ => Err(JsError::new(&format!(
            "the type is \"{column_type}\", not \"integer\", \"float\", \"boolean\" or \"text\""
        ))),
    }
}

/// The string of a type.
fn column_type_code(column_type: ColumnType) -> &'static str {
    match column_type {
        ColumnType::Integer => "integer",
        ColumnType::Float => "float",
        ColumnType::Boolean => "boolean",
        ColumnType::Text => "text",
    }
}

/// The string of a format.
fn format_code(format: Format) -> &'static str {
    match format {
        Format::Text => "text",
        Format::Xlsx => "xlsx",
    }
}

/// The string of an encoding a text file was read with.
fn found_encoding_code(encoding: FoundEncoding) -> &'static str {
    match encoding {
        FoundEncoding::Utf8 => "utf-8",
        FoundEncoding::Utf16 => "utf-16",
        FoundEncoding::Windows1252 => "windows-1252",
    }
}

/// The string of a separator.
fn separator_code(separator: Separator) -> &'static str {
    match separator {
        Separator::Tab => "tab",
        Separator::Semicolon => "semicolon",
        Separator::Comma => "comma",
    }
}

/// The string of a decimal mark.
fn decimal_code(decimal: DecimalMark) -> &'static str {
    match decimal {
        DecimalMark::Point => "point",
        DecimalMark::Comma => "comma",
    }
}

/// The five arrays of the values of a column.
fn arrays_of_values(values: ColumnValues) -> ColumnArrays {
    match values {
        ColumnValues::Integer(integers) => ColumnArrays {
            missing: missing_of(&integers),
            integers: integers
                .iter()
                .map(|integer| integer.unwrap_or(0))
                .collect(),
            floats: Vec::new(),
            booleans: Vec::new(),
            texts: Vec::new(),
        },
        ColumnValues::Float(floats) => ColumnArrays {
            missing: missing_of(&floats),
            integers: Vec::new(),
            floats: floats.iter().map(|float| float.unwrap_or(0.0)).collect(),
            booleans: Vec::new(),
            texts: Vec::new(),
        },
        ColumnValues::Boolean(booleans) => ColumnArrays {
            missing: missing_of(&booleans),
            integers: Vec::new(),
            floats: Vec::new(),
            booleans: booleans
                .iter()
                .map(|boolean| u8::from(boolean.unwrap_or(false)))
                .collect(),
            texts: Vec::new(),
        },
        ColumnValues::Text(texts) => ColumnArrays {
            missing: missing_of(&texts),
            integers: Vec::new(),
            floats: Vec::new(),
            booleans: Vec::new(),
            texts: texts.into_iter().map(Option::unwrap_or_default).collect(),
        },
    }
}

/// For each value, 1 when it is missing and 0 otherwise.
fn missing_of<T>(values: &[Option<T>]) -> Vec<u8> {
    values
        .iter()
        .map(|cell_value| u8::from(cell_value.is_none()))
        .collect()
}

/// The values of a column of the type `column_type` given as its arrays:
/// `missing` and the array of its type.
fn values_of_arrays(
    column_type: ColumnType,
    arrays: ColumnArrays,
) -> Result<ColumnValues, JsError> {
    let ColumnArrays {
        missing,
        integers,
        floats,
        booleans,
        texts,
    } = arrays;
    let is_missing_each = flags_of(&missing, "missing")?;
    match column_type {
        ColumnType::Integer => Ok(ColumnValues::Integer(present_of(
            &is_missing_each,
            integers,
            "integers",
        )?)),
        ColumnType::Float => Ok(ColumnValues::Float(present_of(
            &is_missing_each,
            floats,
            "floats",
        )?)),
        ColumnType::Boolean => {
            let booleans = flags_of(&booleans, "booleans")?;
            Ok(ColumnValues::Boolean(present_of(
                &is_missing_each,
                booleans,
                "booleans",
            )?))
        }
        ColumnType::Text => Ok(ColumnValues::Text(present_of(
            &is_missing_each,
            texts,
            "texts",
        )?)),
    }
}

/// The entries of the array `flags`, named `array_name`, as booleans, 1
/// true and 0 false.
fn flags_of(flags: &[u8], array_name: &str) -> Result<Vec<bool>, JsError> {
    flags
        .iter()
        .map(|&flag| match flag {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(JsError::new(&format!(
                "{array_name} holds {flag}, not 0 or 1"
            ))),
        })
        .collect()
}

/// The values of `entries`, the array named `array_name`, each `None`
/// where `is_missing_each` is true.
fn present_of<T>(
    is_missing_each: &[bool],
    entries: Vec<T>,
    array_name: &str,
) -> Result<Vec<Option<T>>, JsError> {
    if entries.len() != is_missing_each.len() {
        return Err(JsError::new(&format!(
            "{array_name} has {} entries and missing {}",
            entries.len(),
            is_missing_each.len()
        )));
    }
    Ok(entries
        .into_iter()
        .zip(is_missing_each)
        .map(|(entry, &is_missing_entry)| (!is_missing_entry).then_some(entry))
        .collect())
}

/// The `Conversion` of a column converted.
fn conversion_of_values(values: ColumnValues) -> Conversion {
    let ColumnArrays {
        missing,
        integers,
        floats,
        booleans,
        texts,
    } = arrays_of_values(values);
    Conversion {
        num_failed: 0.0,
        first_row: 0,
        first_text: String::new(),
        missing,
        integers,
        floats,
        booleans,
        texts,
    }
}

/// The `Conversion` of a column some of whose values do not convert.
fn conversion_of_failure(failure: ConversionFailure) -> Conversion {
    let ConversionFailure {
        num_failed,
        first_row,
        first_text,
    } = failure;
    Conversion {
        num_failed: number_of_count(num_failed),
        first_row,
        first_text,
        missing: Vec::new(),
        integers: Vec::new(),
        floats: Vec::new(),
        booleans: Vec::new(),
        texts: Vec::new(),
    }
}

/// A count of 64 bits as a number of JavaScript, exact up to 2^53, beyond
/// any count of rows or of bytes the memory of the wasm can hold.
fn number_of_count(count: u64) -> f64 {
    count as f64
}

/// The `TableRead` of a table.
fn read_of_table(table: Table) -> TableRead {
    let Table {
        names,
        columns,
        read,
    } = table;
    let (format, encoding, separator, decimal, undecoded_line, sheet) = match read {
        HowRead::Text(text_read) => (
            Format::Text,
            found_encoding_code(text_read.encoding),
            separator_code(text_read.separator),
            text_read.decimal,
            text_read.undecoded_line,
            String::new(),
        ),
        HowRead::Xlsx { sheet } => (Format::Xlsx, "", "", DecimalMark::Point, None, sheet),
    };
    let columns: Vec<ColumnRead> = columns
        .into_iter()
        .map(|column| ColumnRead {
            column_type: column.values.column_type(),
            name: column.name,
            number: column.number,
            arrays: arrays_of_values(column.values),
        })
        .collect();
    TableRead {
        refusal: String::new(),
        format: format_code(format).to_owned(),
        text: String::new(),
        size: 0.0,
        line: 0,
        second_line: 0,
        row: 0,
        second_row: 0,
        column: 0,
        second_column: 0,
        expected: 0,
        found: 0,
        sheet_rows: 0,
        sheet_columns: 0,
        encoding: encoding.to_owned(),
        separator: separator.to_owned(),
        decimal: decimal_code(decimal).to_owned(),
        undecoded_line,
        sheet,
        names_header: names.header,
        names_number: names.number,
        names: names.names,
        // A table of the memory of the wasm, whose addresses are of 32
        // bits, has fewer columns than u32::MAX.
        num_columns: u32::try_from(columns.len()).unwrap_or(u32::MAX),
        columns,
    }
}

/// The fields of a refusal in a `TableRead`, each 0 or "" when its kind
/// does not fill it.
struct RefusalFields {
    /// The kind, in camelCase.
    kind: &'static str,
    /// The format found, "" for a file that cannot be read.
    format: &'static str,
    /// The message, the error or the name.
    text: String,
    /// The size of the file, in bytes.
    size: f64,
    /// The line of a text file.
    line: u32,
    /// The line that repeats a name.
    second_line: u32,
    /// The row of the sheet.
    row: u32,
    /// The row that repeats a name.
    second_row: u32,
    /// The column.
    column: u32,
    /// The column that repeats a name.
    second_column: u32,
    /// The cells of the header.
    expected: u32,
    /// The cells of the row.
    found: u32,
    /// The rows of the rectangle.
    sheet_rows: u32,
    /// The columns of the rectangle.
    sheet_columns: u32,
    /// The separator.
    separator: &'static str,
    /// The sheet.
    sheet: String,
}

/// The `TableRead` of a file that cannot be read, with its message.
fn read_of_unreadable(message: String) -> TableRead {
    read_of_fields(RefusalFields {
        kind: "unreadable",
        format: "",
        text: message,
        size: 0.0,
        line: 0,
        second_line: 0,
        row: 0,
        second_row: 0,
        column: 0,
        second_column: 0,
        expected: 0,
        found: 0,
        sheet_rows: 0,
        sheet_columns: 0,
        separator: "",
        sheet: String::new(),
    })
}

/// The `TableRead` of a refusal of a file of the format `format`.
fn read_of_refusal(format: Format, refusal: Refusal) -> TableRead {
    let format_name = format_code(format);
    let fields = match refusal {
        Refusal::TooLarge { size, max_bytes: _ } => RefusalFields {
            kind: "tooLarge",
            format: format_name,
            text: String::new(),
            size: number_of_count(size),
            line: 0,
            second_line: 0,
            row: 0,
            second_row: 0,
            column: 0,
            second_column: 0,
            expected: 0,
            found: 0,
            sheet_rows: 0,
            sheet_columns: 0,
            separator: "",
            sheet: String::new(),
        },
        Refusal::FormatNotBuilt => fields_of_kind("formatNotBuilt", format_name),
        Refusal::OldExcel => fields_of_kind("oldExcel", format_name),
        Refusal::Encrypted => fields_of_kind("encrypted", format_name),
        Refusal::NotWorkbook => fields_of_kind("notWorkbook", format_name),
        Refusal::CutShort => fields_of_kind("cutShort", format_name),
        Refusal::NotText => fields_of_kind("notText", format_name),
        Refusal::VariantsFile => fields_of_kind("variantsFile", format_name),
        Refusal::Empty => fields_of_kind("empty", format_name),
        Refusal::EmptySheet { sheet } => RefusalFields {
            kind: "emptySheet",
            format: format_name,
            text: String::new(),
            size: 0.0,
            line: 0,
            second_line: 0,
            row: 0,
            second_row: 0,
            column: 0,
            second_column: 0,
            expected: 0,
            found: 0,
            sheet_rows: 0,
            sheet_columns: 0,
            separator: "",
            sheet,
        },
        Refusal::CellError { error } => RefusalFields {
            kind: "cellError",
            format: format_name,
            text: error,
            size: 0.0,
            line: 0,
            second_line: 0,
            row: 0,
            second_row: 0,
            column: 0,
            second_column: 0,
            expected: 0,
            found: 0,
            sheet_rows: 0,
            sheet_columns: 0,
            separator: "",
            sheet: String::new(),
        },
        Refusal::SheetTooLarge {
            sheet,
            first_row,
            first_column,
            num_rows,
            num_columns,
        } => RefusalFields {
            kind: "sheetTooLarge",
            format: format_name,
            text: String::new(),
            size: 0.0,
            line: 0,
            second_line: 0,
            row: first_row,
            second_row: 0,
            column: first_column,
            second_column: 0,
            expected: 0,
            found: 0,
            sheet_rows: num_rows,
            sheet_columns: num_columns,
            separator: "",
            sheet,
        },
        Refusal::UnclosedQuote { line, separator } => RefusalFields {
            kind: "unclosedQuote",
            format: format_name,
            text: String::new(),
            size: 0.0,
            line,
            second_line: 0,
            row: 0,
            second_row: 0,
            column: 0,
            second_column: 0,
            expected: 0,
            found: 0,
            sheet_rows: 0,
            sheet_columns: 0,
            separator: separator_code(separator),
            sheet: String::new(),
        },
        Refusal::HeaderError { row, column, error } => RefusalFields {
            kind: "headerError",
            format: format_name,
            text: error,
            size: 0.0,
            line: 0,
            second_line: 0,
            row,
            second_row: 0,
            column,
            second_column: 0,
            expected: 0,
            found: 0,
            sheet_rows: 0,
            sheet_columns: 0,
            separator: "",
            sheet: String::new(),
        },
        Refusal::UnnamedColumn { column } => RefusalFields {
            kind: "unnamedColumn",
            format: format_name,
            text: String::new(),
            size: 0.0,
            line: 0,
            second_line: 0,
            row: 0,
            second_row: 0,
            column,
            second_column: 0,
            expected: 0,
            found: 0,
            sheet_rows: 0,
            sheet_columns: 0,
            separator: "",
            sheet: String::new(),
        },
        Refusal::RaggedRow {
            line,
            expected,
            found,
            separator,
        } => RefusalFields {
            kind: "raggedRow",
            format: format_name,
            text: String::new(),
            size: 0.0,
            line,
            second_line: 0,
            row: 0,
            second_row: 0,
            column: 0,
            second_column: 0,
            expected,
            found,
            sheet_rows: 0,
            sheet_columns: 0,
            separator: separator_code(separator),
            sheet: String::new(),
        },
        Refusal::DuplicateColumn {
            name,
            first_column,
            second_column,
        } => RefusalFields {
            kind: "duplicateColumn",
            format: format_name,
            text: name,
            size: 0.0,
            line: 0,
            second_line: 0,
            row: 0,
            second_row: 0,
            column: first_column,
            second_column,
            expected: 0,
            found: 0,
            sheet_rows: 0,
            sheet_columns: 0,
            separator: "",
            sheet: String::new(),
        },
        Refusal::EmptyIndividual { row } => {
            let (line, row) = line_or_row(format, row);
            RefusalFields {
                kind: "emptyIndividual",
                format: format_name,
                text: String::new(),
                size: 0.0,
                line,
                second_line: 0,
                row,
                second_row: 0,
                column: 0,
                second_column: 0,
                expected: 0,
                found: 0,
                sheet_rows: 0,
                sheet_columns: 0,
                separator: "",
                sheet: String::new(),
            }
        }
        Refusal::DuplicateIndividual {
            name,
            first_row,
            second_row,
        } => {
            let (line, row) = line_or_row(format, first_row);
            let (second_line, second_row) = line_or_row(format, second_row);
            RefusalFields {
                kind: "duplicateIndividual",
                format: format_name,
                text: name,
                size: 0.0,
                line,
                second_line,
                row,
                second_row,
                column: 0,
                second_column: 0,
                expected: 0,
                found: 0,
                sheet_rows: 0,
                sheet_columns: 0,
                separator: "",
                sheet: String::new(),
            }
        }
    };
    read_of_fields(fields)
}

/// A place of the library's refusal, which is a line of a text file and a
/// row of the sheet of an xlsx, as the fields `line` and `row`: the one of
/// its format filled, the other 0.
fn line_or_row(format: Format, place: u32) -> (u32, u32) {
    match format {
        Format::Text => (place, 0),
        Format::Xlsx => (0, place),
    }
}

/// The fields of a refusal of the kind `kind` that fills none but `format`.
fn fields_of_kind(kind: &'static str, format: &'static str) -> RefusalFields {
    RefusalFields {
        kind,
        format,
        text: String::new(),
        size: 0.0,
        line: 0,
        second_line: 0,
        row: 0,
        second_row: 0,
        column: 0,
        second_column: 0,
        expected: 0,
        found: 0,
        sheet_rows: 0,
        sheet_columns: 0,
        separator: "",
        sheet: String::new(),
    }
}

/// The `TableRead` of a refusal, or of a file that cannot be read, with no
/// table.
fn read_of_fields(fields: RefusalFields) -> TableRead {
    let RefusalFields {
        kind,
        format,
        text,
        size,
        line,
        second_line,
        row,
        second_row,
        column,
        second_column,
        expected,
        found,
        sheet_rows,
        sheet_columns,
        separator,
        sheet,
    } = fields;
    TableRead {
        refusal: kind.to_owned(),
        format: format.to_owned(),
        text,
        size,
        line,
        second_line,
        row,
        second_row,
        column,
        second_column,
        expected,
        found,
        sheet_rows,
        sheet_columns,
        encoding: String::new(),
        separator: separator.to_owned(),
        decimal: String::new(),
        undecoded_line: None,
        sheet,
        names_header: String::new(),
        names_number: 0,
        names: Vec::new(),
        num_columns: 0,
        columns: Vec::new(),
    }
}
