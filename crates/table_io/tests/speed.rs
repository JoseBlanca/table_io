//! The time `import_table` takes over a CSV of 100,000 rows and 50
//! columns, against the target of less than one second of goal 7 of
//! `docs/objectives.md`. The test is ignored, since it measures and
//! asserts no time, and is run by hand in release:
//!
//! ```text
//! cargo test -p table_io --release --test speed -- --ignored --nocapture
//! ```
//!
//! It prints the size of the file and the time of the import, and writes
//! the file as `speed.csv` in cargo's folder of temporary files of the
//! tests, `target/tmp/`, for the same import to be timed through the
//! package under node.

#![cfg(feature = "csv")]

use std::time::Instant;

use table_io::{
    ColumnValues, DecimalMark, FoundEncoding, HowRead, ImportOptions, Separator, TextOptions,
    TextRead, import_table,
};

/// The rows of the table, below the header.
const NUM_ROWS: u32 = 100_000;

/// The integer columns, then the float ones, then the text ones, 49 in
/// all beside the column of the names.
const NUM_INTEGERS: u32 = 20;
const NUM_FLOATS: u32 = 20;
const NUM_TEXTS: u32 = 9;

/// A CSV as a Spanish Excel saves it, UTF-8 with `;` between the cells and
/// `,` for the decimals, lines ended by `\r\n`: a header `id;int1;...;
/// int20;float1;...;float20;text1;...;text9`, then 100,000 rows, each a
/// name, `ind1` to `ind100000`, 20 integers from 0 to 99,999, 20 floats of
/// three decimals from `0,000` to `999,999`, and 9 texts `pop0` to
/// `pop19`; no value is missing.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "the row is at most 100,000 and the column at most 49, so every product and sum is below 10^7, and every divisor is a constant other than 0"
)]
fn csv_of_the_table() -> Vec<u8> {
    let mut csv = String::from("id");
    for integer in 1..=NUM_INTEGERS {
        csv.push_str(&format!(";int{integer}"));
    }
    for float in 1..=NUM_FLOATS {
        csv.push_str(&format!(";float{float}"));
    }
    for text in 1..=NUM_TEXTS {
        csv.push_str(&format!(";text{text}"));
    }
    csv.push_str("\r\n");
    for row in 1..=NUM_ROWS {
        csv.push_str(&format!("ind{row}"));
        for column in 1..=NUM_INTEGERS {
            csv.push_str(&format!(";{}", (row * 7 + column * 13) % 100_000));
        }
        for column in 1..=NUM_FLOATS {
            let whole = (row + column) % 1_000;
            let thousandths = (row * column) % 1_000;
            csv.push_str(&format!(";{whole},{thousandths:03}"));
        }
        for column in 1..=NUM_TEXTS {
            csv.push_str(&format!(";pop{}", (row + column) % 20));
        }
        csv.push_str("\r\n");
    }
    csv.into_bytes()
}

#[test]
#[ignore = "measures the time of an import; run by hand with --release"]
fn a_csv_of_100000_rows_and_50_columns_is_imported_and_timed() {
    let csv = csv_of_the_table();
    std::fs::write(
        std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("speed.csv"),
        &csv,
    )
    .unwrap();
    let options = ImportOptions {
        max_bytes: 100_000_000,
        max_cells: u32::MAX,
        text: TextOptions {
            encoding: None,
            separator: None,
            decimal: None,
        },
    };

    let start = Instant::now();
    let table = import_table(&csv, &options).unwrap();
    let elapsed = start.elapsed();

    println!(
        "{} bytes imported in {:.3} s",
        csv.len(),
        elapsed.as_secs_f64()
    );
    assert_eq!(
        table.read,
        HowRead::Text(TextRead {
            encoding: FoundEncoding::Utf8,
            separator: Separator::Semicolon,
            decimal: DecimalMark::Comma,
            undecoded_line: None,
        })
    );
    assert_eq!(table.names.names.len(), 100_000);
    assert_eq!(table.names.names[99_999], "ind100000");
    assert_eq!(table.columns.len(), 49);
    // Row 1 of int1 is 1 × 7 + 1 × 13, of float1 2,001, of text1 pop2.
    let ColumnValues::Integer(int1) = &table.columns[0].values else {
        panic!("int1 is not an integer column");
    };
    assert_eq!(int1[0], Some(20));
    let ColumnValues::Float(float1) = &table.columns[20].values else {
        panic!("float1 is not a float column");
    };
    assert_eq!(float1[0], Some(2.001));
    let ColumnValues::Text(text1) = &table.columns[40].values else {
        panic!("text1 is not a text column");
    };
    assert_eq!(text1[0].as_deref(), Some("pop2"));
}
