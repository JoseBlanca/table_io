//! The format of a file found by `import_table` from its first bytes, a
//! file too large, a format the build does not read, the refusals of an
//! xlsx before its rows, and a zip that is not a workbook
//! (`docs/specs/import.md`, "The format" and "The refusals").
//!
//! The tests of a format not built run in the builds of one feature, `cargo
//! test -p table_io --no-default-features --features xlsx` and `--features
//! csv`; the others in every build that reads their format.

#[cfg(test)]
#[expect(
    dead_code,
    reason = "no workbook here is of the 1904 system or has a text cell written by hand"
)]
mod hand_written;

use std::path::PathBuf;

use rust_xlsxwriter::{Workbook, XlsxError};
use table_io::{Format, ImportError, ImportOptions, Refusal, TextOptions, import_table};

#[cfg(feature = "xlsx")]
use crate::hand_written::{parts_of_worksheet, stored_zip, xlsx_of_parts};

/// The limit of bytes of popnei_web, 20 MB.
const MAX_BYTES: u64 = 20_000_000;

/// `MAX_SHEET_CELLS` of popnei_web, the limit its light worker gives.
const MAX_SHEET_CELLS: u32 = 2_000_000;

/// The options of an import with the limit of bytes `max_bytes`,
/// popnei_web's limit of cells, and every option of a text file found from
/// the file.
fn options_with_max_bytes(max_bytes: u64) -> ImportOptions {
    ImportOptions {
        max_bytes,
        max_cells: MAX_SHEET_CELLS,
        text: TextOptions {
            encoding: None,
            separator: None,
            decimal: None,
        },
    }
}

/// The error of a refusal of `format`.
fn refused(format: Format, refusal: Refusal) -> ImportError {
    ImportError::Refused { format, refusal }
}

/// An xlsx written by rust_xlsxwriter, a header `id`, `pop` over one
/// individual.
fn small_xlsx() -> Result<Vec<u8>, XlsxError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "id")?;
    worksheet.write_string(0, 1, "pop")?;
    worksheet.write_string(1, 0, "A")?;
    worksheet.write_string(1, 1, "P1")?;
    workbook.save_to_buffer()
}

/// The bytes of `file_name` in `tests/data/` at the root of the
/// repository.
fn owner_file(file_name: &str) -> std::io::Result<Vec<u8>> {
    std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/data")
            .join(file_name),
    )
}

/// The 22 bytes of an empty zip, its end record alone, which starts with
/// `PK` and the bytes 5 and 6, not 3 and 4.
fn empty_zip() -> Vec<u8> {
    let mut bytes = vec![b'P', b'K', 5, 6];
    bytes.resize(22, 0);
    bytes
}

// Too large, before anything else, with the format found.

#[test]
fn an_xlsx_one_byte_over_the_limit_is_too_large_with_its_size_and_the_limit() {
    let bytes = small_xlsx().unwrap();
    let size = u64::try_from(bytes.len()).unwrap();

    let import = import_table(&bytes, &options_with_max_bytes(size - 1));

    assert_eq!(
        import,
        Err(refused(
            Format::Xlsx,
            Refusal::TooLarge {
                size,
                max_bytes: size - 1,
            }
        ))
    );
}

#[test]
fn an_xlsx_at_the_limit_is_not_too_large() {
    let bytes = small_xlsx().unwrap();
    let size = u64::try_from(bytes.len()).unwrap();

    let import = import_table(&bytes, &options_with_max_bytes(size));

    assert!(
        !matches!(
            import,
            Err(ImportError::Refused {
                refusal: Refusal::TooLarge { .. },
                ..
            })
        ),
        "{import:?}"
    );
}

#[test]
fn a_text_file_of_12_bytes_with_the_limit_11_is_too_large_as_a_text_file() {
    let import = import_table(b"id,pop\nA,P1\n", &options_with_max_bytes(11));

    assert_eq!(
        import,
        Err(refused(
            Format::Text,
            Refusal::TooLarge {
                size: 12,
                max_bytes: 11,
            }
        ))
    );
}

// The format, told by the format a refusal of too large carries, which
// every build gives.

#[test]
fn a_csv_whose_header_starts_with_pk_is_a_text_file() {
    let import = import_table(b"PK,pop\nA,P1\n", &options_with_max_bytes(11));

    assert_eq!(
        import,
        Err(refused(
            Format::Text,
            Refusal::TooLarge {
                size: 12,
                max_bytes: 11,
            }
        ))
    );
}

#[test]
fn an_empty_zip_is_a_text_file() {
    let import = import_table(&empty_zip(), &options_with_max_bytes(21));

    assert_eq!(
        import,
        Err(refused(
            Format::Text,
            Refusal::TooLarge {
                size: 22,
                max_bytes: 21,
            }
        ))
    );
}

#[test]
fn a_compound_file_of_the_old_office_is_of_the_format_xlsx() {
    let bytes = owner_file("excel97.xls").unwrap();
    let size = u64::try_from(bytes.len()).unwrap();

    let import = import_table(&bytes, &options_with_max_bytes(size - 1));

    assert_eq!(
        import,
        Err(refused(
            Format::Xlsx,
            Refusal::TooLarge {
                size,
                max_bytes: size - 1,
            }
        ))
    );
}

// A format not built.

#[cfg(not(feature = "csv"))]
#[test]
fn a_text_file_in_a_build_without_csv_is_a_format_not_built_text() {
    let import = import_table(b"id,pop\nA,P1\n", &options_with_max_bytes(MAX_BYTES));

    assert_eq!(import, Err(refused(Format::Text, Refusal::FormatNotBuilt)));
}

#[cfg(not(feature = "csv"))]
#[test]
fn a_csv_whose_header_starts_with_pk_and_an_empty_zip_in_a_build_without_csv_are_text() {
    for bytes in [b"PK,pop\nA,P1\n".to_vec(), empty_zip()] {
        let import = import_table(&bytes, &options_with_max_bytes(MAX_BYTES));

        assert_eq!(
            import,
            Err(refused(Format::Text, Refusal::FormatNotBuilt)),
            "{bytes:?}"
        );
    }
}

#[cfg(not(feature = "xlsx"))]
#[test]
fn an_xlsx_in_a_build_without_xlsx_is_a_format_not_built_xlsx() {
    let import = import_table(&small_xlsx().unwrap(), &options_with_max_bytes(MAX_BYTES));

    assert_eq!(import, Err(refused(Format::Xlsx, Refusal::FormatNotBuilt)));
}

#[cfg(not(feature = "xlsx"))]
#[test]
fn excel97_xls_in_a_build_without_xlsx_is_a_format_not_built_xlsx() {
    let import = import_table(
        &owner_file("excel97.xls").unwrap(),
        &options_with_max_bytes(MAX_BYTES),
    );

    assert_eq!(import, Err(refused(Format::Xlsx, Refusal::FormatNotBuilt)));
}

// The refusals of an xlsx before its rows.

#[cfg(feature = "xlsx")]
#[test]
fn excel97_xls_is_refused_as_old_excel() {
    let import = import_table(
        &owner_file("excel97.xls").unwrap(),
        &options_with_max_bytes(MAX_BYTES),
    );

    assert_eq!(import, Err(refused(Format::Xlsx, Refusal::OldExcel)));
}

#[cfg(feature = "xlsx")]
#[test]
fn encrypted_xlsx_is_refused_as_encrypted() {
    let import = import_table(
        &owner_file("encrypted.xlsx").unwrap(),
        &options_with_max_bytes(MAX_BYTES),
    );

    assert_eq!(import, Err(refused(Format::Xlsx, Refusal::Encrypted)));
}

#[cfg(feature = "xlsx")]
#[test]
fn an_empty_first_sheet_is_refused_with_its_name() {
    let mut workbook = Workbook::new();
    workbook.add_worksheet().set_name("Datos").unwrap();
    let bytes = workbook.save_to_buffer().unwrap();

    let import = import_table(&bytes, &options_with_max_bytes(MAX_BYTES));

    assert_eq!(
        import,
        Err(refused(
            Format::Xlsx,
            Refusal::EmptySheet {
                sheet: "Datos".to_owned()
            }
        ))
    );
}

#[cfg(feature = "xlsx")]
#[test]
fn a_formula_saved_with_an_error_calamine_does_not_know_is_a_cell_error() {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "id").unwrap();
    worksheet
        .write_formula(
            1,
            0,
            rust_xlsxwriter::Formula::new("=A1").set_result("#GETTING_DATA"),
        )
        .unwrap();
    let bytes = workbook.save_to_buffer().unwrap();

    let import = import_table(&bytes, &options_with_max_bytes(MAX_BYTES));

    assert_eq!(
        import,
        Err(refused(
            Format::Xlsx,
            Refusal::CellError {
                error: "#GETTING_DATA".to_owned()
            }
        ))
    );
}

#[cfg(feature = "xlsx")]
#[test]
fn a_rectangle_from_c2_past_the_limit_of_cells_is_a_sheet_too_large() {
    // C2 to D4 is 3 rows of 2 columns, 6 cells, past a limit of 5; the
    // read stops at D4, the first cell past it.
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(1, 2, "id").unwrap();
    worksheet.write_string(3, 3, "x").unwrap();
    let bytes = workbook.save_to_buffer().unwrap();
    let options = ImportOptions {
        max_bytes: MAX_BYTES,
        max_cells: 5,
        text: options_with_max_bytes(MAX_BYTES).text,
    };

    let import = import_table(&bytes, &options);

    assert_eq!(
        import,
        Err(refused(
            Format::Xlsx,
            Refusal::SheetTooLarge {
                sheet: "Sheet1".to_owned(),
                first_row: 2,
                first_column: 3,
                num_rows: 3,
                num_columns: 2,
            }
        ))
    );
}

#[cfg(feature = "xlsx")]
#[test]
fn the_first_500_bytes_of_an_xlsx_are_unreadable_with_the_zip_crates_message() {
    let bytes = small_xlsx().unwrap();

    let import = import_table(&bytes[..500], &options_with_max_bytes(MAX_BYTES));

    assert_eq!(
        import,
        Err(ImportError::Unreadable(
            "invalid Zip archive: Could not find EOCD".to_owned()
        ))
    );
}

// Not a workbook.

/// The `[Content_Types].xml` of a `.docx` of one paragraph.
#[cfg(feature = "xlsx")]
const DOCX_CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;

/// The package relationships of a `.docx`, whose main part is
/// `word/document.xml`.
#[cfg(feature = "xlsx")]
const DOCX_RELATIONSHIPS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;

/// The document of a `.docx` of one paragraph, `Hola`.
#[cfg(feature = "xlsx")]
const DOCX_DOCUMENT: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hola</w:t></w:r></w:p></w:body></w:document>"#;

#[cfg(feature = "xlsx")]
#[test]
fn a_docx_is_refused_as_not_a_workbook() {
    let bytes = stored_zip(&[
        ("[Content_Types].xml", DOCX_CONTENT_TYPES),
        ("_rels/.rels", DOCX_RELATIONSHIPS),
        ("word/document.xml", DOCX_DOCUMENT),
    ]);

    let import = import_table(&bytes, &options_with_max_bytes(MAX_BYTES));

    assert_eq!(import, Err(refused(Format::Xlsx, Refusal::NotWorkbook)));
}

#[cfg(feature = "xlsx")]
#[test]
fn a_zip_holding_one_csv_is_refused_as_not_a_workbook() {
    let bytes = stored_zip(&[("individuals.csv", "id,pop\nA,P1\n")]);

    let import = import_table(&bytes, &options_with_max_bytes(MAX_BYTES));

    assert_eq!(import, Err(refused(Format::Xlsx, Refusal::NotWorkbook)));
}

#[cfg(feature = "xlsx")]
#[test]
fn an_xlsx_without_its_workbook_is_refused_as_not_a_workbook() {
    let mut parts = parts_of_worksheet("<sheetData/>");
    parts.retain(|(name, _)| name != "xl/workbook.xml");
    let bytes = xlsx_of_parts(&parts);

    let import = import_table(&bytes, &options_with_max_bytes(MAX_BYTES));

    assert_eq!(import, Err(refused(Format::Xlsx, Refusal::NotWorkbook)));
}
