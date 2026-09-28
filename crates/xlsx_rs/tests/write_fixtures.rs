//! Writes the xlsx files of `tests/data/` that the tests make rather than
//! the owner. Each test is ignored and run by hand, and the file it writes
//! is committed:
//!
//! ```text
//! cargo test -p xlsx_rs --test write_fixtures -- --ignored
//! ```

use std::error::Error;
use std::path::PathBuf;

use rust_xlsxwriter::{DocProperties, ExcelDateTime, Format, Formula, Workbook, XlsxError};

/// The path of `file_name` in `tests/data/` at the root of the repository.
fn data_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/data")
        .join(file_name)
}

/// A workbook with a fixed date of creation, so that the file written is
/// the same at each run.
fn workbook_of_fixed_date() -> Result<Workbook, XlsxError> {
    let mut workbook = Workbook::new();
    let written_on = ExcelDateTime::from_ymd(2026, 9, 28)?;
    workbook.set_properties(&DocProperties::new().set_creation_datetime(&written_on));
    Ok(workbook)
}

/// Writes `workbook` as `file_name` in `tests/data/`.
fn save(workbook: &mut Workbook, file_name: &str) -> Result<(), Box<dyn Error>> {
    let path = data_path(file_name);
    std::fs::create_dir_all(path.parent().ok_or("tests/data/ has no parent")?)?;
    std::fs::write(&path, workbook.save_to_buffer()?)?;
    Ok(())
}

/// `written.xlsx`, which the test of the package reads until the owner's
/// `excel_en.xlsx` exists ("The package, built" of `docs/specs/read.md`):
/// a header and four individuals, with a text, a number, a boolean, a date,
/// a population merged over two rows, and the height of `ind3` not written,
/// a cell with nothing in the file.
#[test]
#[ignore = "writes tests/data/written.xlsx; run by hand"]
fn write_written_xlsx() {
    let mut workbook = workbook_of_fixed_date().unwrap();
    let worksheet = workbook.add_worksheet().set_name("Individuos").unwrap();
    let date_format = Format::new().set_num_format("dd/mm/yyyy");

    for (column, name) in (0u16..).zip(["Individuo", "Población", "Altura", "Fecha", "Afectado"]) {
        worksheet.write_string(0, column, name).unwrap();
    }
    // 45425 is 13 May 2024 in Excel's system of 1900.
    let rows = [
        ("ind1", Some(1.75), 45425.0, true),
        ("ind2", Some(1.62), 45426.0, false),
        ("ind3", None, 45427.0, true),
        ("ind4", Some(1.55), 45428.0, false),
    ];
    for (row, (individual, height, date, is_affected)) in (1u32..).zip(rows) {
        worksheet.write_string(row, 0, individual).unwrap();
        if let Some(height) = height {
            worksheet.write_number(row, 2, height).unwrap();
        }
        worksheet
            .write_number_with_format(row, 3, date, &date_format)
            .unwrap();
        worksheet.write_boolean(row, 4, is_affected).unwrap();
    }
    worksheet
        .merge_range(1, 1, 2, 1, "Andalucía", &Format::new())
        .unwrap();
    worksheet.write_string(3, 1, "Murcia").unwrap();
    worksheet.write_string(4, 1, "Murcia").unwrap();

    save(&mut workbook, "written.xlsx").unwrap();
}

/// `empty_first_sheet.xlsx`, for the refusal `emptySheet` in the test of
/// the package: a first sheet `Notas` with no value, and a table on the
/// second, `Individuos`.
#[test]
#[ignore = "writes tests/data/empty_first_sheet.xlsx; run by hand"]
fn write_empty_first_sheet_xlsx() {
    let mut workbook = workbook_of_fixed_date().unwrap();
    workbook.add_worksheet().set_name("Notas").unwrap();
    let individuals = workbook.add_worksheet().set_name("Individuos").unwrap();
    individuals.write_string(0, 0, "Individuo").unwrap();
    individuals.write_string(1, 0, "ind1").unwrap();

    save(&mut workbook, "empty_first_sheet.xlsx").unwrap();
}

/// `table_at_c2.xlsx`, for a rectangle whose first row and first column
/// differ in the test of the package: a header and two individuals from
/// C2, with a text, a number and a boolean each.
#[test]
#[ignore = "writes tests/data/table_at_c2.xlsx; run by hand"]
fn write_table_at_c2_xlsx() {
    let mut workbook = workbook_of_fixed_date().unwrap();
    let worksheet = workbook.add_worksheet().set_name("Individuos").unwrap();
    for (column, name) in (2u16..).zip(["Individuo", "Altura", "Afectado"]) {
        worksheet.write_string(1, column, name).unwrap();
    }
    let rows = [("ind1", 1.75, true), ("ind2", 1.62, false)];
    for (row, (individual, height, is_affected)) in (2u32..).zip(rows) {
        worksheet.write_string(row, 2, individual).unwrap();
        worksheet.write_number(row, 3, height).unwrap();
        worksheet.write_boolean(row, 4, is_affected).unwrap();
    }

    save(&mut workbook, "table_at_c2.xlsx").unwrap();
}

/// `getting_data.xlsx`, for the refusal `cellError` in the test of the
/// package: a header, and a formula saved with `#GETTING_DATA`, an error
/// calamine does not know, at A2.
#[test]
#[ignore = "writes tests/data/getting_data.xlsx; run by hand"]
fn write_getting_data_xlsx() {
    let mut workbook = workbook_of_fixed_date().unwrap();
    let worksheet = workbook.add_worksheet().set_name("Individuos").unwrap();
    worksheet.write_string(0, 0, "Individuo").unwrap();
    worksheet
        .write_formula(1, 0, Formula::new("=A1").set_result("#GETTING_DATA"))
        .unwrap();

    save(&mut workbook, "getting_data.xlsx").unwrap();
}

/// `wide_table_at_c2.xlsx`, for the refusal `sheetTooLarge` in the test of
/// the package, read there with a limit of 16 cells: a header of 5 columns
/// from C2 and 5 individuals, so that the rectangle at the refusal, rows 2
/// to 5 and columns C to G, has a first row, a first column, a number of
/// rows and a number of columns that all differ.
#[test]
#[ignore = "writes tests/data/wide_table_at_c2.xlsx; run by hand"]
fn write_wide_table_at_c2_xlsx() {
    let mut workbook = workbook_of_fixed_date().unwrap();
    let worksheet = workbook.add_worksheet().set_name("Individuos").unwrap();
    let header = ["Individuo", "Población", "Altura", "Peso", "Afectado"];
    for (column, name) in (2u16..).zip(header) {
        worksheet.write_string(1, column, name).unwrap();
    }
    let rows = [
        ("ind1", "Andalucía", 1.75, 70.0, true),
        ("ind2", "Andalucía", 1.62, 58.5, false),
        ("ind3", "Murcia", 1.80, 81.0, true),
        ("ind4", "Murcia", 1.55, 52.0, false),
        ("ind5", "Murcia", 1.68, 64.5, true),
    ];
    for (row, (individual, population, height, weight, is_affected)) in (2u32..).zip(rows) {
        worksheet.write_string(row, 2, individual).unwrap();
        worksheet.write_string(row, 3, population).unwrap();
        worksheet.write_number(row, 4, height).unwrap();
        worksheet.write_number(row, 5, weight).unwrap();
        worksheet.write_boolean(row, 6, is_affected).unwrap();
    }

    save(&mut workbook, "wide_table_at_c2.xlsx").unwrap();
}
