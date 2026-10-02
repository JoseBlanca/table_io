//! The CSV `export_table` writes of a table, by "A CSV", "The refusals" and
//! "How it is verified" of `docs/specs/export.md`: each case a literal
//! table, the choices of the CSV and the literal bytes or the refusal, and
//! the cases of the spec read back by `import_table`.
//!
//! The test of a format not built runs in a build without the feature
//! `csv`, `cargo test -p table_io --no-default-features --features xlsx`;
//! the others in every build with it.

#[cfg(not(feature = "csv"))]
use table_io::{
    ColumnValues, CsvEncoding, CsvExport, DecimalMark, ExportError, ExportFormat, ExportRefusal,
    MissingText, NameColumn, Separator, export_table,
};

#[cfg(not(feature = "csv"))]
#[test]
fn a_csv_in_a_build_without_csv_is_a_format_not_built() {
    let names = NameColumn {
        header: "id".to_owned(),
        number: 1,
        names: vec!["A".to_owned()],
    };
    let format = ExportFormat::Csv(CsvExport {
        separator: Separator::Comma,
        decimal: DecimalMark::Point,
        encoding: CsvEncoding::Utf8,
        missing: MissingText::Empty,
    });
    // A column of the wrong length too, which is refused after the format.
    let columns = [table_io::Column {
        name: "h".to_owned(),
        number: 2,
        values: ColumnValues::Integer(vec![]),
    }];

    assert_eq!(
        export_table(&names, &columns, &format),
        Err(ExportError::Refused(ExportRefusal::FormatNotBuilt))
    );
}

#[cfg(all(test, feature = "csv"))]
mod written {
    use table_io::{
        CellPlace, Column, ColumnValues, CsvEncoding, CsvExport, DecimalMark, Encoding,
        ExportError, ExportFormat, ExportRefusal, FoundEncoding, HowRead, ImportError,
        ImportOptions, MissingText, NameColumn, Separator, Table, TextOptions, TextRead,
        export_table, import_table,
    };

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

    /// A CSV with these choices.
    fn csv(
        separator: Separator,
        decimal: DecimalMark,
        encoding: CsvEncoding,
        missing: MissingText,
    ) -> ExportFormat {
        ExportFormat::Csv(CsvExport {
            separator,
            decimal,
            encoding,
            missing,
        })
    }

    /// A CSV in UTF-8 with `separator`, the point and the empty text of a
    /// missing value.
    fn plain(separator: Separator) -> ExportFormat {
        csv(
            separator,
            DecimalMark::Point,
            CsvEncoding::Utf8,
            MissingText::Empty,
        )
    }

    /// The refusal `refusal`, as `export_table` gives it.
    fn refused(refusal: ExportRefusal) -> Result<Vec<u8>, ExportError> {
        Err(ExportError::Refused(refusal))
    }

    /// The place at `column` and `row`.
    fn at(column: u32, row: u32) -> CellPlace {
        CellPlace { column, row }
    }

    /// The table `bytes` read back with `separator`, `decimal` and
    /// `encoding` set, within popnei_web's limits.
    fn read_back(
        bytes: &[u8],
        separator: Separator,
        decimal: DecimalMark,
        encoding: Encoding,
    ) -> Result<Table, ImportError> {
        import_table(
            bytes,
            &ImportOptions {
                max_bytes: 20_000_000,
                max_cells: 2_000_000,
                text: TextOptions {
                    encoding: Some(encoding),
                    separator: Some(separator),
                    decimal: Some(decimal),
                },
            },
        )
    }

    /// The table of the spec: `id`, `h`, `n`, `ok`, `pop` over the rows
    /// `A`, 1.5, 3, true, `P1` and `B`, missing, −2, false, `x;y`.
    fn spec_table() -> (NameColumn, Vec<Column>) {
        (
            names_of("id", &["A", "B"]),
            vec![
                column("h", ColumnValues::Float(vec![Some(1.5), None])),
                column("n", ColumnValues::Integer(vec![Some(3), Some(-2)])),
                column("ok", ColumnValues::Boolean(vec![Some(true), Some(false)])),
                column("pop", texts(&[Some("P1"), Some("x;y")])),
            ],
        )
    }

    // The bytes of a CSV.

    #[test]
    fn the_table_of_the_spec_with_a_semicolon_and_the_comma_in_utf8_with_empty_missing() {
        let (names, columns) = spec_table();
        let format = csv(
            Separator::Semicolon,
            DecimalMark::Comma,
            CsvEncoding::Utf8,
            MissingText::Empty,
        );

        assert_eq!(
            export_table(&names, &columns, &format),
            Ok(b"id;h;n;ok;pop\r\nA;1,5;3;TRUE;P1\r\nB;;-2;FALSE;\"x;y\"\r\n".to_vec())
        );
    }

    #[test]
    fn the_table_of_the_spec_with_a_comma_and_the_point_in_utf8_with_its_mark_and_na() {
        let (names, columns) = spec_table();
        let format = csv(
            Separator::Comma,
            DecimalMark::Point,
            CsvEncoding::Utf8WithMark,
            MissingText::Na,
        );

        assert_eq!(
            export_table(&names, &columns, &format),
            Ok(b"\xEF\xBB\xBFid,h,n,ok,pop\r\nA,1.5,3,TRUE,P1\r\nB,NA,-2,FALSE,x;y\r\n".to_vec())
        );
    }

    #[test]
    fn the_table_of_the_spec_with_the_comma_as_separator_and_decimal_mark_quotes_its_float() {
        let (names, columns) = spec_table();
        let format = csv(
            Separator::Comma,
            DecimalMark::Comma,
            CsvEncoding::Utf8,
            MissingText::Empty,
        );

        assert_eq!(
            export_table(&names, &columns, &format),
            Ok(b"id,h,n,ok,pop\r\nA,\"1,5\",3,TRUE,P1\r\nB,,-2,FALSE,x;y\r\n".to_vec())
        );
    }

    #[test]
    fn a_float_is_its_shortest_digits_with_a_mark_and_a_0_when_it_has_neither_mark_nor_exponent() {
        let names = names_of("id", &["a", "b", "c", "d", "e", "f"]);
        let columns = [column(
            "x",
            ColumnValues::Float(vec![
                Some(1.0),
                Some(0.1 + 0.2),
                Some(1e21),
                Some(1.5e-7),
                Some(-0.0),
                Some(1e20),
            ]),
        )];

        assert_eq!(
            export_table(&names, &columns, &plain(Separator::Comma)),
            Ok(
                b"id,x\r\na,1.0\r\nb,0.30000000000000004\r\nc,1e+21\r\nd,1.5e-7\r\ne,0.0\r\n\
                 f,100000000000000000000.0\r\n"
                    .to_vec()
            )
        );
    }

    #[test]
    fn a_float_with_the_comma_is_written_with_the_comma_and_quoted_when_it_is_the_separator() {
        let names = names_of("id", &["a", "b", "c"]);
        let columns = [column(
            "x",
            ColumnValues::Float(vec![Some(1.75), Some(1.0), Some(-2.5e-9)]),
        )];
        let with = |separator| {
            export_table(
                &names,
                &columns,
                &csv(
                    separator,
                    DecimalMark::Comma,
                    CsvEncoding::Utf8,
                    MissingText::Empty,
                ),
            )
        };

        assert_eq!(
            with(Separator::Semicolon),
            Ok(b"id;x\r\na;1,75\r\nb;1,0\r\nc;-2,5e-9\r\n".to_vec())
        );
        assert_eq!(
            with(Separator::Comma),
            Ok(b"id,x\r\na,\"1,75\"\r\nb,\"1,0\"\r\nc,\"-2,5e-9\"\r\n".to_vec())
        );
    }

    #[test]
    fn a_negative_zero_reads_back_as_0() {
        let names = names_of("id", &["a"]);
        let columns = [column("x", ColumnValues::Float(vec![Some(-0.0)]))];
        let bytes = export_table(&names, &columns, &plain(Separator::Comma)).unwrap();

        let table =
            read_back(&bytes, Separator::Comma, DecimalMark::Point, Encoding::Utf8).unwrap();

        let [read_column] = table.columns.as_slice() else {
            panic!("{table:?}");
        };
        let ColumnValues::Float(floats) = &read_column.values else {
            panic!("{table:?}");
        };
        assert_eq!(floats.len(), 1);
        assert!(floats[0].unwrap().total_cmp(&0.0).is_eq());
    }

    #[test]
    fn a_missing_value_of_each_type_is_written_as_its_text() {
        let names = names_of("id", &["a"]);
        let columns = [
            column("i", ColumnValues::Integer(vec![None])),
            column("f", ColumnValues::Float(vec![None])),
            column("b", ColumnValues::Boolean(vec![None])),
            column("t", texts(&[None])),
        ];
        let with = |missing| {
            export_table(
                &names,
                &columns,
                &csv(
                    Separator::Tab,
                    DecimalMark::Point,
                    CsvEncoding::Utf8,
                    missing,
                ),
            )
        };

        assert_eq!(
            with(MissingText::Empty),
            Ok(b"id\ti\tf\tb\tt\r\na\t\t\t\t\r\n".to_vec())
        );
        assert_eq!(
            with(MissingText::Na),
            Ok(b"id\ti\tf\tb\tt\r\na\tNA\tNA\tNA\tNA\r\n".to_vec())
        );
    }

    // The quoting.

    /// The bytes of the table of one value, the text `text`, with
    /// `separator`.
    fn written_text(text: &str, separator: Separator) -> Result<Vec<u8>, ExportError> {
        export_table(
            &names_of("id", &["a"]),
            &[column("t", texts(&[Some(text)]))],
            &plain(separator),
        )
    }

    #[test]
    fn a_quote_is_doubled_inside_quotes() {
        assert_eq!(
            written_text("say \"hi\"", Separator::Comma),
            Ok(b"id,t\r\na,\"say \"\"hi\"\"\"\r\n".to_vec())
        );
    }

    #[test]
    fn a_line_break_and_a_carriage_return_are_quoted() {
        assert_eq!(
            written_text("a\nb", Separator::Comma),
            Ok(b"id,t\r\na,\"a\nb\"\r\n".to_vec())
        );
        assert_eq!(
            written_text("a\rb", Separator::Comma),
            Ok(b"id,t\r\na,\"a\rb\"\r\n".to_vec())
        );
    }

    #[test]
    fn a_space_at_the_start_or_the_end_is_quoted_and_one_inside_is_not() {
        assert_eq!(
            written_text(" x", Separator::Semicolon),
            Ok(b"id;t\r\na;\" x\"\r\n".to_vec())
        );
        assert_eq!(
            written_text("x ", Separator::Semicolon),
            Ok(b"id;t\r\na;\"x \"\r\n".to_vec())
        );
        assert_eq!(
            written_text("x y", Separator::Semicolon),
            Ok(b"id;t\r\na;x y\r\n".to_vec())
        );
    }

    #[test]
    fn a_tab_inside_is_quoted_only_with_the_tab_as_separator_and_one_at_the_start_with_every_one() {
        assert_eq!(
            written_text("a\tb", Separator::Tab),
            Ok(b"id\tt\r\na\t\"a\tb\"\r\n".to_vec())
        );
        assert_eq!(
            written_text("a\tb", Separator::Semicolon),
            Ok(b"id;t\r\na;a\tb\r\n".to_vec())
        );
        assert_eq!(
            written_text("a\tb", Separator::Comma),
            Ok(b"id,t\r\na,a\tb\r\n".to_vec())
        );
        assert_eq!(
            written_text("\tx", Separator::Tab),
            Ok(b"id\tt\r\na\t\"\tx\"\r\n".to_vec())
        );
        assert_eq!(
            written_text("\tx", Separator::Semicolon),
            Ok(b"id;t\r\na;\"\tx\"\r\n".to_vec())
        );
        assert_eq!(
            written_text("\tx", Separator::Comma),
            Ok(b"id,t\r\na,\"\tx\"\r\n".to_vec())
        );
    }

    #[test]
    fn each_separator_is_quoted_only_when_it_is_the_one_of_the_file() {
        assert_eq!(
            written_text("a;b,c", Separator::Tab),
            Ok(b"id\tt\r\na\ta;b,c\r\n".to_vec())
        );
        assert_eq!(
            written_text("a;b,c", Separator::Semicolon),
            Ok(b"id;t\r\na;\"a;b,c\"\r\n".to_vec())
        );
        assert_eq!(
            written_text("a;b,c", Separator::Comma),
            Ok(b"id,t\r\na,\"a;b,c\"\r\n".to_vec())
        );
    }

    #[test]
    fn a_name_of_a_column_or_of_an_individual_is_quoted_as_a_value_is() {
        let names = names_of("my id", &["P 1 ", "B"]);
        let columns = [column("height ", texts(&[Some("x"), Some("y")]))];

        assert_eq!(
            export_table(&names, &columns, &plain(Separator::Comma)),
            Ok(b"my id,\"height \"\r\n\"P 1 \",x\r\nB,y\r\n".to_vec())
        );
    }

    // The encodings.

    #[test]
    fn utf8_writes_accents_a_character_outside_the_first_plane_and_an_emoji_as_they_are() {
        let names = names_of("Población", &["Ana"]);
        let columns = [column("t", texts(&[Some("𝔸😀ő")]))];

        assert_eq!(
            export_table(&names, &columns, &plain(Separator::Comma)),
            Ok("Población,t\r\nAna,𝔸😀ő\r\n".as_bytes().to_vec())
        );
    }

    /// A CSV in Windows-1252 with `;`, the comma and the empty text of a
    /// missing value.
    fn windows_1252() -> ExportFormat {
        csv(
            Separator::Semicolon,
            DecimalMark::Comma,
            CsvEncoding::Windows1252,
            MissingText::Empty,
        )
    }

    #[test]
    fn windows_1252_writes_an_n_with_tilde_as_f1_and_the_euro_as_80() {
        let names = names_of("id", &["España"]);
        let columns = [column("precio", texts(&[Some("3 €")]))];

        assert_eq!(
            export_table(&names, &columns, &windows_1252()),
            Ok(b"id;precio\r\nEspa\xF1a;3 \x80\r\n".to_vec())
        );
    }

    #[test]
    fn windows_1252_writes_each_of_its_characters_of_80_to_9f_and_the_five_controls_it_keeps() {
        // The 27 characters of 80 to 9F that are not controls, and the five
        // controls of Unicode that the import reads 81, 8D, 8F, 90 and 9D as.
        let text = "€\u{81}‚ƒ„…†‡ˆ‰Š‹Œ\u{8D}Ž\u{8F}\u{90}‘’“”•–—˜™š›œ\u{9D}žŸ";
        let names = names_of("id", &["a"]);
        let columns = [column("t", texts(&[Some(text)]))];

        let mut expected = b"id;t\r\na;".to_vec();
        expected.extend(0x80..=0x9F_u8);
        expected.extend(b"\r\n");
        assert_eq!(
            export_table(&names, &columns, &windows_1252()),
            Ok(expected)
        );
    }

    #[test]
    fn a_character_windows_1252_does_not_have_is_refused_with_its_place() {
        let names = names_of("id", &["a", "b"]);
        let columns = [
            column("n", ColumnValues::Integer(vec![Some(1), Some(2)])),
            column("t", texts(&[Some("x"), Some("Erdős")])),
        ];

        assert_eq!(
            export_table(&names, &columns, &windows_1252()),
            refused(ExportRefusal::CannotCarry {
                place: at(3, 2),
                character: 'ő',
            })
        );
    }

    #[test]
    fn a_control_80_that_windows_1252_does_not_have_is_refused() {
        let names = names_of("id", &["a\u{80}"]);

        assert_eq!(
            export_table(&names, &[], &windows_1252()),
            refused(ExportRefusal::CannotCarry {
                place: at(1, 1),
                character: '\u{80}',
            })
        );
    }

    #[test]
    fn a_text_with_the_euro_in_windows_1252_reads_back_as_itself() {
        let names = names_of("id", &["España"]);
        let columns = [column("precio", texts(&[Some("3 €")]))];
        let bytes = export_table(&names, &columns, &windows_1252()).unwrap();

        assert_eq!(
            read_back(
                &bytes,
                Separator::Semicolon,
                DecimalMark::Comma,
                Encoding::Windows1252
            )
            .unwrap(),
            Table {
                names: NameColumn {
                    header: "id".to_owned(),
                    number: 1,
                    names: vec!["España".to_owned()],
                },
                columns: vec![Column {
                    name: "precio".to_owned(),
                    number: 2,
                    values: texts(&[Some("3 €")]),
                }],
                read: HowRead::Text(TextRead {
                    encoding: FoundEncoding::Windows1252,
                    separator: Separator::Semicolon,
                    decimal: DecimalMark::Comma,
                    undecoded_line: None,
                }),
            }
        );
    }

    // The cases of the spec, read back.

    #[test]
    fn a_text_column_of_001_and_002_reads_back_as_the_integers_1_and_2() {
        let names = names_of("id", &["a", "b"]);
        let columns = [column("code", texts(&[Some("001"), Some("002")]))];
        let bytes = export_table(&names, &columns, &plain(Separator::Comma)).unwrap();

        assert_eq!(bytes, b"id,code\r\na,001\r\nb,002\r\n".to_vec());
        assert_eq!(
            read_back(&bytes, Separator::Comma, DecimalMark::Point, Encoding::Utf8)
                .unwrap()
                .columns,
            vec![Column {
                name: "code".to_owned(),
                number: 2,
                values: ColumnValues::Integer(vec![Some(1), Some(2)]),
            }]
        );
    }

    #[test]
    fn a_name_with_a_space_at_its_end_reads_back_with_it() {
        let names = names_of("id", &["a"]);
        let columns = [column("height ", ColumnValues::Float(vec![Some(1.75)]))];
        let bytes = export_table(&names, &columns, &plain(Separator::Comma)).unwrap();

        assert_eq!(bytes, b"id,\"height \"\r\na,1.75\r\n".to_vec());
        assert_eq!(
            read_back(&bytes, Separator::Comma, DecimalMark::Point, Encoding::Utf8)
                .unwrap()
                .columns,
            vec![Column {
                name: "height ".to_owned(),
                number: 2,
                values: ColumnValues::Float(vec![Some(1.75)]),
            }]
        );
    }

    #[test]
    fn a_text_value_n_a_of_excel_is_written_and_reads_back_as_that_text() {
        let names = names_of("id", &["a"]);
        let columns = [column("t", texts(&[Some("#N/A")]))];
        let bytes = export_table(&names, &columns, &plain(Separator::Comma)).unwrap();

        assert_eq!(bytes, b"id,t\r\na,#N/A\r\n".to_vec());
        assert_eq!(
            read_back(&bytes, Separator::Comma, DecimalMark::Point, Encoding::Utf8)
                .unwrap()
                .columns,
            vec![Column {
                name: "t".to_owned(),
                number: 2,
                values: texts(&[Some("#N/A")]),
            }]
        );
    }

    #[test]
    fn the_integer_2_to_the_60_is_written_as_its_digits_and_reads_back_as_itself() {
        let names = names_of("id", &["a"]);
        let columns = [column(
            "n",
            ColumnValues::Integer(vec![Some(1_152_921_504_606_846_976)]),
        )];
        let bytes = export_table(&names, &columns, &plain(Separator::Comma)).unwrap();

        assert_eq!(bytes, b"id,n\r\na,1152921504606846976\r\n".to_vec());
        assert_eq!(
            read_back(&bytes, Separator::Comma, DecimalMark::Point, Encoding::Utf8)
                .unwrap()
                .columns,
            vec![Column {
                name: "n".to_owned(),
                number: 2,
                values: ColumnValues::Integer(vec![Some(1_152_921_504_606_846_976)]),
            }]
        );
    }

    #[test]
    fn a_text_with_the_replacement_character_reads_back_as_itself_with_its_line_reported() {
        let names = names_of("id", &["a", "b"]);
        let columns = [column("t", texts(&[Some("x"), Some("y\u{FFFD}")]))];
        let bytes = export_table(&names, &columns, &plain(Separator::Comma)).unwrap();

        let table =
            read_back(&bytes, Separator::Comma, DecimalMark::Point, Encoding::Utf8).unwrap();

        assert_eq!(
            table.columns[0].values,
            texts(&[Some("x"), Some("y\u{FFFD}")])
        );
        assert_eq!(
            table.read,
            HowRead::Text(TextRead {
                encoding: FoundEncoding::Utf8,
                separator: Separator::Comma,
                decimal: DecimalMark::Point,
                undecoded_line: Some(3),
            })
        );
    }

    #[test]
    fn an_empty_name_of_the_names_column_with_other_columns_is_written() {
        let names = names_of("", &["a"]);
        let columns = [column("t", texts(&[Some("x")]))];

        assert_eq!(
            export_table(&names, &columns, &plain(Separator::Semicolon)),
            Ok(b";t\r\na;x\r\n".to_vec())
        );
    }

    #[test]
    fn a_table_of_the_names_alone_is_written() {
        let names = names_of("id", &["a", "b"]);

        assert_eq!(
            export_table(&names, &[], &plain(Separator::Semicolon)),
            Ok(b"id\r\na\r\nb\r\n".to_vec())
        );
    }

    #[test]
    fn an_individual_or_a_column_named_na_is_written_since_only_a_value_reads_back_as_missing() {
        let names = names_of("-", &["NA", "-"]);
        let columns = [column("NA", ColumnValues::Integer(vec![Some(1), None]))];
        let bytes = export_table(&names, &columns, &plain(Separator::Comma)).unwrap();

        assert_eq!(bytes, b"-,NA\r\nNA,1\r\n-,\r\n".to_vec());
        let table =
            read_back(&bytes, Separator::Comma, DecimalMark::Point, Encoding::Utf8).unwrap();
        assert_eq!(table.names.header, "-");
        assert_eq!(table.names.names, ["NA", "-"]);
        assert_eq!(table.columns[0].name, "NA");
    }

    // The refusals.

    #[test]
    fn a_table_with_no_individual_is_refused() {
        let names = names_of("id", &[]);
        let columns = [column("t", texts(&[]))];

        assert_eq!(
            export_table(&names, &columns, &plain(Separator::Comma)),
            refused(ExportRefusal::NoIndividual)
        );
    }

    #[test]
    fn a_column_of_the_wrong_length_is_refused_with_its_column_and_both_counts() {
        let names = names_of("id", &["a", "b"]);
        let columns = [
            column("x", ColumnValues::Integer(vec![Some(1), Some(2)])),
            column("y", ColumnValues::Boolean(vec![Some(true), None, None])),
            column("z", texts(&[])),
        ];

        assert_eq!(
            export_table(&names, &columns, &plain(Separator::Comma)),
            refused(ExportRefusal::WrongLength {
                column: 3,
                expected: 2,
                found: 3,
            })
        );
    }

    #[test]
    fn a_column_of_the_wrong_length_is_refused_before_a_table_with_no_individual() {
        let names = names_of("id", &[]);
        let columns = [column("x", ColumnValues::Float(vec![Some(1.0)]))];

        assert_eq!(
            export_table(&names, &columns, &plain(Separator::Comma)),
            refused(ExportRefusal::WrongLength {
                column: 2,
                expected: 0,
                found: 1,
            })
        );
    }

    #[test]
    fn an_empty_name_of_a_column_is_refused_with_its_column() {
        let names = names_of("id", &["a"]);
        let columns = [
            column("x", texts(&[Some("p")])),
            column("", texts(&[Some("q")])),
        ];

        assert_eq!(
            export_table(&names, &columns, &plain(Separator::Comma)),
            refused(ExportRefusal::EmptyName { column: 3 })
        );
    }

    #[test]
    fn an_empty_name_of_the_names_column_of_a_table_with_no_other_column_is_refused() {
        let names = names_of("", &["a"]);

        assert_eq!(
            export_table(&names, &[], &plain(Separator::Comma)),
            refused(ExportRefusal::EmptyName { column: 1 })
        );
    }

    #[test]
    fn two_columns_of_one_name_the_names_column_among_them_are_refused_with_both_columns() {
        let names = names_of("id", &["a"]);
        let columns = [
            column("x", texts(&[Some("p")])),
            column("id", texts(&[Some("q")])),
            column("x", texts(&[Some("r")])),
        ];

        assert_eq!(
            export_table(&names, &columns, &plain(Separator::Comma)),
            refused(ExportRefusal::DuplicateName {
                name: "id".to_owned(),
                first_column: 1,
                second_column: 3,
            })
        );
    }

    #[test]
    fn an_empty_individual_is_refused_with_its_row() {
        let names = names_of("id", &["a", "b", ""]);
        let columns = [column("x", ColumnValues::Integer(vec![None, None, None]))];

        assert_eq!(
            export_table(&names, &columns, &plain(Separator::Comma)),
            refused(ExportRefusal::EmptyIndividual { row: 3 })
        );
    }

    #[test]
    fn an_individual_in_two_rows_is_refused_with_both_rows() {
        let names = names_of("id", &["a", "b", "c", "b"]);

        assert_eq!(
            export_table(&names, &[], &plain(Separator::Comma)),
            refused(ExportRefusal::DuplicateIndividual {
                name: "b".to_owned(),
                first_row: 2,
                second_row: 4,
            })
        );
    }

    #[test]
    fn a_text_that_reads_back_as_missing_is_refused_whatever_the_text_of_a_missing_value_is() {
        let names = names_of("id", &["a", "b"]);
        for missing_text in ["", "NA", "-"] {
            for missing in [MissingText::Empty, MissingText::Na] {
                let columns = [
                    column("n", ColumnValues::Integer(vec![None, None])),
                    column("t", texts(&[Some("x"), Some(missing_text)])),
                ];
                let format = csv(
                    Separator::Comma,
                    DecimalMark::Point,
                    CsvEncoding::Utf8,
                    missing,
                );

                assert_eq!(
                    export_table(&names, &columns, &format),
                    refused(ExportRefusal::ReadsAsMissing { place: at(3, 2) }),
                    "{missing_text:?} {missing:?}"
                );
            }
        }
    }

    #[test]
    fn a_float_that_is_not_finite_is_refused_with_its_place() {
        let names = names_of("id", &["a", "b"]);
        for not_finite in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let columns = [column(
                "h",
                ColumnValues::Float(vec![Some(1.0), Some(not_finite)]),
            )];

            assert_eq!(
                export_table(&names, &columns, &plain(Separator::Comma)),
                refused(ExportRefusal::NotFinite { place: at(2, 2) }),
                "{not_finite}"
            );
        }
    }

    #[test]
    fn a_character_0_is_refused_in_every_encoding_with_its_place() {
        let names = names_of("id", &["a"]);
        let columns = [column("t", texts(&[Some("x\u{0}y")]))];
        for encoding in [
            CsvEncoding::Utf8,
            CsvEncoding::Utf8WithMark,
            CsvEncoding::Windows1252,
        ] {
            let format = csv(
                Separator::Comma,
                DecimalMark::Point,
                encoding,
                MissingText::Empty,
            );

            assert_eq!(
                export_table(&names, &columns, &format),
                refused(ExportRefusal::CannotCarry {
                    place: at(2, 1),
                    character: '\u{0}',
                }),
                "{encoding:?}"
            );
        }
    }

    #[test]
    fn a_byte_order_mark_at_the_start_of_the_first_name_is_refused_and_one_elsewhere_is_not() {
        let names = names_of("\u{FEFF}id", &["a"]);
        for encoding in [CsvEncoding::Utf8, CsvEncoding::Utf8WithMark] {
            let format = csv(
                Separator::Comma,
                DecimalMark::Point,
                encoding,
                MissingText::Empty,
            );

            assert_eq!(
                export_table(&names, &[], &format),
                refused(ExportRefusal::CannotCarry {
                    place: at(1, 0),
                    character: '\u{FEFF}',
                }),
                "{encoding:?}"
            );
        }
        let names = names_of("i\u{FEFF}d", &["\u{FEFF}a"]);
        let columns = [column("\u{FEFF}t", texts(&[Some("\u{FEFF}x")]))];
        assert_eq!(
            export_table(&names, &columns, &plain(Separator::Comma)),
            Ok("i\u{FEFF}d,\u{FEFF}t\r\n\u{FEFF}a,\u{FEFF}x\r\n"
                .as_bytes()
                .to_vec())
        );
    }

    #[test]
    fn a_header_that_starts_as_a_variants_file_is_refused() {
        let vcf = |header: &str, separator| {
            export_table(
                &names_of(header, &["a"]),
                &[column("#CHROM", texts(&[Some("x")]))],
                &plain(separator),
            )
        };

        assert_eq!(
            vcf("##fileformat=VCFv4.2", Separator::Comma),
            refused(ExportRefusal::ReadsAsVariantsFile)
        );
        assert_eq!(
            vcf("", Separator::Tab),
            refused(ExportRefusal::ReadsAsVariantsFile)
        );
        assert_eq!(
            vcf("", Separator::Semicolon),
            Ok(b";#CHROM\r\na;x\r\n".to_vec())
        );
        assert_eq!(
            vcf(" #CHROM", Separator::Comma),
            Ok(b"\" #CHROM\",#CHROM\r\na,x\r\n".to_vec())
        );
    }

    #[test]
    fn a_header_that_starts_as_a_variants_file_is_refused_before_a_refusal_of_a_cell() {
        let names = names_of("#CHROM", &["a", "a"]);
        let columns = [column("", texts(&[Some(""), Some("x")]))];

        assert_eq!(
            export_table(&names, &columns, &plain(Separator::Comma)),
            refused(ExportRefusal::ReadsAsVariantsFile)
        );
    }

    #[test]
    fn of_two_refusals_of_cells_the_first_in_the_order_of_the_file_is_given() {
        let names = names_of("id", &["a", "b"]);
        let columns = [
            column("x", ColumnValues::Float(vec![Some(1.0), Some(f64::NAN)])),
            column("y", texts(&[Some("p"), Some("NA")])),
        ];
        // Row 2: the float of column 2 before the text of column 3.
        assert_eq!(
            export_table(&names, &columns, &plain(Separator::Comma)),
            refused(ExportRefusal::NotFinite { place: at(2, 2) })
        );

        // The header before the rows.
        let columns = [
            column("x", ColumnValues::Float(vec![Some(1.0), Some(f64::NAN)])),
            column("x", texts(&[Some("p"), Some("NA")])),
        ];
        assert_eq!(
            export_table(&names, &columns, &plain(Separator::Comma)),
            refused(ExportRefusal::DuplicateName {
                name: "x".to_owned(),
                first_column: 2,
                second_column: 3,
            })
        );

        // Row 1 before row 2, the last column of row 1 before the first
        // of row 2.
        let names = names_of("id", &["a", ""]);
        let columns = [column("y", texts(&[Some("-"), Some("q")]))];
        assert_eq!(
            export_table(&names, &columns, &plain(Separator::Comma)),
            refused(ExportRefusal::ReadsAsMissing { place: at(2, 1) })
        );
    }

    #[cfg(not(feature = "xlsx"))]
    #[test]
    fn an_xlsx_in_a_build_without_xlsx_is_a_format_not_built() {
        let names = names_of("id", &["a"]);

        assert_eq!(
            export_table(&names, &[], &ExportFormat::Xlsx),
            refused(ExportRefusal::FormatNotBuilt)
        );
    }
}
