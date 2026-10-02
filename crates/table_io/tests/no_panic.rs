//! No file makes `import_table` panic: the no-panic test of "How it is
//! verified" of `docs/specs/import.md`.
//!
//! A small xlsx written by rust_xlsxwriter and a small CSV in Windows-1252
//! with a quoted cell over two lines are each cut short at every length
//! and have each of their bytes changed in turn to every other of the 256
//! values, and every copy, under `std::panic::catch_unwind`, gives a
//! table, a refusal or an error. The copies of a file are shared among
//! threads, each taking one position of a byte at a time, since the xlsx
//! alone makes some hundreds of thousands of them.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};

use table_io::{ImportOptions, TextOptions, import_table};

/// The limit of bytes of popnei_web, 20 MB, far above every copy.
const MAX_BYTES: u64 = 20_000_000;

/// `MAX_SHEET_CELLS` of popnei_web, the limit its light worker gives.
const MAX_SHEET_CELLS: u32 = 2_000_000;

/// The options of every import: popnei_web's limits, and every option of
/// a text file found from the file.
const OPTIONS: ImportOptions = ImportOptions {
    max_bytes: MAX_BYTES,
    max_cells: MAX_SHEET_CELLS,
    text: TextOptions {
        encoding: None,
        separator: None,
        decimal: None,
    },
};

/// A copy of a file that made `import_table` panic.
#[derive(Debug)]
#[expect(
    dead_code,
    reason = "the fields are read by the message of the assertion, through Debug"
)]
enum Panicked {
    /// The file cut short to its first `length` bytes.
    CutShort {
        /// The number of bytes kept.
        length: usize,
    },
    /// The file with its byte at `position`, from 0, changed to `byte`.
    Changed {
        /// The position of the byte changed, from 0.
        position: usize,
        /// The value the byte was changed to.
        byte: u8,
    },
}

/// Whether `import_table` panics on `bytes`; whatever it returns, a table
/// or an error, is not looked at.
fn panics_on(bytes: &[u8]) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        // Any table, refusal or error passes; only a panic fails.
        let _ = import_table(bytes, &OPTIONS);
    }))
    .is_err()
}

/// The copies of `file` cut short at `position` and with the byte at
/// `position` changed to each of the 255 other values that panicked; none
/// when `file` has no byte there.
fn copies_at_position_that_panic(file: &[u8], position: usize) -> Vec<Panicked> {
    let (Some(kept), Some(&original)) = (file.get(..position), file.get(position)) else {
        return Vec::new();
    };
    let mut panicked = Vec::new();
    if panics_on(kept) {
        panicked.push(Panicked::CutShort { length: position });
    }
    let mut copy = file.to_vec();
    for byte in (0..=u8::MAX).filter(|&byte| byte != original) {
        if let Some(changed) = copy.get_mut(position) {
            *changed = byte;
        }
        if panics_on(&copy) {
            panicked.push(Panicked::Changed { position, byte });
        }
    }
    panicked
}

/// The copies of `file` that panicked, among it cut short at each length
/// below its own, from 0 bytes, and with each of its bytes changed to each
/// of the 255 other values, imported on as many threads as the machine
/// has, each taking the next position of a byte; an error when a thread
/// panicked outside `catch_unwind`.
fn copies_that_panic(file: &[u8]) -> std::thread::Result<Vec<Panicked>> {
    let next_position = AtomicUsize::new(0);
    let num_threads = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..num_threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut panicked = Vec::new();
                    loop {
                        let position = next_position.fetch_add(1, Ordering::Relaxed);
                        if position >= file.len() {
                            return panicked;
                        }
                        panicked.extend(copies_at_position_that_panic(file, position));
                    }
                })
            })
            .collect();
        workers
            .into_iter()
            .map(std::thread::ScopedJoinHandle::join)
            .collect::<std::thread::Result<Vec<_>>>()
            .map(|per_thread| per_thread.into_iter().flatten().collect())
    })
}

/// A small xlsx as rust_xlsxwriter writes it: a header and two individuals,
/// with a text, a number, a boolean, a date and a cell left empty.
#[cfg(feature = "xlsx")]
fn small_xlsx() -> Result<Vec<u8>, rust_xlsxwriter::XlsxError> {
    use rust_xlsxwriter::{DocProperties, ExcelDateTime, Format, Workbook};

    let mut workbook = Workbook::new();
    let written_on = ExcelDateTime::from_ymd(2026, 10, 2)?;
    workbook.set_properties(&DocProperties::new().set_creation_datetime(&written_on));
    let worksheet = workbook.add_worksheet();
    let date_format = Format::new().set_num_format("yyyy-mm-dd");
    for (column, name) in (0u16..).zip(["Individuo", "Población", "Altura", "Fecha", "Afectado"]) {
        worksheet.write_string(0, column, name)?;
    }
    worksheet.write_string(1, 0, "ind1")?;
    worksheet.write_string(1, 1, "Andalucía")?;
    worksheet.write_number(1, 2, 1.75)?;
    worksheet.write_number_with_format(1, 3, 45425.0, &date_format)?;
    worksheet.write_boolean(1, 4, true)?;
    worksheet.write_string(2, 0, "ind2")?;
    worksheet.write_string(2, 1, "Murcia")?;
    worksheet.write_number_with_format(2, 3, 45426.0, &date_format)?;
    worksheet.write_boolean(2, 4, false)?;
    workbook.save_to_buffer()
}

/// A small CSV as a Spanish Excel writes it, in Windows-1252, `;`
/// between the cells, the decimal comma and CRLF, with a quoted cell over
/// two lines, a quoted cell with quotes and the separator in it, and an
/// empty cell: `ó` is the byte 0xF3, `í` 0xED.
#[cfg(feature = "csv")]
const SMALL_CSV: &[u8] = b"Individuo;Poblaci\xF3n;Altura;Notas\r\n\
ind1;Andaluc\xEDa;1,75;\"dos\r\nl\xEDneas\"\r\n\
ind2;Murcia;;\"\"\"x\"\";y\"\r\n";

#[cfg(feature = "xlsx")]
#[test]
fn no_copy_of_a_small_xlsx_cut_short_or_with_a_byte_changed_panics() {
    let xlsx = small_xlsx().unwrap();
    assert!(
        import_table(&xlsx, &OPTIONS).is_ok(),
        "the xlsx itself is a table"
    );

    let panicked = copies_that_panic(&xlsx).unwrap();

    let num_copies = xlsx.len().checked_mul(256).unwrap();
    println!("{num_copies} copies of an xlsx of {} bytes", xlsx.len());
    assert!(
        panicked.is_empty(),
        "{} copies panicked: {panicked:?}",
        panicked.len()
    );
}

#[cfg(feature = "csv")]
#[test]
fn no_copy_of_a_small_csv_cut_short_or_with_a_byte_changed_panics() {
    assert!(
        import_table(SMALL_CSV, &OPTIONS).is_ok(),
        "the CSV itself is a table"
    );

    let panicked = copies_that_panic(SMALL_CSV).unwrap();

    let num_copies = SMALL_CSV.len().checked_mul(256).unwrap();
    println!("{num_copies} copies of a CSV of {} bytes", SMALL_CSV.len());
    assert!(
        panicked.is_empty(),
        "{} copies panicked: {panicked:?}",
        panicked.len()
    );
}
