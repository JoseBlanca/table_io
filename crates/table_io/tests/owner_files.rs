//! The files the owner makes by hand in Excel, LibreOffice and Google
//! Sheets, read from `tests/data/` ("Made by the owner" of
//! `docs/specs/read.md`). A test whose file is not there, `libreoffice.xlsx`,
//! is ignored with the name of the file it waits for.
//!
//! The tests assert what the spec says each file holds: the header; in the
//! column a header names, the values the spec gives, and the kind of every
//! cell, a number under `Altura`, a date under `Fecha`; and a population in
//! every row of an individual, which a merged range lost would leave empty.
//! A test of a file with cells also holds the list of the cells the owner
//! typed, each with its reference in Excel, checked by reading each file on
//! 28 September 2026, and fails while that list is empty, so that taking
//! off an `#[ignore]` without the cells cannot pass. What a file gives that
//! the spec does not expect is a finding for the spec, and not a test to be
//! bent.

#![cfg(feature = "xlsx")]

use std::path::PathBuf;

use table_io::{ReadError, Refusal, Sheet, SheetCell, read_first_sheet};

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

/// Whether a cell is of one kind, a number or a date.
type IsOfKind = fn(&SheetCell) -> bool;

/// Whether `cell_text` has the form `pattern`, where each `d` of the
/// pattern is a digit from 0 to 9 and every other character is itself:
/// `dddd-dd-dd` for a date as table_io writes it.
fn has_form(cell_text: &str, pattern: &str) -> bool {
    cell_text.chars().count() == pattern.chars().count()
        && cell_text
            .chars()
            .zip(pattern.chars())
            .all(|(character, pattern_character)| match pattern_character {
                'd' => character.is_ascii_digit(),
                _ => character == pattern_character,
            })
}

/// The cells under `header` that are not empty and are not of the kind
/// `is_of_kind` accepts.
fn cells_not_of_kind(
    sheet: &Sheet,
    header: &str,
    is_of_kind: impl Fn(&SheetCell) -> bool,
) -> Vec<SheetCell> {
    column_under(sheet, header)
        .into_iter()
        .filter(|cell| *cell != SheetCell::Empty && !is_of_kind(cell))
        .collect()
}

/// The rows of `sheet`, counted from 0 for the header, whose `Individuo`
/// is not empty and whose `Población` is.
fn rows_of_an_individual_with_no_population(sheet: &Sheet) -> Vec<usize> {
    let individuals = column_under(sheet, "Individuo");
    let populations = column_under(sheet, "Población");
    (1..)
        .zip(individuals.iter().zip(&populations))
        .filter(|(_, (individual, population))| {
            **individual != SheetCell::Empty && **population == SheetCell::Empty
        })
        .map(|(row, _)| row)
        .collect()
}

/// The cell of `sheet` at `reference`, such as `C3`, as Excel names the
/// cells; `None` when the reference is outside the rectangle or is not one.
fn cell_at<'sheet>(sheet: &'sheet Sheet, reference: &str) -> Option<&'sheet SheetCell> {
    let digits_start = reference.find(|character: char| character.is_ascii_digit())?;
    let (letters, digits) = reference.split_at(digits_start);
    let column = letters.bytes().try_fold(0u32, |column, letter| {
        let letter_number = u32::from(letter.checked_sub(b'A')?).checked_add(1)?;
        (letter_number <= 26).then_some(())?;
        column.checked_mul(26)?.checked_add(letter_number)
    })?;
    let row: u32 = digits.parse().ok()?;
    let row_offset = row.checked_sub(sheet.first_row)?;
    let column_offset = column.checked_sub(sheet.first_column)?;
    if row_offset >= sheet.num_rows || column_offset >= sheet.num_columns {
        return None;
    }
    let position = u64::from(row_offset)
        .checked_mul(u64::from(sheet.num_columns))?
        .checked_add(u64::from(column_offset))?;
    sheet.cells.get(usize::try_from(position).ok()?)
}

/// Asserts that `sheet` holds `owner_cells`, the cells the owner says
/// `file_name` shows, each its reference in Excel and its cell; and that
/// there are some, so that a test whose list was never filled fails.
fn assert_the_cells_the_owner_sees(
    file_name: &str,
    sheet: &Sheet,
    owner_cells: &[(&str, SheetCell)],
) {
    assert!(
        !owner_cells.is_empty(),
        "{file_name}: the test has no cell of the file yet; add to its list, as \
         (\"C3\", SheetCell::Number(1.75)), the cells the owner says the file \
         shows, each with its reference, before the test can pass"
    );
    for (reference, cell) in owner_cells {
        assert_eq!(
            cell_at(sheet, reference),
            Some(cell),
            "{file_name}: the cell at {reference}"
        );
    }
}

/// Asserts what "Made by the owner" says of the table of `excel_es.xlsx`,
/// which `excel_en.xlsx`, `libreoffice.xlsx` and `google_sheets.xlsx`
/// repeat: its header; in `Altura`, the height typed `1,75`; in `Fecha`,
/// the date typed `13/05/2024`; in `Hora`, the time typed `14:30`; in
/// `Afectado`, `VERDADERO` and `FALSO` as booleans; in `Código`, the `7`
/// with the format `000` as the number; somewhere, the identifier typed
/// `'001` as its text, and the errors of `=NA()` and `=1/0` as their texts
/// in English; in `Población`, the population merged over two rows in both;
/// and a blank row. And, of every cell that is not empty, that it is a
/// number under `Altura`, a text `dddd-dd-dd` under `Fecha`, a text
/// `dd:dd:dd` under `Hora`, a boolean under `Afectado`, and that no row
/// with an `Individuo` has an empty `Población`.
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
    assert_the_kinds_of_the_columns(file_name, sheet);
    let rows = rows_of_an_individual_with_no_population(sheet);
    assert!(
        rows.is_empty(),
        "{file_name}: rows, from 0 for the header, with an Individuo and no Población, as a merged range lost leaves them: {rows:?}"
    );
}

/// Asserts, of every cell of `sheet` that is not empty, that it is a
/// number under `Altura`, a text `dddd-dd-dd` under `Fecha`, a text
/// `dd:dd:dd` under `Hora` and a boolean under `Afectado`.
fn assert_the_kinds_of_the_columns(file_name: &str, sheet: &Sheet) {
    let kinds: [(&str, &str, IsOfKind); 4] = [
        ("Altura", "a number", |cell| {
            matches!(cell, SheetCell::Number(_))
        }),
        (
            "Fecha",
            "a text dddd-dd-dd",
            |cell| matches!(cell, SheetCell::Text(cell_text) if has_form(cell_text, "dddd-dd-dd")),
        ),
        (
            "Hora",
            "a text dd:dd:dd",
            |cell| matches!(cell, SheetCell::Text(cell_text) if has_form(cell_text, "dd:dd:dd")),
        ),
        ("Afectado", "a boolean", |cell| {
            matches!(cell, SheetCell::Bool(_))
        }),
    ];
    for (header, kind, is_of_kind) in kinds {
        let other_cells = cells_not_of_kind(sheet, header, is_of_kind);
        assert!(
            other_cells.is_empty(),
            "{file_name}: under {header}, cells that are not {kind}: {other_cells:?}"
        );
    }
}

/// The cells the owner typed in `excel_es.xlsx`, sheet `Hoja1`, from A1,
/// each its reference and its cell: B3:B4 merged, G2:G4 formatted `000`,
/// row 5 blank, G6 `=NOD()` and G7 `=1/0`. `excel_en.xlsx` is a copy of
/// it, since an xlsx stores formulas and errors in English whatever the
/// language of Excel.
fn cells_of_the_first_file() -> Vec<(&'static str, SheetCell)> {
    let rows: [[SheetCell; 7]; 7] = [
        HEADER.map(text),
        [
            text("ind1"),
            text("Andalucía"),
            SheetCell::Number(1.75),
            text("2024-05-13"),
            text("14:30:00"),
            SheetCell::Bool(true),
            SheetCell::Number(7.0),
        ],
        [
            text("ind2"),
            text("Castilla y León"),
            SheetCell::Number(1.62),
            text("2024-05-14"),
            text("09:05:00"),
            SheetCell::Bool(false),
            SheetCell::Number(12.0),
        ],
        [
            text("001"),
            text("Castilla y León"),
            SheetCell::Number(1.8),
            text("2024-05-15"),
            text("18:45:00"),
            SheetCell::Bool(true),
            SheetCell::Number(3.0),
        ],
        std::array::from_fn(|_| SheetCell::Empty),
        [
            text("ind4"),
            text("Murcia"),
            SheetCell::Number(1.55),
            text("2024-05-16"),
            text("07:00:00"),
            SheetCell::Bool(false),
            text("#N/A"),
        ],
        [
            text("ind5"),
            text("Murcia"),
            SheetCell::Number(1.7),
            text("2024-05-17"),
            text("12:15:00"),
            SheetCell::Bool(true),
            text("#DIV/0!"),
        ],
    ];
    with_references(&rows)
}

/// The references of Excel of a table from A1, `A1` to `G7`, each with its
/// cell, row after row.
const REFERENCES: [[&str; 7]; 7] = [
    ["A1", "B1", "C1", "D1", "E1", "F1", "G1"],
    ["A2", "B2", "C2", "D2", "E2", "F2", "G2"],
    ["A3", "B3", "C3", "D3", "E3", "F3", "G3"],
    ["A4", "B4", "C4", "D4", "E4", "F4", "G4"],
    ["A5", "B5", "C5", "D5", "E5", "F5", "G5"],
    ["A6", "B6", "C6", "D6", "E6", "F6", "G6"],
    ["A7", "B7", "C7", "D7", "E7", "F7", "G7"],
];

/// `rows`, a table of 7 rows of 7 cells from A1, as a list of the cells,
/// each with its reference.
fn with_references(rows: &[[SheetCell; 7]; 7]) -> Vec<(&'static str, SheetCell)> {
    REFERENCES
        .iter()
        .flatten()
        .copied()
        .zip(rows.iter().flatten().cloned())
        .collect()
}

/// Asserts that `sheet` is named `name` and is a rectangle of 7 rows of 7
/// columns from A1.
fn assert_a_table_of_7_by_7_at_a1(file_name: &str, sheet: &Sheet, name: &str) {
    assert_eq!(
        (
            sheet.name.as_str(),
            sheet.first_row,
            sheet.first_column,
            sheet.num_rows,
            sheet.num_columns
        ),
        (name, 1, 1, 7, 7),
        "{file_name}: the name and the rectangle of the sheet"
    );
}

#[test]
fn excel_es_xlsx_gives_the_cells_of_spanish_excel() {
    let sheet = read_owner_file("excel_es.xlsx").unwrap().unwrap();

    assert_a_table_of_7_by_7_at_a1("excel_es.xlsx", &sheet, "Hoja1");
    assert_the_table_of_the_first_file("excel_es.xlsx", &sheet);
    assert_the_cells_the_owner_sees("excel_es.xlsx", &sheet, &cells_of_the_first_file());
}

#[test]
fn excel_en_xlsx_gives_the_cells_of_english_excel() {
    let sheet = read_owner_file("excel_en.xlsx").unwrap().unwrap();

    assert_a_table_of_7_by_7_at_a1("excel_en.xlsx", &sheet, "Hoja1");
    assert_the_table_of_the_first_file("excel_en.xlsx", &sheet);
    assert_the_cells_the_owner_sees("excel_en.xlsx", &sheet, &cells_of_the_first_file());
}

#[test]
#[ignore = "waits for tests/data/libreoffice.xlsx, made by the owner"]
fn libreoffice_xlsx_gives_the_cells_of_libreoffice_calc() {
    let sheet = read_owner_file("libreoffice.xlsx").unwrap().unwrap();

    assert_the_table_of_the_first_file("libreoffice.xlsx", &sheet);
    // The cells the owner says libreoffice.xlsx shows in LibreOffice, each
    // its reference and its cell, such as ("C3", SheetCell::Number(1.75)).
    let owner_cells: Vec<(&str, SheetCell)> = Vec::new();
    assert_the_cells_the_owner_sees("libreoffice.xlsx", &sheet, &owner_cells);
}

// The file is in the date system of 1904, `date1904="1"`, with A2 saved
// as 43963, which is 13 May 2024 in it. Excel also writes an element
// `<x15:workbookPr chartTrackingRefBase="1"/>` inside `<extLst>` of
// `xl/workbook.xml`, which calamine 0.36.1 (`xlsx/mod.rs`, `read_workbook`)
// takes by its local name for the `workbookPr` of the workbook, and it sets
// the date system back to 1900: A2 was read as 2020-05-12 until table_io
// took the date system from the `workbookPr` of the root itself.
#[test]
fn excel_1904_xlsx_gives_the_date_as_excel_shows_it() {
    let sheet = read_owner_file("excel_1904.xlsx").unwrap().unwrap();

    assert_eq!(
        sheet,
        Sheet {
            name: "Sheet1".to_owned(),
            first_row: 1,
            first_column: 1,
            num_rows: 2,
            num_columns: 2,
            cells: vec![
                text("Fecha"),
                text("Hora"),
                text("2024-05-13"),
                text("14:30:00"),
            ],
        }
    );
}

// The file is a table saved with a password to open it; which table does
// not matter, and nothing is to be added.
#[test]
fn encrypted_xlsx_is_refused_as_encrypted() {
    let read = read_owner_file("encrypted.xlsx").unwrap();

    assert_eq!(read, Err(ReadError::Refused(Refusal::Encrypted)));
}

// The file is a table saved as "Excel 97-2003 Workbook"; which table does
// not matter, and nothing is to be added.
#[test]
fn excel97_xls_is_refused_as_old_excel() {
    let read = read_owner_file("excel97.xls").unwrap();

    assert_eq!(read, Err(ReadError::Refused(Refusal::OldExcel)));
}

// "The refusals", point 5, of the spec: Excel 365 saves the error of a
// spill, A1 `=SEQUENCE(3)` with a value in A2, as `#VALUE!`, which calamine
// reads, and not as `#SPILL!`, so the file is read and nothing is refused.
#[test]
fn spill_xlsx_is_read_with_value() {
    let sheet = read_owner_file("spill.xlsx").unwrap().unwrap();

    assert_eq!(
        sheet,
        Sheet {
            name: "Hoja1".to_owned(),
            first_row: 1,
            first_column: 1,
            num_rows: 2,
            num_columns: 1,
            cells: vec![text("#VALUE!"), SheetCell::Number(5.0)],
        }
    );
}

// The file is the English table, its values pasted, imported into Google
// Sheets and downloaded as .xlsx. The paste lost what the first file has
// and a user of Google Sheets would not: `001` came as the number 1, the
// merge of B3:B4 is gone, leaving B4 empty, and G7 is the text `#¡DIV/0!`
// of the Spanish Excel, pasted, and not an error.
#[test]
fn google_sheets_xlsx_gives_the_cells_of_google_sheets() {
    let sheet = read_owner_file("google_sheets.xlsx").unwrap().unwrap();

    assert_a_table_of_7_by_7_at_a1("google_sheets.xlsx", &sheet, "Sheet1");
    assert_the_kinds_of_the_columns("google_sheets.xlsx", &sheet);
    let mut owner_cells = cells_of_the_first_file();
    for (reference, cell) in &mut owner_cells {
        match *reference {
            "A4" => *cell = SheetCell::Number(1.0),
            "B4" => *cell = SheetCell::Empty,
            "G7" => *cell = text("#¡DIV/0!"),
            _ => {}
        }
    }
    assert_the_cells_the_owner_sees("google_sheets.xlsx", &sheet, &owner_cells);
}
