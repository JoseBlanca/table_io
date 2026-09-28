//! The files the owner makes by hand in Excel, LibreOffice and Google
//! Sheets, read from `tests/data/` ("Made by the owner" of
//! `docs/specs/read.md`). None is there yet, so each test is ignored with
//! the name of the file it waits for.
//!
//! The owner has not yet said in which cells each value lies, so the tests
//! assert what the spec says each file holds and can be found without it:
//! the header, and in the column a header names, the values the spec gives.
//! When a file arrives, its `#[ignore]` is taken off and the cells the owner
//! says the file shows in Excel are added to its test as literals. What a
//! file gives that the spec does not expect is a finding for the spec, and
//! not a test to be bent.

use std::path::PathBuf;

use xlsx_rs::{ReadError, Refusal, Sheet, SheetCell, read_first_sheet};

/// `MAX_SHEET_CELLS` of popnei_web, the limit its light worker gives.
const MAX_SHEET_CELLS: u32 = 2_000_000;

/// The header of the table of the first file, `excel_es.xlsx`, which
/// `excel_en.xlsx`, `libreoffice.xlsx` and `google_sheets.xlsx` repeat.
const HEADER: [&str; 7] = [
    "Individuo",
    "Población",
    "Altura",
    "Fecha",
    "Hora",
    "Afectado",
    "Código",
];

/// What `read_first_sheet` gives for `file_name` in `tests/data/` at the
/// root of the repository.
fn read_owner_file(file_name: &str) -> std::io::Result<Result<Sheet, ReadError>> {
    let bytes = std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/data")
            .join(file_name),
    )?;
    Ok(read_first_sheet(&bytes, MAX_SHEET_CELLS))
}

/// A text cell of `cell_text`.
fn text(cell_text: &str) -> SheetCell {
    SheetCell::Text(cell_text.to_owned())
}

/// The rows of `sheet`, each a slice of its cells, the header first.
fn rows_of(sheet: &Sheet) -> Vec<&[SheetCell]> {
    let num_columns = usize::try_from(sheet.num_columns).unwrap_or(0);
    if num_columns == 0 {
        return Vec::new();
    }
    sheet.cells.chunks(num_columns).collect()
}

/// The cells under the header `header` in the first row of `sheet`, from
/// the second row to the last; empty when no cell of the first row is
/// `header`.
fn column_under(sheet: &Sheet, header: &str) -> Vec<SheetCell> {
    let rows = rows_of(sheet);
    let Some((header_row, other_rows)) = rows.split_first() else {
        return Vec::new();
    };
    let Some(column) = header_row.iter().position(|cell| *cell == text(header)) else {
        return Vec::new();
    };
    other_rows
        .iter()
        .filter_map(|row| row.get(column).cloned())
        .collect()
}

/// Asserts what "Made by the owner" says of the table of `excel_es.xlsx`,
/// which `excel_en.xlsx`, `libreoffice.xlsx` and `google_sheets.xlsx`
/// repeat: its header; in `Altura`, the height typed `1,75`; in `Fecha`,
/// the date typed `13/05/2024`; in `Hora`, the time typed `14:30`; in
/// `Afectado`, `VERDADERO` and `FALSO` as booleans; in `Código`, the `7`
/// with the format `000` as the number; somewhere, the identifier typed
/// `'001` as its text, and the errors of `=NA()` and `=1/0` as their texts
/// in English; in `Población`, the population merged over two rows in both;
/// and a blank row.
fn assert_the_table_of_the_first_file(file_name: &str, sheet: &Sheet) {
    let header_row = rows_of(sheet).first().map(|row| row.to_vec());
    for name in HEADER {
        assert!(
            header_row
                .as_ref()
                .is_some_and(|row| row.contains(&text(name))),
            "{file_name}: no {name} in the header {header_row:?}"
        );
    }
    let expected_in_columns = [
        ("Altura", SheetCell::Number(1.75)),
        ("Fecha", text("2024-05-13")),
        ("Hora", text("14:30:00")),
        ("Afectado", SheetCell::Bool(true)),
        ("Afectado", SheetCell::Bool(false)),
        ("Código", SheetCell::Number(7.0)),
    ];
    for (header, cell) in expected_in_columns {
        let column = column_under(sheet, header);
        assert!(
            column.contains(&cell),
            "{file_name}: no {cell:?} under {header}, which holds {column:?}"
        );
    }
    for cell in [text("001"), text("#N/A"), text("#DIV/0!")] {
        assert!(
            sheet.cells.contains(&cell),
            "{file_name}: no cell is {cell:?}"
        );
    }
    let populations = column_under(sheet, "Población");
    assert!(
        populations.windows(2).any(|pair| matches!(
            pair,
            [first, second] if first == second && *first != SheetCell::Empty
        )),
        "{file_name}: no population in two rows running, as a merged one is, in {populations:?}"
    );
    assert!(
        rows_of(sheet)
            .iter()
            .any(|row| row.iter().all(|cell| *cell == SheetCell::Empty)),
        "{file_name}: no blank row"
    );
}

// The cells the owner says excel_es.xlsx shows in Excel are to be added
// here as literals when the file arrives.
#[test]
#[ignore = "waits for tests/data/excel_es.xlsx, made by the owner"]
fn excel_es_xlsx_gives_the_cells_of_spanish_excel() {
    let sheet = read_owner_file("excel_es.xlsx").unwrap().unwrap();

    assert_the_table_of_the_first_file("excel_es.xlsx", &sheet);
}

// The cells the owner says excel_en.xlsx shows in Excel are to be added
// here as literals when the file arrives.
#[test]
#[ignore = "waits for tests/data/excel_en.xlsx, made by the owner"]
fn excel_en_xlsx_gives_the_cells_of_english_excel() {
    let sheet = read_owner_file("excel_en.xlsx").unwrap().unwrap();

    assert_the_table_of_the_first_file("excel_en.xlsx", &sheet);
}

// The cells the owner says libreoffice.xlsx shows in LibreOffice are to be
// added here as literals when the file arrives.
#[test]
#[ignore = "waits for tests/data/libreoffice.xlsx, made by the owner"]
fn libreoffice_xlsx_gives_the_cells_of_libreoffice_calc() {
    let sheet = read_owner_file("libreoffice.xlsx").unwrap().unwrap();

    assert_the_table_of_the_first_file("libreoffice.xlsx", &sheet);
}

// The cells the owner says excel_1904.xlsx shows in Excel are to be added
// here as literals when the file arrives.
#[test]
#[ignore = "waits for tests/data/excel_1904.xlsx, made by the owner"]
fn excel_1904_xlsx_gives_the_date_as_excel_shows_it() {
    let sheet = read_owner_file("excel_1904.xlsx").unwrap().unwrap();

    assert!(
        sheet.cells.contains(&text("2024-05-13")),
        "no cell is 2024-05-13 in {:?}",
        sheet.cells
    );
}

// The file is a table saved with a password to open it; which table does
// not matter, and nothing is to be added.
#[test]
#[ignore = "waits for tests/data/encrypted.xlsx, made by the owner"]
fn encrypted_xlsx_is_refused_as_encrypted() {
    let read = read_owner_file("encrypted.xlsx").unwrap();

    assert_eq!(read, Err(ReadError::Refused(Refusal::Encrypted)));
}

// The file is a table saved as "Excel 97-2003 Workbook"; which table does
// not matter, and nothing is to be added.
#[test]
#[ignore = "waits for tests/data/excel97.xls, made by the owner"]
fn excel97_xls_is_refused_as_old_excel() {
    let read = read_owner_file("excel97.xls").unwrap();

    assert_eq!(read, Err(ReadError::Refused(Refusal::OldExcel)));
}

// "The refusals", point 5, of the spec: whether Excel saves the error of a
// spill as #SPILL!, which calamine refuses, or as #VALUE!, which it reads,
// is not known until this file is read. The test takes either and prints
// which; the spec is then corrected to it, and the test asserts that one
// alone, with the cells the owner says the file shows.
#[test]
#[ignore = "waits for tests/data/spill.xlsx, made by the owner"]
fn spill_xlsx_is_refused_as_spill_or_read_with_value() {
    let read = read_owner_file("spill.xlsx").unwrap();

    match read {
        Err(ReadError::Refused(Refusal::CellError { error })) => {
            assert_eq!(error, "#SPILL!");
            println!("spill.xlsx is refused as CellError, #SPILL!");
        }
        Ok(sheet) => {
            assert!(
                sheet.cells.contains(&text("#VALUE!")),
                "spill.xlsx is read with no cell #VALUE!: {:?}",
                sheet.cells
            );
            println!("spill.xlsx is read, with a cell #VALUE!");
        }
        Err(read_error) => panic!("spill.xlsx gives neither: {read_error:?}"),
    }
}

// The cells the owner says google_sheets.xlsx shows in Google Sheets are to
// be added here as literals when the file arrives, if the owner uses it.
#[test]
#[ignore = "waits for tests/data/google_sheets.xlsx, made by the owner"]
fn google_sheets_xlsx_gives_the_cells_of_google_sheets() {
    let sheet = read_owner_file("google_sheets.xlsx").unwrap().unwrap();

    assert_the_table_of_the_first_file("google_sheets.xlsx", &sheet);
}
