//! The binding of xlsx_rs to JavaScript: the one exported function,
//! `readXlsx`, a thin wrapper over the library crate.
//!
//! It is compiled for `wasm32-unknown-unknown`, and the `wasm-bindgen`
//! command line writes from it the JavaScript and the declarations of the
//! package `js/xlsx_rs`. It holds no rule about a cell: it copies the sheet
//! or the refusal that `xlsx_rs::read_first_sheet` gives into [`XlsxRead`],
//! and turns a file calamine cannot read into a JavaScript `Error` with
//! calamine's message.
//!
//! wasm-bindgen copies the doc comments of the exported items into the
//! declarations, `wasm/xlsx_rs.d.ts`, which the test of the package
//! compares with `js/xlsx_rs/test/xlsx_rs.d.ts`: a change of one of them is
//! a change of that file too.

use wasm_bindgen::prelude::{JsError, JsValue, wasm_bindgen};
use xlsx_rs::{ReadError, Refusal, Sheet, SheetCell};

/// What `readXlsx` gives: the sheet read, or the code of a refusal with the
/// fields its words need. It stays in the memory of the wasm, each field
/// read is a copy, and `free()` releases it.
#[wasm_bindgen(getter_with_clone)]
pub struct XlsxRead {
    /// "" for a sheet read; otherwise "notXlsx", "oldExcel",
    /// "encrypted", "emptySheet", "cellError" or "sheetTooLarge".
    #[wasm_bindgen(readonly)]
    pub refusal: String,
    /// The text of the error, for "cellError"; "" otherwise.
    #[wasm_bindgen(readonly)]
    pub detail: String,
    /// The name of the sheet, for a sheet read, "emptySheet" and
    /// "sheetTooLarge"; "" otherwise.
    #[wasm_bindgen(readonly)]
    pub sheet: String,
    /// The first row of the rectangle of the sheet read, or of
    /// "sheetTooLarge", as Excel numbers the rows, from 1; 0 otherwise.
    #[wasm_bindgen(readonly, js_name = firstRow)]
    pub first_row: u32,
    /// The first column of the rectangle, column A being 1; 0 when there
    /// is no rectangle.
    #[wasm_bindgen(readonly, js_name = firstColumn)]
    pub first_column: u32,
    /// The number of rows of the rectangle; 0 when there is none.
    #[wasm_bindgen(readonly, js_name = numRows)]
    pub num_rows: u32,
    /// The number of columns of the rectangle; 0 when there is none.
    #[wasm_bindgen(readonly, js_name = numColumns)]
    pub num_columns: u32,
    /// Row after row: null, a string, a number or a boolean; empty for a
    /// refusal. Each read of it makes a new array.
    #[wasm_bindgen(readonly)]
    pub cells: Vec<JsValue>,
}

/// Reads the first worksheet that is not hidden of the xlsx `bytes`, or
/// refuses it; one reason is a rectangle of the values larger than
/// `max_cells` cells.
///
/// # Errors
///
/// Throws an `Error` with calamine's message for a file it cannot read.
#[wasm_bindgen(js_name = readXlsx)]
pub fn read_xlsx(bytes: &[u8], max_cells: u32) -> Result<XlsxRead, JsError> {
    match xlsx_rs::read_first_sheet(bytes, max_cells) {
        Ok(sheet) => Ok(read_of_sheet(sheet)),
        Err(ReadError::Refused(refusal)) => Ok(read_of_refusal(refusal)),
        Err(ReadError::Unreadable(message)) => Err(JsError::new(&message)),
    }
}

/// The `XlsxRead` of a sheet read.
fn read_of_sheet(sheet: Sheet) -> XlsxRead {
    XlsxRead {
        refusal: String::new(),
        detail: String::new(),
        sheet: sheet.name,
        first_row: sheet.first_row,
        first_column: sheet.first_column,
        num_rows: sheet.num_rows,
        num_columns: sheet.num_columns,
        cells: sheet.cells.into_iter().map(js_value_of_cell).collect(),
    }
}

/// The JavaScript value of a cell: null, a string, a number or a boolean.
fn js_value_of_cell(cell: SheetCell) -> JsValue {
    match cell {
        SheetCell::Empty => JsValue::NULL,
        SheetCell::Text(text) => JsValue::from(text),
        SheetCell::Number(number) => JsValue::from(number),
        SheetCell::Bool(is_true) => JsValue::from(is_true),
    }
}

/// The rectangle a refusal gives, in the numbers of [`XlsxRead`]: all 0
/// for a refusal with no rectangle.
struct RefusalRectangle {
    /// The first row, from 1.
    first_row: u32,
    /// The first column, column A being 1.
    first_column: u32,
    /// The number of rows.
    num_rows: u32,
    /// The number of columns.
    num_columns: u32,
}

/// The rectangle of a refusal that has none.
const NO_RECTANGLE: RefusalRectangle = RefusalRectangle {
    first_row: 0,
    first_column: 0,
    num_rows: 0,
    num_columns: 0,
};

/// The `XlsxRead` of a refusal: its code, and the fields its words need.
fn read_of_refusal(refusal: Refusal) -> XlsxRead {
    let (code, detail, sheet, rectangle) = match refusal {
        Refusal::NotXlsx => ("notXlsx", String::new(), String::new(), NO_RECTANGLE),
        Refusal::OldExcel => ("oldExcel", String::new(), String::new(), NO_RECTANGLE),
        Refusal::Encrypted => ("encrypted", String::new(), String::new(), NO_RECTANGLE),
        Refusal::EmptySheet { sheet } => ("emptySheet", String::new(), sheet, NO_RECTANGLE),
        Refusal::CellError { error } => ("cellError", error, String::new(), NO_RECTANGLE),
        Refusal::SheetTooLarge {
            sheet,
            first_row,
            first_column,
            num_rows,
            num_columns,
        } => (
            "sheetTooLarge",
            String::new(),
            sheet,
            RefusalRectangle {
                first_row,
                first_column,
                num_rows,
                num_columns,
            },
        ),
    };
    let RefusalRectangle {
        first_row,
        first_column,
        num_rows,
        num_columns,
    } = rectangle;
    XlsxRead {
        refusal: code.to_owned(),
        detail,
        sheet,
        first_row,
        first_column,
        num_rows,
        num_columns,
        cells: Vec::new(),
    }
}
