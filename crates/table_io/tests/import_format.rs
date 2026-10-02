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

// A zip that holds no workbook is not a workbook whatever its other parts
// hold: its package relationships are read first, and no other part is
// read or bounded once they name no workbook in it.

/// The method of compression of a part of a zip: 8, deflate, which the
/// zip crate reads, and 12, bzip2, which the build of table_io does not.
#[cfg(feature = "xlsx")]
const DEFLATE: u16 = 8;
/// See [`DEFLATE`].
#[cfg(feature = "xlsx")]
const BZIP2: u16 = 12;

/// The flag of a part of a zip saved with a password, bit 0 of its
/// general purpose flags.
#[cfg(feature = "xlsx")]
const ENCRYPTED: u16 = 1;

/// A zip of one part named `name`, whose bytes in the zip are `stored`,
/// compressed by `method` with the flags `flags`, and which unzip to
/// `size` bytes of CRC-32 `crc`: a local header and its bytes, the central
/// directory that lists it, and the record that ends the zip.
#[cfg(feature = "xlsx")]
fn zip_of_one_part(
    name: &str,
    method: u16,
    flags: u16,
    stored: &[u8],
    size: u32,
    crc: u32,
) -> Result<Vec<u8>, std::num::TryFromIntError> {
    let name_length = u16::try_from(name.len())?;
    let stored_size = u32::try_from(stored.len())?;
    // Version 2.0, the flags and the method, a time of 0 and 1 January 1980,
    // then the checksum and the two sizes.
    let mut fields = Vec::new();
    fields.extend(flags.to_le_bytes());
    fields.extend(method.to_le_bytes());
    fields.extend([0, 0, 0x21, 0]);
    fields.extend(crc.to_le_bytes());
    fields.extend(stored_size.to_le_bytes());
    fields.extend(size.to_le_bytes());
    fields.extend(name_length.to_le_bytes());

    let mut zip = Vec::new();
    zip.extend(0x0403_4b50_u32.to_le_bytes());
    zip.extend([20, 0]);
    zip.extend(&fields);
    zip.extend(0_u16.to_le_bytes());
    zip.extend(name.as_bytes());
    zip.extend(stored);
    let directory_offset = u32::try_from(zip.len())?;

    let mut directory = Vec::new();
    directory.extend(0x0201_4b50_u32.to_le_bytes());
    directory.extend([20, 0, 20, 0]);
    directory.extend(&fields);
    // No extra field, no comment, disk 0, no attributes, offset 0.
    directory.extend([0; 16]);
    directory.extend(name.as_bytes());
    let directory_size = u32::try_from(directory.len())?;

    zip.extend(directory);
    zip.extend(0x0605_4b50_u32.to_le_bytes());
    zip.extend([0, 0, 0, 0, 1, 0, 1, 0]);
    zip.extend(directory_size.to_le_bytes());
    zip.extend(directory_offset.to_le_bytes());
    zip.extend(0_u16.to_le_bytes());
    Ok(zip)
}

/// The number of copies of 258 bytes after the first two in
/// [`deflated_zeros_and_commas`], which make 310,000,160 bytes.
#[cfg(feature = "xlsx")]
const NUM_COPIES: u32 = 1_201_551;

/// A deflate stream, of one block of the fixed codes, of `0,` repeated to
/// 2 + 258 × [`NUM_COPIES`] bytes: the literals `0` and `,`, then each copy
/// of 258 bytes from 2 bytes back, 13 bits, and the end of the block; about
/// 2 MB for 310 MB.
#[cfg(feature = "xlsx")]
#[expect(
    clippy::arithmetic_side_effects,
    reason = "a bit buffer of fewer than 64 bits, shifted by fewer than 64"
)]
fn deflated_zeros_and_commas() -> Vec<u8> {
    let mut stream = Vec::new();
    let mut buffer: u64 = 0;
    let mut num_bits: u32 = 0;
    // A Huffman code is written from its highest bit, the other fields
    // from their lowest.
    let mut push = |value: u64, length: u32, is_code: bool| {
        let bits = if is_code {
            value.reverse_bits() >> (64 - length)
        } else {
            value
        };
        buffer |= bits << num_bits;
        num_bits += length;
        while num_bits >= 8 {
            stream.push(buffer.to_le_bytes()[0]);
            buffer >>= 8;
            num_bits -= 8;
        }
    };
    // The last block, of the fixed codes.
    push(1, 1, false);
    push(1, 2, false);
    // `0` and `,`, the literals 0x30 and 0x2C, codes 0x30 + literal.
    push(0x60, 8, true);
    push(0x5C, 8, true);
    for _ in 0..NUM_COPIES {
        // The length 258, code 285, and the distance 2, code 1.
        push(0xC5, 8, true);
        push(1, 5, true);
    }
    // The end of the block, code 256, and the last bits.
    push(0, 7, true);
    push(0, 7, false);
    stream
}

#[cfg(feature = "xlsx")]
#[test]
fn a_zip_of_one_csv_that_unzips_past_the_bound_of_a_part_is_not_a_workbook() {
    // 310,000,160 bytes, past the 300,000,000 of a part; its checksum is
    // 0, since a read that reached it would have passed the bound first.
    let size = 2 + 258 * NUM_COPIES;
    let bytes = zip_of_one_part(
        "geno.csv",
        DEFLATE,
        0,
        &deflated_zeros_and_commas(),
        size,
        0,
    )
    .unwrap();

    let import = import_table(&bytes, &options_with_max_bytes(MAX_BYTES));

    assert_eq!(import, Err(refused(Format::Xlsx, Refusal::NotWorkbook)));
}

#[cfg(feature = "xlsx")]
#[test]
fn a_zip_of_one_part_in_bzip2_is_not_a_workbook() {
    let bytes = zip_of_one_part("geno.csv", BZIP2, 0, b"BZh9", 4, 0).unwrap();

    let import = import_table(&bytes, &options_with_max_bytes(MAX_BYTES));

    assert_eq!(import, Err(refused(Format::Xlsx, Refusal::NotWorkbook)));
}

#[cfg(feature = "xlsx")]
#[test]
fn a_zip_of_one_part_saved_with_a_password_is_not_a_workbook() {
    let bytes = zip_of_one_part("geno.csv", 0, ENCRYPTED, &[0; 20], 8, 0).unwrap();

    let import = import_table(&bytes, &options_with_max_bytes(MAX_BYTES));

    assert_eq!(import, Err(refused(Format::Xlsx, Refusal::NotWorkbook)));
}

// What the types of the import give a caller besides their fields.

#[test]
fn the_decimal_mark_of_an_xlsx_is_the_point_and_of_a_text_file_its_own() {
    let xlsx = table_io::HowRead::Xlsx {
        sheet: "Hoja1".to_owned(),
    };
    let text = table_io::HowRead::Text(table_io::TextRead {
        encoding: table_io::FoundEncoding::Windows1252,
        separator: table_io::Separator::Semicolon,
        decimal: table_io::DecimalMark::Comma,
        undecoded_line: None,
    });

    assert_eq!(
        (xlsx.decimal(), text.decimal()),
        (table_io::DecimalMark::Point, table_io::DecimalMark::Comma)
    );
}

#[test]
fn the_default_options_of_a_text_file_find_all_three_from_the_file() {
    assert_eq!(
        TextOptions::default(),
        TextOptions {
            encoding: None,
            separator: None,
            decimal: None,
        }
    );
}

#[test]
fn an_import_error_is_written_in_english_with_its_fields() {
    let cases = [
        (
            refused(
                Format::Text,
                Refusal::TooLarge {
                    size: 25,
                    max_bytes: 20,
                },
            ),
            "a text file refused: too large, 25 bytes, past the limit of 20 bytes",
        ),
        (
            refused(Format::Xlsx, Refusal::FormatNotBuilt),
            "an xlsx refused: a format this build of table_io does not read",
        ),
        (
            refused(
                Format::Xlsx,
                Refusal::HeaderError {
                    row: 6,
                    column: 5,
                    error: "#VALUE!".to_owned(),
                },
            ),
            "an xlsx refused: the error #VALUE! in the header, at row 6 and column 5 of the sheet",
        ),
        (
            refused(
                Format::Text,
                Refusal::RaggedRow {
                    line: 3,
                    expected: 2,
                    found: 1,
                    separator: table_io::Separator::Comma,
                },
            ),
            "a text file refused: line 3 has 1 cells where the header has 2, split at ','",
        ),
        (
            refused(
                Format::Xlsx,
                Refusal::DuplicateIndividual {
                    name: "A".to_owned(),
                    first_row: 2,
                    second_row: 3,
                },
            ),
            "an xlsx refused: the individual 'A' is in the rows 2 and 3",
        ),
        (
            ImportError::Unreadable("Invalid checksum".to_owned()),
            "an unreadable file: Invalid checksum",
        ),
    ];

    for (import_error, text) in cases {
        let as_error: &dyn std::error::Error = &import_error;
        assert_eq!(as_error.to_string(), text);
    }
}
