//! No file makes `read_first_sheet` panic ("How it is verified", "No file
//! makes it panic", of `docs/specs/read.md`): a panic is a trap in the
//! wasm, which ends popnei_web's light worker.
//!
//! Two files, an xlsx written here and a compound file of the old Office
//! with `EncryptedPackage` among its bytes, are read cut short at every
//! length from 0 bytes to their whole size, and with each of their bytes in
//! turn replaced by its complement, the byte with every bit flipped. Every
//! copy has to return, a sheet, a refusal or an error, whichever; the tests
//! assert nothing of which. calamine is compiled without its checks of
//! overflow in these builds too, as in the package (`docs/architecture.md`,
//! section 6), so that a panic here is one the package has.

use std::panic;

use rust_xlsxwriter::{DocProperties, ExcelDateTime, Format, Formula, Workbook, XlsxError};
use xlsx_rs::read_first_sheet;

/// `MAX_SHEET_CELLS` of popnei_web, the limit its light worker gives.
const MAX_SHEET_CELLS: u32 = 2_000_000;

/// The eight bytes every compound file of the old Office starts with.
const COMPOUND_FILE_MARK: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// An xlsx with every kind the spec names for this test: a hidden first
/// sheet, then a sheet with a text, a number, a boolean, a date, a formula
/// saved with its value, and a population merged over two rows. Its date
/// of creation is fixed, so that its bytes, and the copy a failure names,
/// are the same at each run.
fn xlsx_of_every_kind() -> Result<Vec<u8>, XlsxError> {
    let mut workbook = Workbook::new();
    let written_on = ExcelDateTime::from_ymd(2026, 9, 28)?;
    workbook.set_properties(&DocProperties::new().set_creation_datetime(&written_on));

    let lists = workbook.add_worksheet().set_name("Listas")?;
    lists.write_string(0, 0, "a list of a form")?;
    lists.set_hidden(true);

    let individuals = workbook.add_worksheet().set_name("Individuos")?;
    for (column, name) in (0u16..).zip([
        "Individuo",
        "Población",
        "Altura",
        "Fecha",
        "Afectado",
        "Doble",
    ]) {
        individuals.write_string(0, column, name)?;
    }
    individuals.write_string(1, 0, "ind1")?;
    individuals.write_string(2, 0, "ind2")?;
    individuals.merge_range(1, 1, 2, 1, "Andalucía", &Format::new())?;
    individuals.write_number(1, 2, 1.75)?;
    individuals.write_number(2, 2, 1.62)?;
    let date_format = Format::new().set_num_format("dd/mm/yyyy");
    // 45425 is 13 May 2024 in Excel's system of 1900.
    individuals.write_number_with_format(1, 3, 45425.0, &date_format)?;
    individuals.write_number_with_format(2, 3, 45426.0, &date_format)?;
    individuals.write_boolean(1, 4, true)?;
    individuals.write_boolean(2, 4, false)?;
    individuals.write_formula(1, 5, Formula::new("=C2*2").set_result("3.5"))?;
    individuals.write_formula(2, 5, Formula::new("=C3*2").set_result("3.24"))?;
    // rust_xlsxwriter shows the active sheet, the first one unless told.
    individuals.set_active(true);

    workbook.save_to_buffer()
}

/// A compound file as an xlsx saved with a password is, as far as xlsx_rs
/// looks: the mark, zeros, and the name of the part `EncryptedPackage` in
/// UTF-16 little-endian at byte 600, as a compound file writes its names,
/// then zeros up to 1,024 bytes.
fn encrypted_compound_file() -> Vec<u8> {
    let mut bytes = COMPOUND_FILE_MARK.to_vec();
    bytes.resize(600, 0);
    bytes.extend("EncryptedPackage".encode_utf16().flat_map(u16::to_le_bytes));
    bytes.resize(1024, 0);
    bytes
}

/// Whether `read_first_sheet` returns for `bytes`, whatever it returns,
/// rather than panicking.
fn returns_for(bytes: &[u8]) -> bool {
    panic::catch_unwind(|| {
        // What the read returns is not asserted: any sheet, refusal or error
        // will do.
        let _read = read_first_sheet(bytes, MAX_SHEET_CELLS);
    })
    .is_ok()
}

/// The lengths from 0 to the whole size of `bytes` at which the copy cut
/// short panics.
fn lengths_that_panic(bytes: &[u8]) -> Vec<usize> {
    (0..=bytes.len())
        .filter(|&length| bytes.get(..length).is_some_and(|cut| !returns_for(cut)))
        .collect()
}

/// The positions of the bytes of `bytes` whose complement, in a copy with
/// that byte alone changed, panics.
fn flipped_bytes_that_panic(bytes: &[u8]) -> Vec<usize> {
    (0..bytes.len())
        .filter(|&position| {
            let copy: Vec<u8> = (0..)
                .zip(bytes)
                .map(|(other_position, &byte)| {
                    if other_position == position {
                        !byte
                    } else {
                        byte
                    }
                })
                .collect();
            !returns_for(&copy)
        })
        .collect()
}

#[test]
fn the_xlsx_of_every_kind_is_a_sheet_when_whole() {
    // So that the copies below are copies of a file calamine reads.
    let bytes = xlsx_of_every_kind().unwrap();

    let sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();

    assert_eq!(sheet.name, "Individuos");
    assert_eq!((sheet.num_rows, sheet.num_columns), (3, 6));
}

#[test]
fn the_xlsx_of_every_kind_cut_short_at_any_length_does_not_panic() {
    let bytes = xlsx_of_every_kind().unwrap();

    let lengths = lengths_that_panic(&bytes);

    assert!(
        lengths.is_empty(),
        "the xlsx of {} bytes panics when cut at these lengths: {lengths:?}",
        bytes.len()
    );
}

#[test]
fn the_xlsx_of_every_kind_with_any_byte_flipped_does_not_panic() {
    let bytes = xlsx_of_every_kind().unwrap();

    let positions = flipped_bytes_that_panic(&bytes);

    assert!(
        positions.is_empty(),
        "the xlsx of {} bytes panics with the byte at these positions, from 0, flipped: {positions:?}",
        bytes.len()
    );
}

#[test]
fn the_encrypted_compound_file_cut_short_at_any_length_does_not_panic() {
    let bytes = encrypted_compound_file();

    let lengths = lengths_that_panic(&bytes);

    assert!(
        lengths.is_empty(),
        "the compound file of {} bytes panics when cut at these lengths: {lengths:?}",
        bytes.len()
    );
}

#[test]
fn the_encrypted_compound_file_with_any_byte_flipped_does_not_panic() {
    let bytes = encrypted_compound_file();

    let positions = flipped_bytes_that_panic(&bytes);

    assert!(
        positions.is_empty(),
        "the compound file of {} bytes panics with the byte at these positions, from 0, flipped: {positions:?}",
        bytes.len()
    );
}
