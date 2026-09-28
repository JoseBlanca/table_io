//! The files of `tests/data/` that `tests/write_fixtures.rs` writes and
//! that other tests read: `written.xlsx`, which the test of the package
//! reads, with the same cells asserted here; and `individuals_10000.xlsx`,
//! the sheet of 10,000 rows and 20 columns of popnei_web's test ("How it
//! is verified" of `docs/specs/read.md`).

use std::path::PathBuf;

use xlsx_rs::{Sheet, SheetCell, read_first_sheet};

/// `MAX_SHEET_CELLS` of popnei_web, the limit its light worker gives.
const MAX_SHEET_CELLS: u32 = 2_000_000;

/// The bytes of `file_name` in `tests/data/` at the root of the repository.
fn data_file(file_name: &str) -> std::io::Result<Vec<u8>> {
    std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/data")
            .join(file_name),
    )
}

/// A text cell of `cell_text`.
fn text(cell_text: &str) -> SheetCell {
    SheetCell::Text(cell_text.to_owned())
}

#[test]
fn written_xlsx_is_read_with_every_cell() {
    let bytes = data_file("written.xlsx").unwrap();

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    // B3, in the population merged over B2 and B3, is "Andalucía"; the
    // height of ind3 is not in the file.
    assert_eq!(
        sheet,
        Sheet {
            name: "Individuos".to_owned(),
            first_row: 1,
            first_column: 1,
            num_rows: 5,
            num_columns: 5,
            cells: vec![
                text("Individuo"),
                text("Población"),
                text("Altura"),
                text("Fecha"),
                text("Afectado"),
                text("ind1"),
                text("Andalucía"),
                SheetCell::Number(1.75),
                text("2024-05-13"),
                SheetCell::Bool(true),
                text("ind2"),
                text("Andalucía"),
                SheetCell::Number(1.62),
                text("2024-05-14"),
                SheetCell::Bool(false),
                text("ind3"),
                text("Murcia"),
                SheetCell::Empty,
                text("2024-05-15"),
                SheetCell::Bool(true),
                text("ind4"),
                text("Murcia"),
                SheetCell::Number(1.55),
                text("2024-05-16"),
                SheetCell::Bool(false),
            ],
        }
    );
}

/// The number of columns of `individuals_10000.xlsx`.
const NUM_COLUMNS: usize = 20;

#[test]
fn individuals_10000_xlsx_is_read_as_10001_rows_of_20_columns() {
    let bytes = data_file("individuals_10000.xlsx").unwrap();

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!(sheet.name, "Individuos");
    assert_eq!(
        (
            sheet.first_row,
            sheet.first_column,
            sheet.num_rows,
            sheet.num_columns
        ),
        (1, 1, 10_001, 20)
    );
    assert_eq!(sheet.cells.len(), 200_020);
    let row = |index: usize| &sheet.cells[index * NUM_COLUMNS..(index + 1) * NUM_COLUMNS];
    assert_eq!(
        row(0),
        [
            "Individuo",
            "Población",
            "Localidad",
            "Sexo",
            "Edad",
            "Altura",
            "Peso",
            "Latitud",
            "Longitud",
            "Altitud",
            "Fecha de muestreo",
            "Afectado",
            "Tratado",
            "Hijos",
            "Presión sistólica",
            "Presión diastólica",
            "Colesterol",
            "Glucosa",
            "Lote",
            "Observaciones",
        ]
        .map(text)
    );
    assert_eq!(
        row(1),
        [
            text("ind00001"),
            text("Murcia"),
            text("Localidad 2"),
            text("M"),
            SheetCell::Number(19.0),
            SheetCell::Number(1.51),
            SheetCell::Number(50.1),
            SheetCell::Number(36.01),
            SheetCell::Number(-8.99),
            SheetCell::Number(1.0),
            text("2024-01-02"),
            SheetCell::Bool(false),
            SheetCell::Bool(false),
            SheetCell::Number(1.0),
            SheetCell::Number(101.0),
            SheetCell::Number(61.0),
            SheetCell::Number(150.1),
            SheetCell::Number(70.1),
            text("L002"),
            SheetCell::Empty,
        ]
    );
    // The weight of every 50th individual is missing, and every 97th has
    // an observation.
    assert_eq!(row(50)[0], text("ind00050"));
    assert_eq!(row(50)[6], SheetCell::Empty);
    assert_eq!(row(97)[0], text("ind00097"));
    assert_eq!(row(97)[19], text("revisar la muestra"));
    assert_eq!(
        row(10_000),
        [
            text("ind10000"),
            text("Andalucía"),
            text("Localidad 1"),
            text("H"),
            SheetCell::Number(58.0),
            SheetCell::Number(1.5),
            SheetCell::Empty,
            SheetCell::Number(38.0),
            SheetCell::Number(-5.0),
            SheetCell::Number(0.0),
            text("2024-04-28"),
            SheetCell::Bool(false),
            SheetCell::Bool(false),
            SheetCell::Number(0.0),
            SheetCell::Number(140.0),
            SheetCell::Number(60.0),
            SheetCell::Number(150.0),
            SheetCell::Number(110.0),
            text("L001"),
            SheetCell::Empty,
        ]
    );
}
