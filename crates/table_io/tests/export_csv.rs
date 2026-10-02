//! The CSV `export_table` writes of a table, by "A CSV", "The refusals" and
//! "How it is verified" of `docs/specs/export.md`: each case a literal
//! table, the choices of the CSV and the literal bytes or the refusal, and
//! the cases of the spec read back by `import_table`; and the property of
//! "What reads back", over generated tables exported with each of the 36
//! combinations of the choices of a CSV.
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

#[cfg(test)]
#[cfg(feature = "csv")]
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

    #[test]
    fn the_smallest_and_the_largest_floats_are_written_with_their_shortest_digits() {
        let names = names_of("id", &["a", "b"]);
        let columns = [column(
            "x",
            ColumnValues::Float(vec![Some(5e-324), Some(f64::MAX)]),
        )];

        assert_eq!(
            export_table(&names, &columns, &plain(Separator::Comma)),
            Ok(b"id,x\r\na,5e-324\r\nb,1.7976931348623157e+308\r\n".to_vec())
        );
    }

    #[test]
    fn a_text_holding_u_fffe_is_written_in_utf8() {
        assert_eq!(
            written_text("a\u{FFFE}", Separator::Comma),
            Ok("id,t\r\na,a\u{FFFE}\r\n".as_bytes().to_vec())
        );
    }

    #[test]
    fn a_first_name_starting_with_a_hash_that_is_not_a_variants_file_is_written() {
        let names = names_of("#x", &["a"]);

        assert_eq!(
            export_table(&names, &[], &plain(Separator::Tab)),
            Ok(b"#x\r\na\r\n".to_vec())
        );
    }

    #[test]
    fn a_text_column_comes_back_integer_float_or_boolean_when_every_value_is_one() {
        let names = names_of("id", &["a", "b"]);
        let columns = [
            column("whole", texts(&[Some("+5"), Some("-0")])),
            column("number", texts(&[Some("1,5"), None])),
            column("boolean", texts(&[Some("True"), Some("false")])),
        ];
        let format = csv(
            Separator::Semicolon,
            DecimalMark::Comma,
            CsvEncoding::Utf8,
            MissingText::Na,
        );
        let bytes = export_table(&names, &columns, &format).unwrap();

        assert_eq!(
            bytes,
            b"id;whole;number;boolean\r\na;+5;1,5;True\r\nb;-0;NA;false\r\n".to_vec()
        );
        let read_values: Vec<ColumnValues> = read_back(
            &bytes,
            Separator::Semicolon,
            DecimalMark::Comma,
            Encoding::Utf8,
        )
        .unwrap()
        .columns
        .into_iter()
        .map(|read_column| read_column.values)
        .collect();
        assert_eq!(
            read_values,
            vec![
                ColumnValues::Integer(vec![Some(5), Some(0)]),
                ColumnValues::Float(vec![Some(1.5), None]),
                ColumnValues::Boolean(vec![Some(true), Some(false)]),
            ]
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
    fn a_tab_at_the_end_is_quoted_with_every_separator() {
        assert_eq!(
            written_text("x\t", Separator::Tab),
            Ok(b"id\tt\r\na\t\"x\t\"\r\n".to_vec())
        );
        assert_eq!(
            written_text("x\t", Separator::Semicolon),
            Ok(b"id;t\r\na;\"x\t\"\r\n".to_vec())
        );
        assert_eq!(
            written_text("x\t", Separator::Comma),
            Ok(b"id,t\r\na,\"x\t\"\r\n".to_vec())
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
    fn a_table_with_no_individual_is_refused_before_a_header_that_reads_as_a_variants_file() {
        let names = names_of("", &[]);
        let columns = [column("#CHROM", texts(&[]))];

        assert_eq!(
            export_table(&names, &columns, &plain(Separator::Tab)),
            refused(ExportRefusal::NoIndividual)
        );
    }

    #[test]
    fn a_first_name_of_a_byte_order_mark_and_chrom_reads_as_a_variants_file() {
        let names = names_of("\u{FEFF}#CHROM", &["a"]);

        assert_eq!(
            export_table(&names, &[], &plain(Separator::Comma)),
            refused(ExportRefusal::ReadsAsVariantsFile)
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

    // The bytes of a CSV of 20,000 rows of 100 floats, 16.9 MB, held
    // 33.6 MB, the capacity their Vec had doubled to (release build, on the
    // owner's Mac, 2 October 2026).
    #[test]
    fn the_bytes_of_a_csv_hold_no_more_memory_than_their_length() {
        let num_rows = 1_000;
        let names = NameColumn {
            header: "id".to_owned(),
            number: 1,
            names: (0..num_rows).map(|row| format!("ind{row}")).collect(),
        };
        let columns = [column(
            "x",
            ColumnValues::Float(
                (0..num_rows)
                    .map(|row| Some(f64::from(row) / 8.0))
                    .collect(),
            ),
        )];

        let bytes = export_table(&names, &columns, &plain(Separator::Comma)).unwrap();

        assert_eq!(bytes.capacity(), bytes.len());
    }

    #[test]
    fn an_export_error_is_written_in_english_with_its_fields_and_is_an_error() {
        let names = names_of("id", &["a", "b"]);
        let columns = [column("t", texts(&[Some("x"), Some("NA")]))];
        let error = export_table(&names, &columns, &plain(Separator::Comma)).unwrap_err();
        let as_error: &dyn std::error::Error = &error;

        assert_eq!(
            as_error.to_string(),
            "an export refused: the text at column 2 and row 2 reads back as a missing value"
        );
        assert_eq!(
            ExportError::Refused(ExportRefusal::CannotCarry {
                place: at(4, 0),
                character: 'ő',
            })
            .to_string(),
            "an export refused: the character 'ő' (U+0151) at column 4 and row 0 cannot be \
             carried by the file"
        );
        assert_eq!(
            ExportError::Refused(ExportRefusal::DuplicateIndividual {
                name: "b".to_owned(),
                first_row: 2,
                second_row: 4,
            })
            .to_string(),
            "an export refused: the individual 'b' is in the rows 2 and 4"
        );
        assert_eq!(
            ExportError::Failed("a message of rust_xlsxwriter".to_owned()).to_string(),
            "an export failed: a message of rust_xlsxwriter"
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

// The round trip of "What reads back" of docs/specs/export.md, for a CSV.
#[cfg(test)]
#[cfg(feature = "csv")]
mod round_trip {
    use table_io::{
        CellPlace, Column, ColumnType, ColumnValues, CsvEncoding, CsvExport, DecimalMark, Encoding,
        ExportError, ExportFormat, ExportRefusal, ImportError, ImportOptions, MissingText,
        NameColumn, Refusal, Separator, TextOptions, convert_column, export_table, import_table,
        is_missing, parse_boolean, parse_float, parse_integer,
    };

    /// The number of tables the property is tried on, each exported with
    /// the 36 combinations of the choices of a CSV.
    const NUM_TABLES: u32 = 2_000;

    /// The seed of the generator of the tables, fixed so that a failure is
    /// seen again.
    const SEED: u64 = 0xE4_7AB1_E0C5_7E57;

    /// The most times a table refused is made again for one combination;
    /// each repair removes one value or character of the few in a table.
    const MAX_REPAIRS: u32 = 200;

    /// A generator of numbers, xorshift64*, which needs no dependency.
    struct Generator {
        /// The state, never 0.
        state: u64,
    }

    impl Generator {
        /// The next number.
        fn next_number(&mut self) -> u64 {
            self.state ^= self.state >> 12;
            self.state ^= self.state << 25;
            self.state ^= self.state >> 27;
            self.state.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }

        /// A number from 0 to `below` − 1.
        fn below(&mut self, below: usize) -> usize {
            usize::try_from(
                self.next_number()
                    .checked_rem(u64::try_from(below).unwrap())
                    .unwrap(),
            )
            .unwrap()
        }

        /// Whether a chance of 1 in `chances` came.
        fn one_in(&mut self, chances: usize) -> bool {
            self.below(chances) == 0
        }

        /// One of `choices`.
        fn one_of<'choice, T>(&mut self, choices: &'choice [T]) -> &'choice T {
            &choices[self.below(choices.len())]
        }
    }

    /// The characters of a generated name or text: accents, `€`, which
    /// Windows-1252 has, `ő`, which it has not, a character outside the
    /// first plane, an emoji, a quote, the three separators, the line
    /// breaks, a space, `#`, U+FFFD, U+FFFE and U+0000, and letters and
    /// digits, none of `f`, `i` and `x`, which the names of a repair take.
    /// No `ÿ`, `þ`, `ï` or control below U+0020 but the tab and the line
    /// breaks: a first name that starts with `ÿþ`, `þÿ` or `ï»¿` in
    /// Windows-1252, or with `PK` and the controls 3 and 4, is written and
    /// does not read back, a question for the owner.
    const CHARACTERS: [char; 29] = [
        'a', 'B', 'n', 'A', 'N', '1', '7', '0', '.', '-', '+', 'e', 'E', 'ó', 'ñ', '€', 'ő', '𝔸',
        '😀', '"', ',', ';', '\t', '\n', '\r', ' ', '#', '\u{FFFD}', '\u{FFFE}',
    ];

    /// A text of 0 to 5 of [`CHARACTERS`], with U+0000 in 1 in 50 and a
    /// U+FEFF at its start in 1 in 10.
    fn generated_text(generator: &mut Generator) -> String {
        let length = generator.below(6);
        let mut text: String = (0..length)
            .map(|_| *generator.one_of(&CHARACTERS))
            .collect();
        if generator.one_in(50) {
            text.push('\0');
        }
        if generator.one_in(10) {
            text.insert(0, '\u{FEFF}');
        }
        text
    }

    /// A generated text, not empty and none of `taken`.
    fn generated_name(generator: &mut Generator, taken: &[String]) -> String {
        loop {
            let name = generated_text(generator);
            if !name.is_empty() && !taken.contains(&name) {
                return name;
            }
        }
    }

    /// The value of an integer column: small, at 2^53 + 1, at the ends of
    /// an integer, any of 64 bits, or missing.
    fn generated_integer(generator: &mut Generator) -> Option<i64> {
        match generator.below(6) {
            0 => None,
            1 => Some(*generator.one_of(&[
                9_007_199_254_740_993,
                -9_007_199_254_740_993,
                i64::MAX,
                i64::MIN,
                0,
            ])),
            2 => Some(i64::from_ne_bytes(generator.next_number().to_ne_bytes())),
            _ => Some(
                i64::try_from(generator.below(2_001))
                    .unwrap()
                    .checked_sub(1_000)
                    .unwrap(),
            ),
        }
    }

    /// The value of a float column: one with decimals, a whole one, −0,
    /// one of the floats of the spec, any finite float of 64 bits, a float
    /// that is not finite in 1 in 40, or missing.
    fn generated_float(generator: &mut Generator) -> Option<f64> {
        if generator.one_in(40) {
            return Some(*generator.one_of(&[f64::NAN, f64::INFINITY, f64::NEG_INFINITY]));
        }
        match generator.below(5) {
            0 => None,
            1 => Some(*generator.one_of(&[
                1.5,
                1.0,
                -3.0,
                -0.0,
                0.1 + 0.2,
                1e21,
                1e20,
                1.5e-7,
                5e-324,
                f64::MAX,
                9_007_199_254_740_993.0,
            ])),
            2 => {
                let float = f64::from_bits(generator.next_number());
                float.is_finite().then_some(float)
            }
            _ => Some(f64::from(u32::try_from(generator.below(10_000)).unwrap()) / 8.0 - 600.0),
        }
    }

    /// The texts of a text column of numbers, whole or with either mark,
    /// and of booleans, that read back as a narrower type.
    const NUMBER_TEXTS: [&str; 12] = [
        "001",
        "-3",
        "+5",
        "9007199254740993",
        "1,5",
        "-2,5e-3",
        "1.5",
        "1e3",
        "TRUE",
        "false",
        "True",
        "0",
    ];

    /// The values of a text column: generated texts, those of
    /// [`NUMBER_TEXTS`] of one kind, so that the column reads back
    /// narrower, or any of them, with missing values.
    fn generated_texts(generator: &mut Generator, num_rows: usize) -> Vec<Option<String>> {
        let kind = generator.below(5);
        (0..num_rows)
            .map(|_| {
                if generator.one_in(4) {
                    return None;
                }
                Some(match kind {
                    0 => (*generator.one_of(&NUMBER_TEXTS[..4])).to_owned(),
                    1 => (*generator.one_of(&NUMBER_TEXTS[4..8])).to_owned(),
                    2 => (*generator.one_of(&NUMBER_TEXTS[8..])).to_owned(),
                    3 => (*generator.one_of(&NUMBER_TEXTS)).to_owned(),
                    _ => generated_text(generator),
                })
            })
            .collect()
    }

    /// A table of 0 to 5 columns besides the names' and 0 to 6 rows, of
    /// the four types; an empty name of the names' column in 1 in 6, and
    /// in 1 in 10 of those a first column `#CHROM`.
    fn generated_table(generator: &mut Generator) -> (NameColumn, Vec<Column>) {
        let num_columns = generator.below(6);
        let num_rows = if generator.one_in(12) {
            0
        } else {
            generator.below(6).checked_add(1).unwrap()
        };
        let mut header = vec![generated_name(generator, &[])];
        if num_columns > 0 && generator.one_in(6) {
            header[0] = String::new();
            if generator.one_in(10) {
                header.push("#CHROM".to_owned());
            }
        }
        while header.len() <= num_columns {
            let name = generated_name(generator, &header);
            header.push(name);
        }
        let mut names = Vec::new();
        for _ in 0..num_rows {
            let name = generated_name(generator, &names);
            names.push(name);
        }
        let columns = header[1..]
            .iter()
            .map(|name| Column {
                name: name.clone(),
                number: 0,
                values: match generator.below(4) {
                    0 => ColumnValues::Integer(
                        (0..num_rows)
                            .map(|_| generated_integer(generator))
                            .collect(),
                    ),
                    1 => ColumnValues::Float(
                        (0..num_rows).map(|_| generated_float(generator)).collect(),
                    ),
                    2 => ColumnValues::Boolean(
                        (0..num_rows)
                            .map(|_| [None, Some(true), Some(false)][generator.below(3)])
                            .collect(),
                    ),
                    _ => ColumnValues::Text(generated_texts(generator, num_rows)),
                },
            })
            .collect();
        (
            NameColumn {
                header: header[0].clone(),
                number: 0,
                names,
            },
            columns,
        )
    }

    /// The 36 combinations of the choices of a CSV.
    fn combinations() -> Vec<CsvExport> {
        let mut combinations = Vec::new();
        for separator in [Separator::Tab, Separator::Semicolon, Separator::Comma] {
            for decimal in [DecimalMark::Point, DecimalMark::Comma] {
                for encoding in [
                    CsvEncoding::Utf8,
                    CsvEncoding::Utf8WithMark,
                    CsvEncoding::Windows1252,
                ] {
                    for missing in [MissingText::Empty, MissingText::Na] {
                        combinations.push(CsvExport {
                            separator,
                            decimal,
                            encoding,
                            missing,
                        });
                    }
                }
            }
        }
        combinations
    }

    /// The text of the cell at `place`: a name of the header, of an
    /// individual, or a text value.
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "a refusal names a column from 1 and a row from 0, of a table of a few"
    )]
    fn text_at<'table>(
        names: &'table mut NameColumn,
        columns: &'table mut [Column],
        place: CellPlace,
    ) -> &'table mut String {
        let row = usize::try_from(place.row).unwrap();
        let column = usize::try_from(place.column).unwrap();
        match (row, column) {
            (0, 1) => &mut names.header,
            (0, _) => &mut columns[column - 2].name,
            (_, 1) => &mut names.names[row - 1],
            (_, _) => match &mut columns[column - 2].values {
                ColumnValues::Text(texts) => texts[row - 1].as_mut().unwrap(),
                other @ (ColumnValues::Integer(_)
                | ColumnValues::Float(_)
                | ColumnValues::Boolean(_)) => panic!("no text at {place:?} of {other:?}"),
            },
        }
    }

    /// Makes the value at `place` missing.
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "a refusal of a value names a column from 2 and a row from 1"
    )]
    fn make_missing(columns: &mut [Column], place: CellPlace) {
        let row = usize::try_from(place.row).unwrap() - 1;
        match &mut columns[usize::try_from(place.column).unwrap() - 2].values {
            ColumnValues::Integer(integers) => integers[row] = None,
            ColumnValues::Float(floats) => floats[row] = None,
            ColumnValues::Boolean(booleans) => booleans[row] = None,
            ColumnValues::Text(texts) => texts[row] = None,
        }
    }

    /// The characters of Windows-1252 for the bytes `80` to `9F`, by the
    /// index of the Encoding Standard, the five controls it keeps among
    /// them; every other byte but 0 is the character of its number.
    const WINDOWS_1252_80_TO_9F: &str = "€\u{81}‚ƒ„…†‡ˆ‰Š‹Œ\u{8D}Ž\u{8F}\u{90}‘’“”•–—˜™š›œ\u{9D}žŸ";

    /// Whether Windows-1252 has `character`.
    fn is_in_windows_1252(character: char) -> bool {
        matches!(u32::from(character), 0x01..=0x7F | 0xA0..=0xFF)
            || WINDOWS_1252_80_TO_9F.contains(character)
    }

    /// Whether a CSV with `csv_export` cannot carry `character` of the
    /// text `text` at `place`: U+0000, a U+FEFF at the start of the first
    /// name of the header, or in Windows-1252 a character it has not.
    fn cannot_carry(character: char, text: &str, place: CellPlace, csv_export: &CsvExport) -> bool {
        character == '\0'
            || (character == '\u{FEFF}'
                && place == (CellPlace { column: 1, row: 0 })
                && text.starts_with('\u{FEFF}'))
            || (csv_export.encoding == CsvEncoding::Windows1252 && !is_in_windows_1252(character))
    }

    /// The character of `separator`.
    fn separator_character(separator: Separator) -> char {
        match separator {
            Separator::Tab => '\t',
            Separator::Semicolon => ';',
            Separator::Comma => ',',
        }
    }

    /// `cell` as "A CSV" of the spec writes it with `separator`: in
    /// quotes, each quote doubled, when it holds the separator, a quote or
    /// a line break, or a space or a tab at its ends.
    fn written_cell(cell: &str, separator: Separator) -> String {
        let needs_quotes = cell.contains([separator_character(separator), '"', '\n', '\r'])
            || cell.starts_with([' ', '\t'])
            || cell.ends_with([' ', '\t']);
        if needs_quotes {
            format!("\"{}\"", cell.replace('"', "\"\""))
        } else {
            cell.to_owned()
        }
    }

    /// Whether the import refuses as a variants file a file whose header is
    /// that of `names` and `columns`, written with `separator`; its
    /// characters U+0000, which the import refuses first as not text, left
    /// out.
    fn reads_as_variants_file(
        names: &NameColumn,
        columns: &[Column],
        separator: Separator,
    ) -> bool {
        let line = std::iter::once(names.header.as_str())
            .chain(columns.iter().map(|column| column.name.as_str()))
            .map(|name| written_cell(name, separator))
            .collect::<Vec<_>>()
            .join(&separator_character(separator).to_string())
            .replace('\0', "");
        let options = ImportOptions {
            max_bytes: 20_000_000,
            max_cells: 2_000_000,
            text: TextOptions {
                encoding: Some(Encoding::Utf8),
                separator: Some(separator),
                decimal: None,
            },
        };
        matches!(
            import_table(format!("{line}\r\nz\r\n").as_bytes(), &options),
            Err(ImportError::Refused {
                refusal: Refusal::VariantsFile,
                ..
            })
        )
    }

    /// Asserts that `refusal` of `names` and `columns`, exported with
    /// `csv_export`, is due by the rule of the spec it names.
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "a refusal of a value names a column from 2 and a row from 1"
    )]
    fn assert_due(
        names: &NameColumn,
        columns: &[Column],
        refusal: &ExportRefusal,
        csv_export: &CsvExport,
    ) {
        let mut names_copy = names.clone();
        let mut columns_copy = columns.to_vec();
        let mut text_of =
            |place: CellPlace| text_at(&mut names_copy, &mut columns_copy, place).clone();
        let header_place = |column: u32| CellPlace { column, row: 0 };
        let name_place = |row: u32| CellPlace { column: 1, row };
        let is_due = match refusal {
            ExportRefusal::EmptyName { column } => {
                text_of(header_place(*column)).is_empty() && (*column != 1 || columns.is_empty())
            }
            ExportRefusal::DuplicateName {
                name,
                first_column,
                second_column,
            } => {
                first_column < second_column
                    && text_of(header_place(*first_column)) == *name
                    && text_of(header_place(*second_column)) == *name
            }
            ExportRefusal::EmptyIndividual { row } => text_of(name_place(*row)).is_empty(),
            ExportRefusal::DuplicateIndividual {
                name,
                first_row,
                second_row,
            } => {
                first_row < second_row
                    && text_of(name_place(*first_row)) == *name
                    && text_of(name_place(*second_row)) == *name
            }
            ExportRefusal::ReadsAsMissing { place } => is_missing(&text_of(*place)),
            ExportRefusal::NotFinite { place } => {
                let row = usize::try_from(place.row).unwrap() - 1;
                match &columns[usize::try_from(place.column).unwrap() - 2].values {
                    ColumnValues::Float(floats) => {
                        floats[row].is_some_and(|float| !float.is_finite())
                    }
                    ColumnValues::Integer(_) | ColumnValues::Boolean(_) | ColumnValues::Text(_) => {
                        false
                    }
                }
            }
            ExportRefusal::CannotCarry { place, character } => {
                let text = text_of(*place);
                let first = text.chars().find(|&text_character| {
                    cannot_carry(text_character, &text, *place, csv_export)
                });
                first == Some(*character)
            }
            ExportRefusal::ReadsAsVariantsFile => {
                reads_as_variants_file(names, columns, csv_export.separator)
            }
            ExportRefusal::FormatNotBuilt
            | ExportRefusal::NoIndividual
            | ExportRefusal::WrongLength { .. }
            | ExportRefusal::ErrorAsName { .. }
            | ExportRefusal::SpacesAtEnds { .. }
            | ExportRefusal::IntegerTooLarge { .. }
            | ExportRefusal::TextTooLong { .. }
            | ExportRefusal::TooLargeForSheet { .. } => false,
        };
        assert!(
            is_due,
            "a refusal not due: {refusal:?} of {csv_export:?} {names:?} {columns:?}"
        );
    }

    /// The table made again without what `refusal` names: a name given
    /// that of a repair, `fix` and a number, a value made missing, or the
    /// character that cannot be carried removed.
    fn repaired(
        names: &mut NameColumn,
        columns: &mut [Column],
        refusal: &ExportRefusal,
        num_repairs: u32,
    ) {
        let fixed_name = format!("fix{num_repairs}");
        match refusal {
            ExportRefusal::EmptyName { column } => {
                *text_at(
                    names,
                    columns,
                    CellPlace {
                        column: *column,
                        row: 0,
                    },
                ) = fixed_name;
            }
            ExportRefusal::DuplicateName { second_column, .. } => {
                *text_at(
                    names,
                    columns,
                    CellPlace {
                        column: *second_column,
                        row: 0,
                    },
                ) = fixed_name;
            }
            ExportRefusal::EmptyIndividual { row }
            | ExportRefusal::DuplicateIndividual {
                second_row: row, ..
            } => {
                *text_at(
                    names,
                    columns,
                    CellPlace {
                        column: 1,
                        row: *row,
                    },
                ) = fixed_name;
            }
            ExportRefusal::ReadsAsMissing { place } | ExportRefusal::NotFinite { place } => {
                make_missing(columns, *place);
            }
            ExportRefusal::CannotCarry { place, character } => {
                text_at(names, columns, *place).retain(|kept| kept != *character);
            }
            ExportRefusal::ReadsAsVariantsFile => names.header = fixed_name,
            ExportRefusal::FormatNotBuilt
            | ExportRefusal::NoIndividual
            | ExportRefusal::WrongLength { .. }
            | ExportRefusal::ErrorAsName { .. }
            | ExportRefusal::SpacesAtEnds { .. }
            | ExportRefusal::IntegerTooLarge { .. }
            | ExportRefusal::TextTooLong { .. }
            | ExportRefusal::TooLargeForSheet { .. } => {
                panic!("a refusal a CSV of a generated table does not meet: {refusal:?}")
            }
        }
    }

    /// The type a column of `values` reads back as from a CSV with
    /// `decimal`, by "What reads back": text when every value is missing,
    /// its own when it is not text, and for a text column the narrowest
    /// type of its values.
    fn type_read_back(values: &ColumnValues, decimal: DecimalMark) -> ColumnType {
        let ColumnValues::Text(texts) = values else {
            let is_all_missing = match values {
                ColumnValues::Integer(integers) => integers.iter().all(Option::is_none),
                ColumnValues::Float(floats) => floats.iter().all(Option::is_none),
                ColumnValues::Boolean(booleans) => booleans.iter().all(Option::is_none),
                ColumnValues::Text(_) => unreachable!(),
            };
            return if is_all_missing {
                ColumnType::Text
            } else {
                values.column_type()
            };
        };
        let present: Vec<&str> = texts.iter().flatten().map(String::as_str).collect();
        if present.is_empty() {
            ColumnType::Text
        } else if present.iter().all(|text| parse_integer(text).is_some()) {
            ColumnType::Integer
        } else if present
            .iter()
            .all(|text| parse_float(text, decimal).is_some())
        {
            ColumnType::Float
        } else if present.iter().all(|text| parse_boolean(text).is_some()) {
            ColumnType::Boolean
        } else {
            ColumnType::Text
        }
    }

    /// The import of `bytes` with the choices of the export `csv_export`.
    fn imported(
        bytes: &[u8],
        csv_export: &CsvExport,
    ) -> Result<table_io::Table, table_io::ImportError> {
        import_table(
            bytes,
            &ImportOptions {
                max_bytes: 20_000_000,
                max_cells: 2_000_000,
                text: TextOptions {
                    encoding: Some(match csv_export.encoding {
                        CsvEncoding::Utf8 | CsvEncoding::Utf8WithMark => Encoding::Utf8,
                        CsvEncoding::Windows1252 => Encoding::Windows1252,
                    }),
                    separator: Some(csv_export.separator),
                    decimal: Some(csv_export.decimal),
                },
            },
        )
    }

    #[test]
    fn a_table_exported_as_a_csv_with_each_of_the_36_combinations_reads_back_as_itself() {
        let mut generator = Generator { state: SEED };
        let combinations = combinations();
        assert_eq!(combinations.len(), 36);
        let mut num_exported = 0_u32;
        let mut num_no_individual = 0_u32;
        let mut num_repaired = 0_u32;
        let mut num_narrowed = 0_u32;
        for _ in 0..NUM_TABLES {
            let (generated_names, generated_columns) = generated_table(&mut generator);
            for csv_export in &combinations {
                let format = ExportFormat::Csv(*csv_export);
                let mut names = generated_names.clone();
                let mut columns = generated_columns.clone();
                let mut num_repairs = 0_u32;
                let bytes = loop {
                    match export_table(&names, &columns, &format) {
                        Ok(bytes) => break Some(bytes),
                        Err(ExportError::Refused(ExportRefusal::NoIndividual)) => break None,
                        Err(ExportError::Refused(refusal)) => {
                            num_repairs += 1;
                            assert!(num_repairs <= MAX_REPAIRS, "{refusal:?}");
                            assert_due(&names, &columns, &refusal, csv_export);
                            repaired(&mut names, &mut columns, &refusal, num_repairs);
                        }
                        Err(error) => panic!("{error:?} of {names:?} {columns:?}"),
                    }
                };
                let Some(bytes) = bytes else {
                    assert!(names.names.is_empty());
                    num_no_individual += 1;
                    continue;
                };
                num_exported += 1;
                if num_repairs > 0 {
                    num_repaired += 1;
                }
                let context = format!("{csv_export:?} {names:?} {columns:?} {bytes:02X?}");

                let table = imported(&bytes, csv_export).expect(&context);

                assert_eq!(table.names.header, names.header, "{context}");
                assert_eq!(table.names.names, names.names, "{context}");
                assert_eq!(table.columns.len(), columns.len(), "{context}");
                for (read_column, column) in table.columns.iter().zip(&columns) {
                    assert_eq!(read_column.name, column.name, "{context}");
                    let read_type = type_read_back(&column.values, csv_export.decimal);
                    if read_type != column.values.column_type() {
                        num_narrowed += 1;
                    }
                    assert_eq!(
                        Ok(&read_column.values),
                        convert_column(&column.values, read_type, csv_export.decimal).as_ref(),
                        "{context}"
                    );
                }
            }
        }
        // The counts the report gives, and a check that each kind of table
        // was met.
        assert_eq!(num_exported + num_no_individual, NUM_TABLES * 36);
        assert!(num_no_individual > 0 && num_repaired > 1_000 && num_narrowed > 1_000);
        println!(
            "{NUM_TABLES} tables, {} exports of the 36 combinations: {num_exported} read back, \
             {num_repaired} of them made again after a refusal, {num_no_individual} refused \
             for no individual; {num_narrowed} columns read back as another type",
            NUM_TABLES * 36
        );
    }
}
