//! The table `import_table` makes of the cells of an xlsx, "The cells of an
//! xlsx, as the table takes them", "The rows" and "The refusals" of
//! `docs/specs/import.md`, and the guess of the type of each column, "The
//! type of a column" of `docs/specs/values.md`.
//!
//! Each case is an xlsx written by the test with rust_xlsxwriter, its cells
//! at the places said, and the literal table or refusal it gives: every row
//! of the table of the cells of a sheet of "How it is verified" of
//! `docs/specs/import.md`; the order of the refusals, each pair of them one
//! xlsx can hold; the cases of the guess of `docs/specs/values.md` that an
//! xlsx can hold; the property that the same table with its rows in another
//! order gives the same types; and the owner's files of `docs/specs/read.md`
//! as the tables they show.

#![cfg(feature = "xlsx")]

use std::path::PathBuf;

use rust_xlsxwriter::{ExcelDateTime, Formula, Workbook, XlsxError};
use table_io::{
    Column, ColumnType, ColumnValues, DecimalMark, Format, HowRead, ImportError, ImportOptions,
    NameColumn, Refusal, Table, TextOptions, convert_column, import_table,
};

/// The limit of bytes of popnei_web, 20 MB.
const MAX_BYTES: u64 = 20_000_000;

/// `MAX_SHEET_CELLS` of popnei_web, the limit its light worker gives.
const MAX_SHEET_CELLS: u32 = 2_000_000;

/// 2^63, the first whole number past the integers.
const TWO_TO_THE_63: f64 = 9_223_372_036_854_775_808.0;

/// A cell the test writes into a sheet.
#[derive(Debug, Clone, Copy)]
enum Written<'text> {
    /// A text cell.
    Text(&'text str),
    /// A number cell.
    Number(f64),
    /// A boolean cell.
    Boolean(bool),
    /// A formula whose saved value is this error of Excel, which the file
    /// holds as an error cell, `t="e"`.
    Error(&'text str),
    /// A date, year, month and day, as a number with the format
    /// `yyyy-mm-dd`.
    Date(u16, u8, u8),
}

use Written::{Boolean, Date, Error, Number, Text};

/// The row and the column, from 0, of a reference of Excel such as `C5`,
/// or None for a text that is not one.
fn place_of(reference: &str) -> Option<(u32, u16)> {
    let digits_start = reference.find(|character: char| character.is_ascii_digit())?;
    let (letters, digits) = reference.split_at(digits_start);
    let column = letters.bytes().try_fold(0_u16, |column, letter| {
        let letter_number = u16::from(letter.checked_sub(b'A')?).checked_add(1)?;
        column.checked_mul(26)?.checked_add(letter_number)
    })?;
    let row: u32 = digits.parse().ok()?;
    Some((row.checked_sub(1)?, column.checked_sub(1)?))
}

/// An xlsx of one sheet, `Sheet1`, with `cells` at their references.
fn xlsx_of(cells: &[(&str, Written<'_>)]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let placed_cells = cells
        .iter()
        .map(|&(reference, cell)| {
            let (row, column) = place_of(reference).ok_or("not a reference of Excel")?;
            Ok((row, column, cell))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    Ok(xlsx_at(&placed_cells)?)
}

/// An xlsx of one sheet, `Sheet1`, with `cells` each at its row and its
/// column, from 0.
fn xlsx_at(cells: &[(u32, u16, Written<'_>)]) -> Result<Vec<u8>, XlsxError> {
    let mut workbook = Workbook::new();
    let date_format = rust_xlsxwriter::Format::new().set_num_format("yyyy-mm-dd");
    let worksheet = workbook.add_worksheet();
    for &(row, column, cell) in cells {
        match cell {
            Text(cell_text) => worksheet.write_string(row, column, cell_text)?,
            Number(number) => worksheet.write_number(row, column, number)?,
            Boolean(is_true) => worksheet.write_boolean(row, column, is_true)?,
            Error(error) => {
                worksheet.write_formula(row, column, Formula::new("=NA()").set_result(error))?
            }
            Date(year, month, day) => worksheet.write_datetime_with_format(
                row,
                column,
                &ExcelDateTime::from_ymd(year, month, day)?,
                &date_format,
            )?,
        };
    }
    workbook.save_to_buffer()
}

/// The options of an import with popnei_web's limits and every option of a
/// text file found from the file.
fn options() -> ImportOptions {
    ImportOptions {
        max_bytes: MAX_BYTES,
        max_cells: MAX_SHEET_CELLS,
        text: TextOptions {
            encoding: None,
            separator: None,
            decimal: None,
        },
    }
}

/// What `import_table` gives for an xlsx of `cells`, or the error of the
/// test when the xlsx is not written.
fn import(
    cells: &[(&str, Written<'_>)],
) -> Result<Result<Table, ImportError>, Box<dyn std::error::Error>> {
    Ok(import_table(&xlsx_of(cells)?, &options()))
}

/// The error of a refusal of an xlsx.
fn refused(refusal: Refusal) -> Result<Table, ImportError> {
    Err(ImportError::Refused {
        format: Format::Xlsx,
        refusal,
    })
}

/// A table of the sheet `Sheet1`, its first column headed `header` at the
/// column `number` of the sheet with `names`, and `columns`.
fn table(header: &str, number: u32, names: &[&str], columns: Vec<Column>) -> Table {
    Table {
        names: NameColumn {
            header: header.to_owned(),
            number,
            names: names.iter().map(|&name| name.to_owned()).collect(),
        },
        columns,
        read: HowRead::Xlsx {
            sheet: "Sheet1".to_owned(),
        },
    }
}

/// A column named `name` at the column `number` of the sheet.
fn column(name: &str, number: u32, values: ColumnValues) -> Column {
    Column {
        name: name.to_owned(),
        number,
        values,
    }
}

/// The values of a text column.
fn texts(values: &[Option<&str>]) -> ColumnValues {
    ColumnValues::Text(
        values
            .iter()
            .map(|value| value.map(str::to_owned))
            .collect(),
    )
}

/// The values of an integer column.
fn integers(values: &[Option<i64>]) -> ColumnValues {
    ColumnValues::Integer(values.to_vec())
}

/// The values of a float column.
fn floats(values: &[Option<f64>]) -> ColumnValues {
    ColumnValues::Float(values.to_vec())
}

/// The values of a boolean column.
fn booleans(values: &[Option<bool>]) -> ColumnValues {
    ColumnValues::Boolean(values.to_vec())
}

// The table of the cells of a sheet of "How it is verified" of
// docs/specs/import.md, row by row, each cell from A1 unless said.

#[test]
fn whole_numbers_in_the_first_column_are_names_and_pop_is_text() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("pop")),
        ("A2", Number(1.0)),
        ("B2", Text("P1")),
        ("A3", Number(2.0)),
        ("B3", Text("P2")),
        ("A4", Number(3.0)),
        ("B4", Text("P1")),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "id",
            1,
            &["1", "2", "3"],
            vec![column(
                "pop",
                2,
                texts(&[Some("P1"), Some("P2"), Some("P1")])
            )],
        ))
    );
}

#[test]
fn number_cells_and_a_text_with_the_point_are_a_float_column() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("h")),
        ("A2", Text("A")),
        ("B2", Number(1.75)),
        ("A3", Text("B")),
        ("B3", Text("1.8")),
        ("A4", Text("C")),
        ("B4", Number(1.69)),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "id",
            1,
            &["A", "B", "C"],
            vec![column("h", 2, floats(&[Some(1.75), Some(1.8), Some(1.69)]))],
        ))
    );
}

#[test]
fn whole_number_cells_and_a_text_of_a_whole_number_are_an_integer_column() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("g")),
        ("A2", Text("A")),
        ("B2", Number(1.0)),
        ("A3", Text("B")),
        ("B3", Text("1")),
        ("A4", Text("C")),
        ("B4", Number(0.0)),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "id",
            1,
            &["A", "B", "C"],
            vec![column("g", 2, integers(&[Some(1), Some(1), Some(0)]))],
        ))
    );
}

#[test]
fn a_text_with_the_comma_among_number_cells_is_a_text_column() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("h")),
        ("A2", Text("A")),
        ("B2", Text("1,75")),
        ("A3", Text("B")),
        ("B3", Number(1.8)),
        ("A4", Text("C")),
        ("B4", Number(1.7)),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "id",
            1,
            &["A", "B", "C"],
            vec![column(
                "h",
                2,
                texts(&[Some("1,75"), Some("1.8"), Some("1.7")])
            )],
        ))
    );
}

#[test]
fn boolean_cells_are_a_boolean_column() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("ok")),
        ("A2", Text("A")),
        ("B2", Boolean(true)),
        ("A3", Text("B")),
        ("B3", Boolean(false)),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "id",
            1,
            &["A", "B"],
            vec![column("ok", 2, booleans(&[Some(true), Some(false)]))],
        ))
    );
}

#[test]
fn a_header_of_the_number_2024_names_the_column_2024() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Number(2024.0)),
        ("A2", Text("A")),
        ("B2", Number(1.0)),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "id",
            1,
            &["A"],
            vec![column("2024", 2, integers(&[Some(1)]))],
        ))
    );
}

#[test]
fn texts_are_trimmed_and_na_and_spaces_alone_are_missing() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("pop")),
        ("A2", Text("A")),
        ("B2", Text(" P1 ")),
        ("A3", Text("B")),
        ("B3", Text("NA")),
        ("A4", Text("C")),
        ("B4", Text("  ")),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "id",
            1,
            &["A", "B", "C"],
            vec![column("pop", 2, texts(&[Some("P1"), None, None]))],
        ))
    );
}

#[test]
fn a_blank_row_between_the_header_and_an_individual_is_skipped() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("pop")),
        ("A3", Text("A")),
        ("B3", Text("P1")),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "id",
            1,
            &["A"],
            vec![column("pop", 2, texts(&[Some("P1")]))],
        ))
    );
}

#[test]
fn a_third_column_with_no_name_and_a_cell_of_spaces_is_dropped() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("pop")),
        ("A2", Text("A")),
        ("B2", Text("P1")),
        ("C2", Text("  ")),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "id",
            1,
            &["A"],
            vec![column("pop", 2, texts(&[Some("P1")]))],
        ))
    );
}

#[test]
fn a_column_with_no_name_and_a_value_at_c5_is_refused_as_column_d() {
    let import = import(&[
        ("C5", Text("id")),
        ("E5", Text("pop")),
        ("C6", Text("A")),
        ("D6", Text("x")),
        ("E6", Text("P1")),
    ])
    .unwrap();

    assert_eq!(import, refused(Refusal::UnnamedColumn { column: 4 }));
}

#[test]
fn an_individual_with_no_name_below_a_header_at_a5_is_refused_at_sheet_row_6() {
    let import = import(&[("A5", Text("id")), ("B5", Text("pop")), ("B6", Text("P1"))]).unwrap();

    assert_eq!(import, refused(Refusal::EmptyIndividual { row: 6 }));
}

#[test]
fn the_number_1_and_the_text_1_are_a_duplicate_individual() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("pop")),
        ("A2", Number(1.0)),
        ("B2", Text("P1")),
        ("A3", Text("1")),
        ("B3", Text("P2")),
    ])
    .unwrap();

    assert_eq!(
        import,
        refused(Refusal::DuplicateIndividual {
            name: "1".to_owned(),
            first_row: 2,
            second_row: 3,
        })
    );
}

#[test]
fn a_header_alone_is_empty() {
    let import = import(&[("A1", Text("id")), ("B1", Text("pop"))]).unwrap();

    assert_eq!(import, refused(Refusal::Empty));
}

#[test]
fn an_error_in_the_header_is_a_header_error_with_its_row_and_column() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Error("#N/A")),
        ("A2", Text("A")),
        ("B2", Error("#N/A")),
    ])
    .unwrap();

    assert_eq!(
        import,
        refused(Refusal::HeaderError {
            row: 1,
            column: 2,
            error: "#N/A".to_owned(),
        })
    );
}

#[test]
fn an_error_typed_with_spaces_in_a_header_at_c6_is_a_header_error_at_row_6_column_5() {
    let import = import(&[
        ("C6", Text("id")),
        ("D6", Text("pop")),
        ("E6", Text(" #VALUE! ")),
        ("C7", Text("A")),
        ("D7", Text("P1")),
        ("E7", Number(1.0)),
    ])
    .unwrap();

    assert_eq!(
        import,
        refused(Refusal::HeaderError {
            row: 6,
            column: 5,
            error: "#VALUE!".to_owned(),
        })
    );
}

#[test]
fn an_error_naming_the_first_column_of_a_header_alone_is_a_header_error_before_empty() {
    let import = import(&[("A1", Error("#REF!")), ("B1", Text("pop"))]).unwrap();

    assert_eq!(
        import,
        refused(Refusal::HeaderError {
            row: 1,
            column: 1,
            error: "#REF!".to_owned(),
        })
    );
}

#[test]
fn an_error_in_a_text_column_is_missing() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("pop")),
        ("A2", Text("A")),
        ("B2", Error("#VALUE!")),
        ("A3", Text("B")),
        ("B3", Text("P1")),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "id",
            1,
            &["A", "B"],
            vec![column("pop", 2, texts(&[None, Some("P1")]))],
        ))
    );
}

#[test]
fn an_error_in_a_column_of_heights_is_missing_and_the_column_a_float() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("h")),
        ("A2", Text("A")),
        ("B2", Error("#DIV/0!")),
        ("A3", Text("B")),
        ("B3", Number(1.5)),
        ("A4", Text("C")),
        ("B4", Number(1.6)),
        ("A5", Text("D")),
        ("B5", Number(1.7)),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "id",
            1,
            &["A", "B", "C", "D"],
            vec![column(
                "h",
                2,
                floats(&[None, Some(1.5), Some(1.6), Some(1.7)])
            )],
        ))
    );
}

#[test]
fn the_five_other_errors_of_excel_are_missing() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("h")),
        ("A2", Text("A")),
        ("B2", Error("#NAME?")),
        ("A3", Text("B")),
        ("B3", Error("#NULL!")),
        ("A4", Text("C")),
        ("B4", Error("#NUM!")),
        ("A5", Text("D")),
        ("B5", Error("#REF!")),
        ("A6", Text("E")),
        ("B6", Error("#VALUE!")),
        ("A7", Text("F")),
        ("B7", Number(1.5)),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "id",
            1,
            &["A", "B", "C", "D", "E", "F"],
            vec![column(
                "h",
                2,
                floats(&[None, None, None, None, None, Some(1.5)])
            )],
        ))
    );
}

#[test]
fn errors_in_the_first_column_are_names() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("h")),
        ("A2", Error("#N/A")),
        ("B2", Number(1.5)),
        ("A3", Error("#REF!")),
        ("B3", Number(2.5)),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "id",
            1,
            &["#N/A", "#REF!"],
            vec![column("h", 2, floats(&[Some(1.5), Some(2.5)]))],
        ))
    );
}

#[test]
fn a_boolean_cell_in_the_first_column_is_the_name_true() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("ok")),
        ("A2", Boolean(true)),
        ("B2", Number(1.0)),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "id",
            1,
            &["TRUE"],
            vec![column("ok", 2, integers(&[Some(1)]))],
        ))
    );
}

// The other rules of the rows, as an xlsx holds them.

#[test]
fn a_column_with_no_name_of_na_and_dashes_in_the_middle_is_dropped() {
    let import = import(&[
        ("A1", Text("id")),
        ("C1", Text("pop")),
        ("A2", Text("A")),
        ("B2", Text("NA")),
        ("C2", Text("P1")),
        ("A3", Text("B")),
        ("B3", Text("-")),
        ("C3", Text("P2")),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "id",
            1,
            &["A", "B"],
            vec![column("pop", 3, texts(&[Some("P1"), Some("P2")]))],
        ))
    );
}

#[test]
fn a_row_of_spaces_above_the_header_is_blank_and_the_header_is_its_next_row() {
    let import = import(&[
        ("B2", Text("  ")),
        ("B3", Text("id")),
        ("C3", Text("pop")),
        ("C4", Text("P1")),
    ])
    .unwrap();

    assert_eq!(import, refused(Refusal::EmptyIndividual { row: 4 }));
}

#[test]
fn a_sheet_whose_only_values_are_spaces_is_empty() {
    let import = import(&[("B2", Text("  ")), ("C3", Text("\t"))]).unwrap();

    assert_eq!(import, refused(Refusal::Empty));
}

#[test]
fn the_first_name_that_repeats_one_before_it_is_the_duplicate_column() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("a")),
        ("C1", Text("b")),
        ("D1", Text("b")),
        ("E1", Text("a")),
        ("A2", Text("A")),
    ])
    .unwrap();

    assert_eq!(
        import,
        refused(Refusal::DuplicateColumn {
            name: "b".to_owned(),
            first_column: 3,
            second_column: 4,
        })
    );
}

#[test]
fn the_name_of_the_first_column_repeated_is_a_duplicate_column() {
    let import = import(&[
        ("B2", Text("pop")),
        ("C2", Text("pop")),
        ("B3", Text("A")),
        ("C3", Text("P1")),
    ])
    .unwrap();

    assert_eq!(
        import,
        refused(Refusal::DuplicateColumn {
            name: "pop".to_owned(),
            first_column: 2,
            second_column: 3,
        })
    );
}

#[test]
fn names_of_two_cases_are_two_columns() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("Pop")),
        ("C1", Text("pop")),
        ("A2", Text("A")),
        ("B2", Number(1.0)),
        ("C2", Number(2.0)),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "id",
            1,
            &["A"],
            vec![
                column("Pop", 2, integers(&[Some(1)])),
                column("pop", 3, integers(&[Some(2)])),
            ],
        ))
    );
}

#[test]
fn an_empty_name_of_the_first_column_and_na_as_a_name_of_a_column_are_kept() {
    let import = import(&[("B1", Text("NA")), ("A2", Text("A")), ("B2", Text("x"))]).unwrap();

    assert_eq!(
        import,
        Ok(table(
            "",
            1,
            &["A"],
            vec![column("NA", 2, texts(&[Some("x")]))],
        ))
    );
}

#[test]
fn a_table_at_c6_numbers_its_columns_from_3_and_its_rows_from_6() {
    let import = import(&[
        ("C6", Text("id")),
        ("D6", Text("pop")),
        ("E6", Text("h")),
        ("C7", Text("A")),
        ("D7", Text("P1")),
        ("E7", Number(1.5)),
        ("C8", Text("A")),
        ("D8", Text("P2")),
        ("E8", Number(1.6)),
    ])
    .unwrap();

    assert_eq!(
        import,
        refused(Refusal::DuplicateIndividual {
            name: "A".to_owned(),
            first_row: 7,
            second_row: 8,
        })
    );
}

#[test]
fn a_table_at_c6_gives_its_columns_of_the_sheet() {
    let import = import(&[
        ("C6", Text("id")),
        ("D6", Text("pop")),
        ("E6", Text("h")),
        ("C7", Text("A")),
        ("D7", Text("P1")),
        ("E7", Number(1.5)),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "id",
            3,
            &["A"],
            vec![
                column("pop", 4, texts(&[Some("P1")])),
                column("h", 5, floats(&[Some(1.5)])),
            ],
        ))
    );
}

#[test]
fn texts_and_names_keep_accents_emoji_and_line_breaks() {
    let import = import(&[
        ("A1", Text("Individuo")),
        ("B1", Text("Población")),
        ("A2", Text("ind😀1")),
        ("B2", Text("Castilla\ny León")),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "Individuo",
            1,
            &["ind😀1"],
            vec![column("Población", 2, texts(&[Some("Castilla\ny León")]))],
        ))
    );
}

// The order of the refusals of "The refusals" of docs/specs/import.md,
// each pair one xlsx can hold, the first in the list given.

#[test]
fn a_cell_error_comes_before_a_header_error() {
    let import = import(&[
        ("A1", Error("#REF!")),
        ("B1", Text("pop")),
        ("A2", Text("A")),
        ("B2", Error("#GETTING_DATA")),
    ])
    .unwrap();

    assert_eq!(
        import,
        refused(Refusal::CellError {
            error: "#GETTING_DATA".to_owned(),
        })
    );
}

#[test]
fn a_sheet_too_large_comes_before_a_header_error() {
    let bytes = xlsx_of(&[
        ("A1", Error("#REF!")),
        ("B1", Text("pop")),
        ("A2", Text("A")),
        ("B2", Text("P1")),
    ])
    .unwrap();
    let options = ImportOptions {
        max_bytes: MAX_BYTES,
        max_cells: 3,
        text: options().text,
    };

    let import = import_table(&bytes, &options);

    assert_eq!(
        import,
        refused(Refusal::SheetTooLarge {
            sheet: "Sheet1".to_owned(),
            first_row: 1,
            first_column: 1,
            num_rows: 2,
            num_columns: 2,
        })
    );
}

#[test]
fn of_two_header_errors_the_first_by_position_is_given() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Error("#NUM!")),
        ("C1", Error("#NAME?")),
        ("A2", Text("A")),
    ])
    .unwrap();

    assert_eq!(
        import,
        refused(Refusal::HeaderError {
            row: 1,
            column: 2,
            error: "#NUM!".to_owned(),
        })
    );
}

/// The cells of a header error at B1 over the refusal of each later kind
/// one xlsx can hold beside it, with the name of that refusal.
fn cells_of_a_header_error_and_a_later_refusal()
-> Vec<(&'static str, Vec<(&'static str, Written<'static>)>)> {
    vec![
        (
            "an unnamed column in the run at the end of the header",
            vec![
                ("A1", Text("id")),
                ("B1", Error("#N/A")),
                ("A2", Text("A")),
                ("C2", Text("x")),
            ],
        ),
        (
            "an unnamed column elsewhere",
            vec![
                ("A1", Text("id")),
                ("B1", Error("#N/A")),
                ("D1", Text("pop")),
                ("A2", Text("A")),
                ("C2", Text("x")),
            ],
        ),
        (
            "a duplicate column",
            vec![
                ("A1", Text("id")),
                ("B1", Error("#N/A")),
                ("C1", Text("pop")),
                ("D1", Text("pop")),
                ("A2", Text("A")),
            ],
        ),
        (
            "an empty individual",
            vec![("A1", Text("id")), ("B1", Error("#N/A")), ("B2", Text("x"))],
        ),
        (
            "a duplicate individual",
            vec![
                ("A1", Text("id")),
                ("B1", Error("#N/A")),
                ("A2", Text("A")),
                ("A3", Text("A")),
            ],
        ),
    ]
}

#[test]
fn a_header_error_comes_before_every_refusal_of_the_rows() {
    for (later_refusal, cells) in cells_of_a_header_error_and_a_later_refusal() {
        let import = import(&cells).unwrap();

        assert_eq!(
            import,
            refused(Refusal::HeaderError {
                row: 1,
                column: 2,
                error: "#N/A".to_owned(),
            }),
            "a header error and {later_refusal}"
        );
    }
}

#[test]
fn empty_comes_before_a_duplicate_column() {
    let import = import(&[("A1", Text("id")), ("B1", Text("pop")), ("C1", Text("pop"))]).unwrap();

    assert_eq!(import, refused(Refusal::Empty));
}

#[test]
fn an_unnamed_column_at_the_end_comes_before_one_elsewhere_to_its_left() {
    let import = import(&[
        ("A1", Text("id")),
        ("C1", Text("pop")),
        ("A2", Text("A")),
        ("B2", Text("x")),
        ("C2", Text("P1")),
        ("D2", Text("y")),
    ])
    .unwrap();

    assert_eq!(import, refused(Refusal::UnnamedColumn { column: 4 }));
}

#[test]
fn an_unnamed_column_at_the_end_comes_before_a_duplicate_column() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("pop")),
        ("C1", Text("pop")),
        ("A2", Text("A")),
        ("D2", Text("x")),
    ])
    .unwrap();

    assert_eq!(import, refused(Refusal::UnnamedColumn { column: 4 }));
}

#[test]
fn an_unnamed_column_at_the_end_comes_before_an_empty_individual() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("pop")),
        ("B2", Text("P1")),
        ("C2", Text("x")),
    ])
    .unwrap();

    assert_eq!(import, refused(Refusal::UnnamedColumn { column: 3 }));
}

#[test]
fn an_unnamed_column_at_the_end_comes_before_a_duplicate_individual() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("pop")),
        ("A2", Text("A")),
        ("A3", Text("A")),
        ("C3", Text("x")),
    ])
    .unwrap();

    assert_eq!(import, refused(Refusal::UnnamedColumn { column: 3 }));
}

#[test]
fn of_the_run_at_the_end_the_first_column_with_a_value_is_refused() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("pop")),
        ("A2", Text("A")),
        ("D2", Text("x")),
        ("A3", Text("B")),
        ("C3", Text("y")),
    ])
    .unwrap();

    assert_eq!(import, refused(Refusal::UnnamedColumn { column: 3 }));
}

#[test]
fn an_unnamed_column_comes_before_a_duplicate_column_to_its_left() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("pop")),
        ("C1", Text("pop")),
        ("E1", Text("h")),
        ("A2", Text("A")),
        ("D2", Text("x")),
    ])
    .unwrap();

    assert_eq!(import, refused(Refusal::UnnamedColumn { column: 4 }));
}

#[test]
fn an_unnamed_column_comes_before_an_empty_individual() {
    let import = import(&[("A1", Text("id")), ("C1", Text("pop")), ("B2", Text("x"))]).unwrap();

    assert_eq!(import, refused(Refusal::UnnamedColumn { column: 2 }));
}

#[test]
fn an_unnamed_column_comes_before_a_duplicate_individual() {
    let import = import(&[
        ("A1", Text("id")),
        ("C1", Text("pop")),
        ("A2", Text("A")),
        ("B2", Text("x")),
        ("A3", Text("A")),
    ])
    .unwrap();

    assert_eq!(import, refused(Refusal::UnnamedColumn { column: 2 }));
}

#[test]
fn a_duplicate_column_comes_before_an_empty_individual() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("pop")),
        ("C1", Text("pop")),
        ("B2", Text("x")),
    ])
    .unwrap();

    assert_eq!(
        import,
        refused(Refusal::DuplicateColumn {
            name: "pop".to_owned(),
            first_column: 2,
            second_column: 3,
        })
    );
}

#[test]
fn a_duplicate_column_comes_before_a_duplicate_individual() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("pop")),
        ("C1", Text("pop")),
        ("A2", Text("A")),
        ("A3", Text("A")),
    ])
    .unwrap();

    assert_eq!(
        import,
        refused(Refusal::DuplicateColumn {
            name: "pop".to_owned(),
            first_column: 2,
            second_column: 3,
        })
    );
}

#[test]
fn an_empty_individual_in_an_earlier_row_comes_before_a_duplicate_individual() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("pop")),
        ("B2", Text("P1")),
        ("A3", Text("A")),
        ("A4", Text("A")),
    ])
    .unwrap();

    assert_eq!(import, refused(Refusal::EmptyIndividual { row: 2 }));
}

#[test]
fn a_duplicate_individual_in_an_earlier_row_comes_before_an_empty_individual() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("pop")),
        ("A2", Text("A")),
        ("A3", Text("A")),
        ("B4", Text("P1")),
    ])
    .unwrap();

    assert_eq!(
        import,
        refused(Refusal::DuplicateIndividual {
            name: "A".to_owned(),
            first_row: 2,
            second_row: 3,
        })
    );
}

// The guess of the type, "The type of a column" and "The cases" of
// docs/specs/values.md, as an xlsx holds them.

/// What `import_table` gives for an xlsx of a header `id`, `x` and one
/// individual for each of `values`, named `A`, `B` and on, its value of `x`
/// at column B; None for a cell left empty.
fn import_column(
    values: &[Option<Written<'static>>],
) -> Result<Result<Table, ImportError>, Box<dyn std::error::Error>> {
    const NAMES: [&str; 6] = ["A", "B", "C", "D", "E", "F"];
    const ROWS: [(&str, &str); 6] = [
        ("A2", "B2"),
        ("A3", "B3"),
        ("A4", "B4"),
        ("A5", "B5"),
        ("A6", "B6"),
        ("A7", "B7"),
    ];
    let mut cells = vec![("A1", Text("id")), ("B1", Text("x"))];
    for ((name, (name_place, value_place)), value) in NAMES.iter().zip(ROWS).zip(values) {
        cells.push((name_place, Text(name)));
        if let Some(value) = value {
            cells.push((value_place, *value));
        }
    }
    import(&cells)
}

/// The table of [`import_column`] whose column `x` holds `values`, one
/// for each of its first rows.
fn table_of_column(num_rows: usize, values: ColumnValues) -> Result<Table, ImportError> {
    let names: Vec<&str> = ["A", "B", "C", "D", "E", "F"]
        .into_iter()
        .take(num_rows)
        .collect();
    Ok(table("id", 1, &names, vec![column("x", 2, values)]))
}

#[test]
fn texts_of_codes_with_zeros_are_an_integer_column_and_the_zeros_lost() {
    let import = import_column(&[Some(Text("001")), Some(Text("002")), Some(Text("010"))]).unwrap();

    assert_eq!(
        import,
        table_of_column(3, integers(&[Some(1), Some(2), Some(10)]))
    );
}

#[test]
fn texts_1_and_1_point_0_are_a_float_column() {
    let import = import_column(&[Some(Text("1")), Some(Text("1.0"))]).unwrap();

    assert_eq!(import, table_of_column(2, floats(&[Some(1.0), Some(1.0)])));
}

#[test]
fn number_cells_1_and_2_are_an_integer_column_as_shown_1_or_1_point_0() {
    let import = import_column(&[Some(Number(1.0)), Some(Number(2.0))]).unwrap();

    assert_eq!(import, table_of_column(2, integers(&[Some(1), Some(2)])));
}

#[test]
fn number_cells_0_and_1_are_an_integer_column_not_a_boolean_one() {
    let import = import_column(&[Some(Number(0.0)), Some(Number(1.0))]).unwrap();

    assert_eq!(import, table_of_column(2, integers(&[Some(0), Some(1)])));
}

#[test]
fn texts_true_false_and_na_are_a_boolean_column() {
    let import =
        import_column(&[Some(Text("TRUE")), Some(Text("false")), Some(Text("NA"))]).unwrap();

    assert_eq!(
        import,
        table_of_column(3, booleans(&[Some(true), Some(false), None]))
    );
}

#[test]
fn boolean_cells_and_a_text_false_are_a_boolean_column() {
    let import = import_column(&[Some(Boolean(true)), Some(Text("False")), None]).unwrap();

    assert_eq!(
        import,
        table_of_column(3, booleans(&[Some(true), Some(false), None]))
    );
}

#[test]
fn texts_with_the_comma_are_a_text_column_since_an_xlsx_is_read_with_the_point() {
    let import = import_column(&[Some(Text("1,5")), Some(Text("2"))]).unwrap();

    assert_eq!(import, table_of_column(2, texts(&[Some("1,5"), Some("2")])));
}

#[test]
fn a_text_past_the_integers_makes_a_float_column_of_two_to_the_63() {
    let import = import_column(&[
        Some(Text("9223372036854775807")),
        Some(Text("9223372036854775808")),
    ])
    .unwrap();

    assert_eq!(
        import,
        table_of_column(2, floats(&[Some(TWO_TO_THE_63), Some(TWO_TO_THE_63)]))
    );
}

#[test]
fn number_cells_1_and_2_point_5_are_a_float_column() {
    let import = import_column(&[Some(Number(1.0)), Some(Number(2.5))]).unwrap();

    assert_eq!(import, table_of_column(2, floats(&[Some(1.0), Some(2.5)])));
}

#[test]
fn a_number_cell_of_two_to_the_63_is_a_float_column() {
    let import = import_column(&[Some(Number(TWO_TO_THE_63)), Some(Number(1.0))]).unwrap();

    assert_eq!(
        import,
        table_of_column(2, floats(&[Some(TWO_TO_THE_63), Some(1.0)]))
    );
}

#[test]
fn a_number_cell_of_minus_two_to_the_63_is_the_least_integer() {
    let import = import_column(&[Some(Number(-TWO_TO_THE_63)), Some(Number(-0.0))]).unwrap();

    assert_eq!(
        import,
        table_of_column(2, integers(&[Some(i64::MIN), Some(0)]))
    );
}

#[test]
fn a_number_cell_1_and_a_text_2_are_an_integer_column() {
    let import = import_column(&[Some(Number(1.0)), Some(Text("2"))]).unwrap();

    assert_eq!(import, table_of_column(2, integers(&[Some(1), Some(2)])));
}

#[test]
fn a_number_cell_1_point_5_and_a_text_2_comma_5_are_a_text_column() {
    let import = import_column(&[Some(Number(1.5)), Some(Text("2,5"))]).unwrap();

    assert_eq!(
        import,
        table_of_column(2, texts(&[Some("1.5"), Some("2,5")]))
    );
}

#[test]
fn a_boolean_cell_and_a_number_cell_are_a_text_column_of_true_and_1() {
    let import = import_column(&[Some(Boolean(true)), Some(Number(1.0))]).unwrap();

    assert_eq!(
        import,
        table_of_column(2, texts(&[Some("TRUE"), Some("1")]))
    );
}

#[test]
fn number_cells_in_a_text_column_are_written_as_javascript_writes_them() {
    let import = import_column(&[
        Some(Number(1e21)),
        Some(Number(1.5e-7)),
        Some(Number(0.1)),
        Some(Text("x")),
    ])
    .unwrap();

    assert_eq!(
        import,
        table_of_column(
            4,
            texts(&[Some("1e+21"), Some("1.5e-7"), Some("0.1"), Some("x")])
        )
    );
}

#[test]
fn a_column_of_missing_cells_alone_is_a_text_column_kept_with_its_name() {
    let import =
        import_column(&[Some(Text("NA")), Some(Error("#N/A")), None, Some(Text("-"))]).unwrap();

    assert_eq!(import, table_of_column(4, texts(&[None, None, None, None])));
}

#[test]
fn a_column_of_heights_with_one_text_n_d_is_a_text_column() {
    let import =
        import_column(&[Some(Number(1.75)), Some(Text("n.d.")), Some(Number(1.62))]).unwrap();

    assert_eq!(
        import,
        table_of_column(3, texts(&[Some("1.75"), Some("n.d."), Some("1.62")]))
    );
}

#[test]
fn a_column_of_dates_is_text_and_a_year_typed_as_a_number_an_integer() {
    let import = import(&[
        ("A1", Text("id")),
        ("B1", Text("date")),
        ("C1", Text("year")),
        ("A2", Text("A")),
        ("B2", Date(2024, 5, 13)),
        ("C2", Number(2024.0)),
    ])
    .unwrap();

    assert_eq!(
        import,
        Ok(table(
            "id",
            1,
            &["A"],
            vec![
                column("date", 2, texts(&[Some("2024-05-13")])),
                column("year", 3, integers(&[Some(2024)])),
            ],
        ))
    );
}

#[test]
fn a_file_of_the_names_alone_is_a_table_with_no_other_column() {
    let import = import(&[("A1", Text("only")), ("A2", Text("A")), ("A3", Text("B"))]).unwrap();

    assert_eq!(import, Ok(table("only", 1, &["A", "B"], Vec::new())));
}

// The property of "The type of a column": the guess depends on the values
// and not on their order.

/// A generator of numbers for the tests, xorshift64*, with a fixed seed
/// so that every run makes the same tables.
struct Generator {
    /// The state, never 0.
    state: u64,
}

impl Generator {
    /// The next number, from 0 to `bound` − 1, or None when `bound` is 0.
    fn below(&mut self, bound: usize) -> Option<usize> {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        let number = self.state.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 33;
        usize::try_from(number).ok()?.checked_rem(bound)
    }

    /// One of `choices`, or None when there is none.
    fn pick<T: Copy>(&mut self, choices: &[T]) -> Option<T> {
        let index = self.below(choices.len())?;
        choices.get(index).copied()
    }
}

/// The values a generated column draws from, each set making a type more
/// likely, None for a cell left empty.
const POOLS: [&[Option<Written<'static>>]; 4] = [
    &[
        Some(Number(1.0)),
        Some(Number(-3.0)),
        Some(Text("2")),
        Some(Text("+7")),
        Some(Number(0.0)),
    ],
    &[
        Some(Number(1.5)),
        Some(Text("2.5")),
        Some(Text("1")),
        Some(Number(2.0)),
        Some(Text("1e3")),
    ],
    &[
        Some(Boolean(true)),
        Some(Boolean(false)),
        Some(Text("TRUE")),
        Some(Text("false")),
    ],
    &[
        Some(Text("x")),
        Some(Text("1,5")),
        Some(Number(4.25)),
        Some(Boolean(true)),
        Some(Text("2")),
    ],
];

/// The missing cells a generated column draws from.
const MISSING: [Option<Written<'static>>; 5] = [
    None,
    Some(Text("NA")),
    Some(Text("-")),
    Some(Text("  ")),
    Some(Error("#N/A")),
];

/// The names of the individuals of the generated tables.
const INDIVIDUALS: [&str; 12] = [
    "i1", "i2", "i3", "i4", "i5", "i6", "i7", "i8", "i9", "i10", "i11", "i12",
];

/// The names of the columns of the generated tables.
const COLUMN_NAMES: [&str; 4] = ["c1", "c2", "c3", "c4"];

/// A row of a generated table: the name of its individual and its values.
type GeneratedRow = (&'static str, Vec<Option<Written<'static>>>);

/// What `import_table` gives for an xlsx of a header `id` and the first
/// `num_columns` of [`COLUMN_NAMES`] over `rows`, from A1, or the error of
/// the test when the xlsx is not written.
fn import_rows(
    num_columns: usize,
    rows: &[GeneratedRow],
) -> Result<Result<Table, ImportError>, Box<dyn std::error::Error>> {
    let mut cells = vec![(0, 0, Text("id"))];
    for (column, name) in (1_u16..).zip(COLUMN_NAMES.iter().take(num_columns)) {
        cells.push((0, column, Text(name)));
    }
    for (row, (name, values)) in (1_u32..).zip(rows) {
        cells.push((row, 0, Text(name)));
        for (column, value) in (1_u16..).zip(values) {
            if let Some(value) = value {
                cells.push((row, column, *value));
            }
        }
    }
    Ok(import_table(&xlsx_at(&cells)?, &options()))
}

/// The types of the columns of a table, and the values of each column as
/// text, each with the name of its individual, sorted by it.
type TypesAndValues = (Vec<ColumnType>, Vec<Vec<(String, Option<String>)>>);

/// The [`TypesAndValues`] of `table`, or None when a column does not
/// convert to text.
fn types_and_values(table: &Table) -> Option<TypesAndValues> {
    let types = table
        .columns
        .iter()
        .map(|column| column.values.column_type())
        .collect();
    let mut values = Vec::new();
    for column in &table.columns {
        let ColumnValues::Text(column_texts) =
            convert_column(&column.values, ColumnType::Text, DecimalMark::Point).ok()?
        else {
            return None;
        };
        let mut by_name: Vec<(String, Option<String>)> = table
            .names
            .names
            .iter()
            .cloned()
            .zip(column_texts)
            .collect();
        by_name.sort();
        values.push(by_name);
    }
    Some((types, values))
}

#[test]
fn a_table_and_the_same_table_with_its_rows_shuffled_give_the_same_types() {
    let mut generator = Generator {
        state: 0x9E37_79B9_7F4A_7C15,
    };
    let mut types_seen = std::collections::HashSet::new();
    for _ in 0..300 {
        let num_columns = 1 + generator.below(COLUMN_NAMES.len()).unwrap();
        let num_rows = 1 + generator.below(INDIVIDUALS.len()).unwrap();
        let pools: Vec<&[Option<Written<'static>>]> = (0..num_columns)
            .map(|_| generator.pick(&POOLS).unwrap())
            .collect();
        let mut rows: Vec<GeneratedRow> = INDIVIDUALS
            .iter()
            .take(num_rows)
            .map(|&name| {
                let values = pools
                    .iter()
                    .map(|pool| match generator.below(10).unwrap() {
                        0..=1 => generator.pick(&MISSING).unwrap(),
                        2 => {
                            let other_pool = generator.pick(&POOLS).unwrap();
                            generator.pick(other_pool).unwrap()
                        }
                        _ => generator.pick(pool).unwrap(),
                    })
                    .collect();
                (name, values)
            })
            .collect();
        let table = import_rows(num_columns, &rows).unwrap().unwrap();
        for index in (1..rows.len()).rev() {
            rows.swap(index, generator.below(index + 1).unwrap());
        }
        let shuffled = import_rows(num_columns, &rows).unwrap().unwrap();

        let (types, values) = types_and_values(&table).unwrap();
        assert_eq!(
            types_and_values(&shuffled),
            Some((types.clone(), values)),
            "the table {table:?}"
        );
        types_seen.extend(types);
    }
    assert_eq!(
        types_seen.len(),
        4,
        "the types of the tables made: {types_seen:?}"
    );
}

// The owner's files of "Made by the owner" of docs/specs/read.md, as the
// tables they show.

/// What `import_table` gives for `file_name` in `tests/data/` at the root
/// of the repository.
fn import_owner_file(file_name: &str) -> std::io::Result<Result<Table, ImportError>> {
    let bytes = std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/data")
            .join(file_name),
    )?;
    Ok(import_table(&bytes, &options()))
}

/// The table of `excel_es.xlsx`, sheet `Hoja1`, from A1: the blank row 5
/// skipped, the population merged over B3:B4 in both rows, `'001` the name
/// `001`, and the errors of `=NOD()` and `=1/0` in `Código` missing.
fn table_of_the_first_file() -> Table {
    Table {
        names: NameColumn {
            header: "Individuo".to_owned(),
            number: 1,
            names: ["ind1", "ind2", "001", "ind4", "ind5"]
                .map(str::to_owned)
                .to_vec(),
        },
        columns: vec![
            column(
                "Población",
                2,
                texts(&[
                    Some("Andalucía"),
                    Some("Castilla y León"),
                    Some("Castilla y León"),
                    Some("Murcia"),
                    Some("Murcia"),
                ]),
            ),
            column(
                "Altura",
                3,
                floats(&[Some(1.75), Some(1.62), Some(1.8), Some(1.55), Some(1.7)]),
            ),
            column(
                "Fecha",
                4,
                texts(&[
                    Some("2024-05-13"),
                    Some("2024-05-14"),
                    Some("2024-05-15"),
                    Some("2024-05-16"),
                    Some("2024-05-17"),
                ]),
            ),
            column(
                "Hora",
                5,
                texts(&[
                    Some("14:30:00"),
                    Some("09:05:00"),
                    Some("18:45:00"),
                    Some("07:00:00"),
                    Some("12:15:00"),
                ]),
            ),
            column(
                "Afectado",
                6,
                booleans(&[Some(true), Some(false), Some(true), Some(false), Some(true)]),
            ),
            column(
                "Código",
                7,
                integers(&[Some(7), Some(12), Some(3), None, None]),
            ),
        ],
        read: HowRead::Xlsx {
            sheet: "Hoja1".to_owned(),
        },
    }
}

#[test]
fn excel_es_xlsx_is_the_table_of_spanish_excel() {
    let import = import_owner_file("excel_es.xlsx").unwrap();

    assert_eq!(import, Ok(table_of_the_first_file()));
}

#[test]
fn excel_en_xlsx_is_the_table_of_english_excel() {
    let import = import_owner_file("excel_en.xlsx").unwrap();

    assert_eq!(import, Ok(table_of_the_first_file()));
}

#[test]
#[ignore = "waits for tests/data/libreoffice.xlsx, made by the owner"]
fn libreoffice_xlsx_is_the_table_of_libreoffice_calc() {
    let table = import_owner_file("libreoffice.xlsx").unwrap().unwrap();

    // The name of the sheet LibreOffice gives is not known until the file
    // is made; the rest is the table of the first file.
    assert_eq!(
        (table.names, table.columns),
        (
            table_of_the_first_file().names,
            table_of_the_first_file().columns
        )
    );
}

#[test]
fn google_sheets_xlsx_is_the_table_of_the_pasted_values() {
    // The paste made `001` the number 1, lost the merge of B3:B4, and
    // left G7 the text `#¡DIV/0!`, which is not an error of Excel, so
    // that `Código` is text.
    let first = table_of_the_first_file();
    let mut columns = first.columns;
    if let Some(population) = columns.first_mut() {
        population.values = texts(&[
            Some("Andalucía"),
            Some("Castilla y León"),
            None,
            Some("Murcia"),
            Some("Murcia"),
        ]);
    }
    if let Some(code) = columns.last_mut() {
        code.values = texts(&[Some("7"), Some("12"), Some("3"), None, Some("#¡DIV/0!")]);
    }
    let expected = Table {
        names: NameColumn {
            header: first.names.header,
            number: first.names.number,
            names: ["ind1", "ind2", "1", "ind4", "ind5"]
                .map(str::to_owned)
                .to_vec(),
        },
        columns,
        read: HowRead::Xlsx {
            sheet: "Sheet1".to_owned(),
        },
    };

    let import = import_owner_file("google_sheets.xlsx").unwrap();

    assert_eq!(import, Ok(expected));
}

#[test]
fn excel_1904_xlsx_is_a_table_of_one_date_named_by_itself() {
    let import = import_owner_file("excel_1904.xlsx").unwrap();

    assert_eq!(
        import,
        Ok(Table {
            names: NameColumn {
                header: "Fecha".to_owned(),
                number: 1,
                names: vec!["2024-05-13".to_owned()],
            },
            columns: vec![column("Hora", 2, texts(&[Some("14:30:00")]))],
            read: HowRead::Xlsx {
                sheet: "Sheet1".to_owned(),
            },
        })
    );
}

#[test]
fn spill_xlsx_is_a_header_error_of_value_at_a1() {
    // A1 `=SEQUENCE(3)` with a value in A2, which Excel 365 saves as
    // `#VALUE!`: the header of the table is the error.
    let import = import_owner_file("spill.xlsx").unwrap();

    assert_eq!(
        import,
        refused(Refusal::HeaderError {
            row: 1,
            column: 1,
            error: "#VALUE!".to_owned(),
        })
    );
}
