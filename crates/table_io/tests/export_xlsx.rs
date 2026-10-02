//! The xlsx `export_table` writes of a table, by "An xlsx", "The refusals"
//! and "How it is verified" of `docs/specs/export.md`: each case a literal
//! table, the file it gives read back by `import_table` and by calamine as
//! the cells of its sheet, or the literal refusal; and the property of
//! "What reads back", over the tables the round trip of a CSV is tried
//! on, each exported as an xlsx.
//!
//! The tests run in every build with the feature `xlsx`; the refusal of
//! an xlsx by a build without it is in `tests/export_csv.rs`.

#![cfg(feature = "xlsx")]

#[cfg(test)]
mod written {
    use std::io::Cursor;

    use calamine::{Data, Range, Reader, Xlsx};
    use table_io::{
        CellPlace, Column, ColumnValues, ExportError, ExportFormat, ExportRefusal, HowRead,
        ImportError, ImportOptions, NameColumn, Table, TextOptions, export_table, import_table,
    };

    /// 2^53, the largest whole number from which a float holds every
    /// whole number below exactly.
    const TWO_TO_THE_53: i64 = 9_007_199_254_740_992;

    /// The seven errors of Excel, as the import of an xlsx reads them.
    const EXCEL_ERRORS: [&str; 7] = [
        "#N/A", "#DIV/0!", "#NAME?", "#NULL!", "#NUM!", "#REF!", "#VALUE!",
    ];

    /// The names' column of `header` and `names`; its number, which the
    /// export does not use, is 9.
    fn names_of(header: &str, names: &[&str]) -> NameColumn {
        NameColumn {
            header: header.to_owned(),
            number: 9,
            names: names.iter().map(|&name| name.to_owned()).collect(),
        }
    }

    /// A column named `name` of `values`; its number, which the export
    /// does not use, is 7.
    fn column(name: &str, values: ColumnValues) -> Column {
        Column {
            name: name.to_owned(),
            number: 7,
            values,
        }
    }

    /// The values of a text column, None for a missing one.
    fn texts(values: &[Option<&str>]) -> ColumnValues {
        ColumnValues::Text(values.iter().map(|text| text.map(str::to_owned)).collect())
    }

    /// The refusal `refusal`, as `export_table` gives it.
    fn refused(refusal: ExportRefusal) -> Result<Vec<u8>, ExportError> {
        Err(ExportError::Refused(refusal))
    }

    /// The place at `column` and `row`.
    fn at(column: u32, row: u32) -> CellPlace {
        CellPlace { column, row }
    }

    /// The xlsx of the table, which the test expects to be written.
    fn xlsx_of(names: &NameColumn, columns: &[Column]) -> Vec<u8> {
        export_table(names, columns, &ExportFormat::Xlsx).unwrap()
    }

    /// The table `bytes` read back within popnei_web's limits, the
    /// options of a text file unset.
    fn read_back(bytes: &[u8]) -> Result<Table, ImportError> {
        import_table(
            bytes,
            &ImportOptions {
                max_bytes: 20_000_000,
                max_cells: 2_000_000,
                text: TextOptions {
                    encoding: None,
                    separator: None,
                    decimal: None,
                },
            },
        )
    }

    /// The names of the sheets of `bytes` and the cells of its first, as
    /// calamine reads them, from A1.
    fn calamine_sheet(bytes: &[u8]) -> (Vec<String>, Range<Data>) {
        let mut workbook: Xlsx<_> = Xlsx::new(Cursor::new(bytes.to_vec())).unwrap();
        let sheet_names = workbook.sheet_names();
        let range = workbook.worksheet_range_at(0).unwrap().unwrap();
        (sheet_names, range)
    }

    /// The cell of calamine's `range` at `row` and `column`, from 0 as
    /// calamine counts them, Empty where the sheet has none.
    fn calamine_cell(range: &Range<Data>, row: u32, column: u32) -> Data {
        range
            .get_value((row, column))
            .cloned()
            .unwrap_or(Data::Empty)
    }

    /// The table of "How it is verified": `id`, `h`, `n`, `ok`, `pop` over
    /// two rows, `A`, 1.5, 3, true, `P1` and `B`, missing, −2, false,
    /// `x;y`, with a column `big` of 2^53 and −2^53.
    fn spec_table() -> (NameColumn, Vec<Column>) {
        (
            names_of("id", &["A", "B"]),
            vec![
                column("h", ColumnValues::Float(vec![Some(1.5), None])),
                column("n", ColumnValues::Integer(vec![Some(3), Some(-2)])),
                column("ok", ColumnValues::Boolean(vec![Some(true), Some(false)])),
                column("pop", texts(&[Some("P1"), Some("x;y")])),
                column(
                    "big",
                    ColumnValues::Integer(vec![Some(TWO_TO_THE_53), Some(-TWO_TO_THE_53)]),
                ),
            ],
        )
    }

    #[test]
    fn the_table_of_the_spec_is_one_sheet_sheet1_of_the_cells_of_its_values() {
        let (names, columns) = spec_table();
        let bytes = xlsx_of(&names, &columns);

        let (sheet_names, range) = calamine_sheet(&bytes);

        assert_eq!(sheet_names, ["Sheet1"]);
        assert_eq!(range.start(), Some((0, 0)));
        assert_eq!(range.end(), Some((2, 5)));
        let header: Vec<Data> = (0..6)
            .map(|column| calamine_cell(&range, 0, column))
            .collect();
        assert_eq!(
            header,
            ["id", "h", "n", "ok", "pop", "big"].map(|name| Data::String(name.to_owned()))
        );
        let first_row: Vec<Data> = (0..6)
            .map(|column| calamine_cell(&range, 1, column))
            .collect();
        assert_eq!(
            first_row,
            [
                Data::String("A".to_owned()),
                Data::Float(1.5),
                Data::Float(3.0),
                Data::Bool(true),
                Data::String("P1".to_owned()),
                Data::Float(9_007_199_254_740_992.0),
            ]
        );
        let second_row: Vec<Data> = (0..6)
            .map(|column| calamine_cell(&range, 2, column))
            .collect();
        assert_eq!(
            second_row,
            [
                Data::String("B".to_owned()),
                Data::Empty,
                Data::Float(-2.0),
                Data::Bool(false),
                Data::String("x;y".to_owned()),
                Data::Float(-9_007_199_254_740_992.0),
            ]
        );
    }

    #[test]
    fn the_table_of_the_spec_reads_back_as_itself() {
        let (names, columns) = spec_table();

        let table = read_back(&xlsx_of(&names, &columns)).unwrap();

        assert_eq!(
            table.read,
            HowRead::Xlsx {
                sheet: "Sheet1".to_owned()
            }
        );
        assert_eq!(table.names.header, "id");
        assert_eq!(table.names.names, ["A", "B"]);
        let read_columns: Vec<(&str, &ColumnValues)> = table
            .columns
            .iter()
            .map(|read_column| (read_column.name.as_str(), &read_column.values))
            .collect();
        assert_eq!(
            read_columns,
            [
                ("h", &ColumnValues::Float(vec![Some(1.5), None])),
                ("n", &ColumnValues::Integer(vec![Some(3), Some(-2)])),
                ("ok", &ColumnValues::Boolean(vec![Some(true), Some(false)])),
                ("pop", &texts(&[Some("P1"), Some("x;y")])),
                (
                    "big",
                    &ColumnValues::Integer(vec![
                        Some(9_007_199_254_740_992),
                        Some(-9_007_199_254_740_992)
                    ])
                ),
            ]
        );
    }

    #[test]
    fn a_float_column_of_whole_values_reads_back_as_integers_and_one_past_an_integer_as_floats() {
        let names = names_of("id", &["a", "b"]);
        let columns = [
            column("w", ColumnValues::Float(vec![Some(1.0), Some(-3.0)])),
            column("z", ColumnValues::Float(vec![Some(-0.0), None])),
            column("e", ColumnValues::Float(vec![Some(1e20), Some(2.0)])),
        ];
        let bytes = xlsx_of(&names, &columns);

        let (_, range) = calamine_sheet(&bytes);
        let table = read_back(&bytes).unwrap();

        assert_eq!(calamine_cell(&range, 1, 1), Data::Float(1.0));
        assert_eq!(calamine_cell(&range, 1, 3), Data::Float(1e20));
        assert_eq!(
            table.columns[0].values,
            ColumnValues::Integer(vec![Some(1), Some(-3)])
        );
        assert_eq!(
            table.columns[1].values,
            ColumnValues::Integer(vec![Some(0), None])
        );
        assert_eq!(
            table.columns[2].values,
            ColumnValues::Float(vec![Some(1e20), Some(2.0)])
        );
    }

    #[test]
    fn the_smallest_and_the_largest_floats_and_one_of_many_digits_read_back_as_themselves() {
        let names = names_of("id", &["a", "b", "c", "d"]);
        let floats = [5e-324, f64::MAX, 0.1 + 0.2, -1.5e-7];
        let columns = [column(
            "x",
            ColumnValues::Float(floats.iter().copied().map(Some).collect()),
        )];
        let bytes = xlsx_of(&names, &columns);

        let (_, range) = calamine_sheet(&bytes);
        let table = read_back(&bytes).unwrap();

        assert_eq!(calamine_cell(&range, 1, 1), Data::Float(5e-324));
        assert_eq!(calamine_cell(&range, 2, 1), Data::Float(f64::MAX));
        assert_eq!(
            calamine_cell(&range, 3, 1),
            Data::Float(0.300_000_000_000_000_04)
        );
        assert_eq!(calamine_cell(&range, 4, 1), Data::Float(-1.5e-7));
        assert_eq!(
            table.columns[0].values,
            ColumnValues::Float(vec![
                Some(5e-324),
                Some(f64::MAX),
                Some(0.300_000_000_000_000_04),
                Some(-1.5e-7)
            ])
        );
    }

    #[test]
    fn an_empty_name_of_the_names_column_is_no_cell_and_reads_back_empty() {
        let names = names_of("", &["a"]);
        let columns = [column("x", ColumnValues::Integer(vec![Some(4)]))];
        let bytes = xlsx_of(&names, &columns);

        let (_, range) = calamine_sheet(&bytes);
        let table = read_back(&bytes).unwrap();

        assert_eq!(calamine_cell(&range, 0, 0), Data::Empty);
        assert_eq!(calamine_cell(&range, 0, 1), Data::String("x".to_owned()));
        assert_eq!(table.names.header, "");
        assert_eq!(table.names.names, ["a"]);
    }

    #[test]
    fn a_table_of_the_names_alone_is_written() {
        let names = names_of("id", &["a", "b"]);
        let bytes = xlsx_of(&names, &[]);

        let table = read_back(&bytes).unwrap();

        assert_eq!(table.names.header, "id");
        assert_eq!(table.names.names, ["a", "b"]);
        assert!(table.columns.is_empty());
    }

    #[test]
    fn every_character_but_u_fffe_and_u_ffff_reads_back_as_itself() {
        let kept = [
            "Población",
            "𝔸😀",
            "a\nb",
            "a\rb",
            "a\r\nb",
            "x\0y",
            "x\u{1}y\u{1F}",
            "\u{FEFF}a",
            "\u{FFFD}",
            "_x0041_",
            "_x005F_x0041_",
            "a\tb",
            "say \"hi\" & <b>",
        ];
        let row_names: Vec<String> = (1..=kept.len()).map(|row| format!("r{row}")).collect();
        let row_name_texts: Vec<&str> = row_names.iter().map(String::as_str).collect();
        let names = names_of("Población", &row_name_texts);
        let columns = [column(
            "año 😀",
            texts(&kept.iter().copied().map(Some).collect::<Vec<_>>()),
        )];
        let bytes = xlsx_of(&names, &columns);

        let (_, range) = calamine_sheet(&bytes);
        let table = read_back(&bytes).unwrap();

        assert_eq!(
            calamine_cell(&range, 0, 0),
            Data::String("Población".to_owned())
        );
        assert_eq!(calamine_cell(&range, 6, 1), Data::String("x\0y".to_owned()));
        assert_eq!(
            calamine_cell(&range, 10, 1),
            Data::String("_x0041_".to_owned())
        );
        assert_eq!(table.names.header, "Población");
        assert_eq!(table.columns[0].name, "año 😀");
        assert_eq!(
            table.columns[0].values,
            texts(&kept.iter().copied().map(Some).collect::<Vec<_>>())
        );
    }

    #[test]
    fn an_individual_named_as_an_error_of_excel_or_as_missing_reads_back_as_that_name() {
        let names = names_of("id", &["#N/A", "NA", "-", "#VALUE!"]);
        let columns = [column(
            "x",
            ColumnValues::Boolean(vec![Some(true), None, Some(false), None]),
        )];

        let table = read_back(&xlsx_of(&names, &columns)).unwrap();

        assert_eq!(table.names.names, ["#N/A", "NA", "-", "#VALUE!"]);
        assert_eq!(
            table.columns[0].values,
            ColumnValues::Boolean(vec![Some(true), None, Some(false), None])
        );
    }

    #[test]
    fn a_text_column_of_001_and_002_reads_back_as_the_integers_1_and_2() {
        let names = names_of("id", &["a", "b", "c"]);
        let columns = [
            column("code", texts(&[Some("001"), Some("002"), None])),
            column("f", texts(&[Some("1.5"), Some("1e3"), Some("2")])),
            column("c", texts(&[Some("1,5"), Some("2"), Some("3")])),
            column("b", texts(&[Some("TRUE"), Some("false"), None])),
        ];

        let table = read_back(&xlsx_of(&names, &columns)).unwrap();

        assert_eq!(
            table.columns[0].values,
            ColumnValues::Integer(vec![Some(1), Some(2), None])
        );
        assert_eq!(
            table.columns[1].values,
            ColumnValues::Float(vec![Some(1.5), Some(1000.0), Some(2.0)])
        );
        assert_eq!(
            table.columns[2].values,
            texts(&[Some("1,5"), Some("2"), Some("3")])
        );
        assert_eq!(
            table.columns[3].values,
            ColumnValues::Boolean(vec![Some(true), Some(false), None])
        );
    }

    #[test]
    fn an_integer_of_2_to_the_53_either_sign_is_written_exact_and_one_past_it_is_refused() {
        let names = names_of("id", &["a", "b", "c"]);
        let written = [column(
            "n",
            ColumnValues::Integer(vec![Some(TWO_TO_THE_53), Some(-TWO_TO_THE_53), None]),
        )];
        let past_above = [
            column("m", ColumnValues::Integer(vec![None, None, Some(0)])),
            column(
                "n",
                ColumnValues::Integer(vec![Some(0), Some(9_007_199_254_740_993), None]),
            ),
        ];
        let past_below = [column(
            "n",
            ColumnValues::Integer(vec![None, None, Some(-9_007_199_254_740_993)]),
        )];

        let table = read_back(&xlsx_of(&names, &written)).unwrap();

        assert_eq!(
            table.columns[0].values,
            ColumnValues::Integer(vec![
                Some(9_007_199_254_740_992),
                Some(-9_007_199_254_740_992),
                None
            ])
        );
        assert_eq!(
            export_table(&names, &past_above, &ExportFormat::Xlsx),
            refused(ExportRefusal::IntegerTooLarge { place: at(3, 2) })
        );
        assert_eq!(
            export_table(&names, &past_below, &ExportFormat::Xlsx),
            refused(ExportRefusal::IntegerTooLarge { place: at(2, 3) })
        );
    }

    #[test]
    fn the_integer_2_to_the_60_is_refused_with_its_place() {
        let names = names_of("id", &["a", "b"]);
        let columns = [
            column("x", texts(&[Some("p"), Some("q")])),
            column(
                "n",
                ColumnValues::Integer(vec![None, Some(1_152_921_504_606_846_976)]),
            ),
        ];

        assert_eq!(
            export_table(&names, &columns, &ExportFormat::Xlsx),
            refused(ExportRefusal::IntegerTooLarge { place: at(3, 2) })
        );
    }

    #[test]
    fn the_largest_and_the_smallest_integers_are_refused() {
        let names = names_of("id", &["a"]);
        for integer in [i64::MAX, i64::MIN] {
            let columns = [column("n", ColumnValues::Integer(vec![Some(integer)]))];

            assert_eq!(
                export_table(&names, &columns, &ExportFormat::Xlsx),
                refused(ExportRefusal::IntegerTooLarge { place: at(2, 1) }),
                "{integer}"
            );
        }
    }

    #[test]
    fn a_name_with_a_space_at_its_end_is_refused_with_its_place() {
        let names = names_of("id", &["a"]);
        let columns = [
            column("x", texts(&[Some("p")])),
            column("height ", ColumnValues::Float(vec![Some(1.75)])),
        ];

        assert_eq!(
            export_table(&names, &columns, &ExportFormat::Xlsx),
            refused(ExportRefusal::SpacesAtEnds { place: at(3, 0) })
        );
    }

    #[test]
    fn a_space_or_a_tab_at_either_end_of_a_name_or_a_value_is_refused_and_one_inside_is_not() {
        let names = names_of("id", &["a", "b"]);
        let cases: [(NameColumn, Vec<Column>, CellPlace); 5] = [
            (
                names_of(" id", &["a", "b"]),
                vec![column("x", texts(&[Some("p"), Some("q")]))],
                at(1, 0),
            ),
            (
                names_of("id", &["a", "b\t"]),
                vec![column("x", texts(&[Some("p"), Some("q")]))],
                at(1, 2),
            ),
            (
                names.clone(),
                vec![
                    column("x", texts(&[Some("p"), Some("q")])),
                    column("y", texts(&[Some("r"), Some("\ts")])),
                ],
                at(3, 2),
            ),
            (
                names.clone(),
                vec![column("x", texts(&[Some("p "), Some("q")]))],
                at(2, 1),
            ),
            (
                names.clone(),
                vec![column("x", texts(&[Some("p"), Some("   ")]))],
                at(2, 2),
            ),
        ];
        for (case_names, case_columns, place) in cases {
            assert_eq!(
                export_table(&case_names, &case_columns, &ExportFormat::Xlsx),
                refused(ExportRefusal::SpacesAtEnds { place }),
                "{case_names:?} {case_columns:?}"
            );
        }
        let inside = [column("x y", texts(&[Some("p q"), Some("r\ts")]))];
        let table = read_back(&xlsx_of(&names, &inside)).unwrap();
        assert_eq!(table.columns[0].name, "x y");
        assert_eq!(table.columns[0].values, texts(&[Some("p q"), Some("r\ts")]));
    }

    #[test]
    fn a_name_of_the_header_that_is_an_error_of_excel_is_refused_with_its_place() {
        let names = names_of("id", &["a"]);
        for error in EXCEL_ERRORS {
            let columns = [
                column("x", texts(&[Some("p")])),
                column(error, texts(&[Some("q")])),
            ];

            assert_eq!(
                export_table(&names, &columns, &ExportFormat::Xlsx),
                refused(ExportRefusal::ErrorAsName { place: at(3, 0) }),
                "{error}"
            );
        }
        let columns = [column("x", texts(&[Some("p")]))];
        assert_eq!(
            export_table(&names_of("#N/A", &["a"]), &columns, &ExportFormat::Xlsx),
            refused(ExportRefusal::ErrorAsName { place: at(1, 0) })
        );
    }

    #[test]
    fn a_text_value_that_is_an_error_of_excel_is_refused_as_reading_back_missing() {
        let names = names_of("id", &["a", "b"]);
        for error in EXCEL_ERRORS {
            let columns = [column("x", texts(&[Some("p"), Some(error)]))];

            assert_eq!(
                export_table(&names, &columns, &ExportFormat::Xlsx),
                refused(ExportRefusal::ReadsAsMissing { place: at(2, 2) }),
                "{error}"
            );
        }
        let columns = [column("x", texts(&[Some("NA"), Some("p")]))];
        assert_eq!(
            export_table(&names, &columns, &ExportFormat::Xlsx),
            refused(ExportRefusal::ReadsAsMissing { place: at(2, 1) })
        );
    }

    #[test]
    fn a_text_of_32767_units_of_utf16_is_written_and_one_of_32768_refused_with_its_length() {
        let names = names_of("id", &["a", "b"]);
        let longest = "a".repeat(32_767);
        let longest_emoji = format!("{}a", "😀".repeat(16_383));
        let written = [column("x", texts(&[Some(&longest), Some(&longest_emoji)]))];
        let too_long = "a".repeat(32_768);
        let too_long_emoji = "😀".repeat(16_384);

        let table = read_back(&xlsx_of(&names, &written)).unwrap();

        assert_eq!(
            table.columns[0].values,
            texts(&[Some(&longest), Some(&longest_emoji)])
        );
        assert_eq!(
            export_table(
                &names,
                &[column("x", texts(&[Some("p"), Some(&too_long)]))],
                &ExportFormat::Xlsx
            ),
            refused(ExportRefusal::TextTooLong {
                place: at(2, 2),
                length: 32_768
            })
        );
        assert_eq!(
            export_table(
                &names_of("id", &["a", &too_long_emoji]),
                &[],
                &ExportFormat::Xlsx
            ),
            refused(ExportRefusal::TextTooLong {
                place: at(1, 2),
                length: 32_768
            })
        );
        let long_name = format!("{too_long}b");
        assert_eq!(
            export_table(
                &names,
                &[column(&long_name, texts(&[Some("p"), Some("q")]))],
                &ExportFormat::Xlsx
            ),
            refused(ExportRefusal::TextTooLong {
                place: at(2, 0),
                length: 32_769
            })
        );
    }

    #[test]
    fn a_text_or_a_name_holding_u_fffe_or_u_ffff_is_refused_with_its_place_and_character() {
        let names = names_of("id", &["a", "b"]);

        assert_eq!(
            export_table(
                &names,
                &[column("x", texts(&[None, Some("p\u{FFFE}q\u{FFFF}")]))],
                &ExportFormat::Xlsx
            ),
            refused(ExportRefusal::CannotCarry {
                place: at(2, 2),
                character: '\u{FFFE}'
            })
        );
        assert_eq!(
            export_table(
                &names,
                &[
                    column("x", texts(&[None, None])),
                    column("y\u{FFFF}", texts(&[None, None]))
                ],
                &ExportFormat::Xlsx
            ),
            refused(ExportRefusal::CannotCarry {
                place: at(3, 0),
                character: '\u{FFFF}'
            })
        );
        assert_eq!(
            export_table(
                &names_of("id", &["a", "\u{FFFF}b"]),
                &[],
                &ExportFormat::Xlsx
            ),
            refused(ExportRefusal::CannotCarry {
                place: at(1, 2),
                character: '\u{FFFF}'
            })
        );
    }

    #[test]
    fn a_float_that_is_not_finite_is_refused_with_its_place() {
        let names = names_of("id", &["a", "b"]);
        let columns = [column(
            "x",
            ColumnValues::Float(vec![Some(1.0), Some(f64::NAN)]),
        )];

        assert_eq!(
            export_table(&names, &columns, &ExportFormat::Xlsx),
            refused(ExportRefusal::NotFinite { place: at(2, 2) })
        );
    }

    #[test]
    fn a_table_of_16384_columns_is_written_and_one_of_16385_refused_with_its_counts() {
        let names = names_of("id", &["a"]);
        let columns: Vec<Column> = (2..=16_384)
            .map(|number| column(&format!("c{number}"), ColumnValues::Integer(vec![Some(1)])))
            .collect();
        let mut too_many = columns.clone();
        too_many.push(column("c16385", ColumnValues::Integer(vec![Some(1)])));

        let table = read_back(&xlsx_of(&names, &columns)).unwrap();

        assert_eq!(table.columns.len(), 16_383);
        assert_eq!(table.columns[16_382].name, "c16384");
        assert_eq!(
            export_table(&names, &too_many, &ExportFormat::Xlsx),
            refused(ExportRefusal::TooLargeForSheet {
                rows: 1,
                columns: 16_385
            })
        );
    }

    #[test]
    fn a_table_of_1048575_individuals_is_written_and_one_of_1048576_refused_with_its_counts() {
        let all_names: Vec<String> = (1..=1_048_576).map(|row| format!("i{row}")).collect();
        let mut names = NameColumn {
            header: "id".to_owned(),
            number: 1,
            names: all_names,
        };

        assert_eq!(
            export_table(&names, &[], &ExportFormat::Xlsx),
            refused(ExportRefusal::TooLargeForSheet {
                rows: 1_048_576,
                columns: 1
            })
        );
        names.names.pop();
        let bytes = xlsx_of(&names, &[]);
        let (_, range) = calamine_sheet(&bytes);
        assert_eq!(range.end(), Some((1_048_575, 0)));
        assert_eq!(
            calamine_cell(&range, 1_048_575, 0),
            Data::String("i1048575".to_owned())
        );
    }

    // The order of the refusals.

    #[test]
    fn a_table_too_large_for_a_sheet_is_refused_after_the_shape_and_before_any_cell() {
        let wide: Vec<Column> = (2..=16_385)
            .map(|number| column(&format!("c{number}"), texts(&[])))
            .collect();
        let mut wide_wrong_length = wide.clone();
        wide_wrong_length[3] = column("c5", texts(&[Some("p")]));
        let mut wide_empty_name: Vec<Column> = (2..=16_385)
            .map(|number| column(&format!("c{number}"), texts(&[Some("p")])))
            .collect();
        wide_empty_name[0].name = String::new();

        assert_eq!(
            export_table(&names_of("id", &[]), &wide, &ExportFormat::Xlsx),
            refused(ExportRefusal::NoIndividual)
        );
        assert_eq!(
            export_table(
                &names_of("id", &[]),
                &wide_wrong_length,
                &ExportFormat::Xlsx
            ),
            refused(ExportRefusal::WrongLength {
                column: 5,
                expected: 0,
                found: 1
            })
        );
        assert_eq!(
            export_table(
                &names_of("id", &["a"]),
                &wide_empty_name,
                &ExportFormat::Xlsx
            ),
            refused(ExportRefusal::TooLargeForSheet {
                rows: 1,
                columns: 16_385
            })
        );
    }

    #[test]
    fn of_two_refusals_of_cells_the_first_in_the_order_of_the_file_is_given() {
        let names = names_of("id", &["a", "b"]);
        // A refusal of the header before one of a row.
        let header_first = [
            column("x", texts(&[Some(" p"), Some("q")])),
            column("#REF!", texts(&[Some("r"), Some("s")])),
        ];
        // In a row, from left to right.
        let left_first = [
            column("n", ColumnValues::Integer(vec![Some(1), Some(i64::MAX)])),
            column("t", texts(&[Some("p"), Some("q ")])),
        ];
        // A row before the next.
        let row_first = [
            column("t", texts(&[Some("p\u{FFFE}"), Some("q")])),
            column("n", ColumnValues::Integer(vec![Some(1), Some(i64::MIN)])),
        ];

        assert_eq!(
            export_table(&names, &header_first, &ExportFormat::Xlsx),
            refused(ExportRefusal::ErrorAsName { place: at(3, 0) })
        );
        assert_eq!(
            export_table(&names, &left_first, &ExportFormat::Xlsx),
            refused(ExportRefusal::IntegerTooLarge { place: at(2, 2) })
        );
        assert_eq!(
            export_table(&names, &row_first, &ExportFormat::Xlsx),
            refused(ExportRefusal::CannotCarry {
                place: at(2, 1),
                character: '\u{FFFE}'
            })
        );
    }

    #[test]
    fn of_two_refusals_of_one_text_spaces_then_length_then_a_character_is_the_order() {
        let names = names_of("id", &["a"]);
        let spaced_and_long = format!(" {}", "a".repeat(32_767));
        let long_and_fffe = format!("{}\u{FFFE}", "a".repeat(32_767));

        assert_eq!(
            export_table(
                &names,
                &[column("x", texts(&[Some(&spaced_and_long)]))],
                &ExportFormat::Xlsx
            ),
            refused(ExportRefusal::SpacesAtEnds { place: at(2, 1) })
        );
        assert_eq!(
            export_table(
                &names,
                &[column("x", texts(&[Some(&long_and_fffe)]))],
                &ExportFormat::Xlsx
            ),
            refused(ExportRefusal::TextTooLong {
                place: at(2, 1),
                length: 32_768
            })
        );
        assert_eq!(
            export_table(
                &names,
                &[column("#N/A ", texts(&[Some("p")]))],
                &ExportFormat::Xlsx
            ),
            refused(ExportRefusal::SpacesAtEnds { place: at(2, 0) })
        );
    }

    #[test]
    fn a_csv_refusal_is_not_an_xlsx_one() {
        // A first name of a byte order mark and `#CHROM`, which a CSV
        // refuses, is written in an xlsx and reads back as itself.
        let names = names_of("\u{FEFF}#CHROM", &["a"]);
        let columns = [column("ő", texts(&[Some("€")]))];

        let table = read_back(&xlsx_of(&names, &columns)).unwrap();

        assert_eq!(table.names.header, "\u{FEFF}#CHROM");
        assert_eq!(table.columns[0].name, "ő");
        assert_eq!(table.columns[0].values, texts(&[Some("€")]));
    }
}
