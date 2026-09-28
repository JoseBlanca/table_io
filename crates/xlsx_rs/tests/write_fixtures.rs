//! Writes the xlsx files of `tests/data/` that the tests make rather than
//! the owner. Each test is ignored and run by hand, and the file it writes
//! is committed:
//!
//! ```text
//! cargo test -p xlsx_rs --test write_fixtures -- --ignored
//! ```

use std::path::PathBuf;

use rust_xlsxwriter::{DocProperties, ExcelDateTime, Format, Workbook};

/// The path of `file_name` in `tests/data/` at the root of the repository.
fn data_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/data")
        .join(file_name)
}

/// `written.xlsx`, which the test of the package reads until the owner's
/// `excel_en.xlsx` exists ("The package, built" of `docs/specs/read.md`):
/// a header and four individuals, with a text, a number, a boolean, a date
/// and a population merged over two rows.
#[test]
#[ignore = "writes tests/data/written.xlsx; run by hand"]
fn write_written_xlsx() {
    let mut workbook = Workbook::new();
    // A fixed date of creation, so that the file is the same at each run.
    let written_on = ExcelDateTime::from_ymd(2026, 9, 28).unwrap();
    workbook.set_properties(&DocProperties::new().set_creation_datetime(&written_on));
    let worksheet = workbook.add_worksheet().set_name("Individuos").unwrap();
    let date_format = Format::new().set_num_format("dd/mm/yyyy");

    for (column, name) in (0u16..).zip(["Individuo", "Población", "Altura", "Fecha", "Afectado"]) {
        worksheet.write_string(0, column, name).unwrap();
    }
    // 45425 is 13 May 2024 in Excel's system of 1900.
    let rows = [
        ("ind1", 1.75, 45425.0, true),
        ("ind2", 1.62, 45426.0, false),
        ("ind3", 1.80, 45427.0, true),
        ("ind4", 1.55, 45428.0, false),
    ];
    for (row, (individual, height, date, is_affected)) in (1u32..).zip(rows) {
        worksheet.write_string(row, 0, individual).unwrap();
        worksheet.write_number(row, 2, height).unwrap();
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

    let path = data_path("written.xlsx");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, workbook.save_to_buffer().unwrap()).unwrap();
}
