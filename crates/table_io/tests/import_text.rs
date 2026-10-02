//! The table `import_table` makes of a text file, a CSV or a TSV, by
//! `docs/specs/text-files.md` and the rules of the rows of
//! `docs/specs/import.md`.
//!
//! Each case is the literal bytes of a text file and the literal table or
//! refusal it gives: the bytes and the encoding of "How it is verified" of
//! `docs/specs/text-files.md`, the 256 bytes of Windows-1252 among them;
//! every row of the table of texts of "How it is verified" of
//! `docs/specs/import.md`, which holds the rows of popnei_web's table at
//! `readCsv`, and the cases of `docs/specs/text-files.md` about the lines,
//! the quotes, the separator and the decimal mark; the property of the
//! round trip of a table written as a CSV; and the owner's text files,
//! which wait for `tests/data/`.

#![cfg(feature = "csv")]

use std::path::PathBuf;

use table_io::{
    Column, ColumnValues, DecimalMark, Encoding, Format, FoundEncoding, HowRead, ImportError,
    ImportOptions, NameColumn, Refusal, Separator, Table, TextOptions, TextRead, import_table,
};

/// The limit of bytes of popnei_web, 20 MB.
const MAX_BYTES: u64 = 20_000_000;

/// `MAX_SHEET_CELLS` of popnei_web, the limit its light worker gives.
const MAX_SHEET_CELLS: u32 = 2_000_000;

/// Every option of a text file found from the file.
const FOUND: TextOptions = TextOptions {
    encoding: None,
    separator: None,
    decimal: None,
};

/// The options of an import with popnei_web's limits and `text`.
fn options_with(text: TextOptions) -> ImportOptions {
    ImportOptions {
        max_bytes: MAX_BYTES,
        max_cells: MAX_SHEET_CELLS,
        text,
    }
}

/// What `import_table` gives for `bytes` with every option found.
fn import(bytes: &[u8]) -> Result<Table, ImportError> {
    import_table(bytes, &options_with(FOUND))
}

/// What `import_table` gives for `bytes` with the options `text`.
fn import_with(bytes: &[u8], text: TextOptions) -> Result<Table, ImportError> {
    import_table(bytes, &options_with(text))
}

/// The options with the separator set and the rest found.
fn separator_set(separator: Separator) -> TextOptions {
    TextOptions {
        encoding: None,
        separator: Some(separator),
        decimal: None,
    }
}

/// The options with the encoding set and the rest found.
fn encoding_set(encoding: Encoding) -> TextOptions {
    TextOptions {
        encoding: Some(encoding),
        separator: None,
        decimal: None,
    }
}

/// The error of a refusal of a text file.
fn refused(refusal: Refusal) -> Result<Table, ImportError> {
    Err(ImportError::Refused {
        format: Format::Text,
        refusal,
    })
}

/// How a text file of UTF-8 with every character decoded was read, with
/// `separator` and `decimal`.
fn utf8(separator: Separator, decimal: DecimalMark) -> HowRead {
    read_as(FoundEncoding::Utf8, separator, decimal, None)
}

/// How a text file was read.
fn read_as(
    encoding: FoundEncoding,
    separator: Separator,
    decimal: DecimalMark,
    undecoded_line: Option<u32>,
) -> HowRead {
    HowRead::Text(TextRead {
        encoding,
        separator,
        decimal,
        undecoded_line,
    })
}

/// A table whose first column, column 1, is headed `header` over `names`,
/// with `columns`, read as `read`.
fn table(header: &str, names: &[&str], columns: Vec<Column>, read: HowRead) -> Table {
    Table {
        names: NameColumn {
            header: header.to_owned(),
            number: 1,
            names: names.iter().map(|&name| name.to_owned()).collect(),
        },
        columns,
        read,
    }
}

/// A column named `name`, its place in the row `number`.
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

/// The names of a table, or the refusal, for the cases that are about
/// the names alone.
fn names_of(import: &Result<Table, ImportError>) -> Result<Vec<&str>, &ImportError> {
    match import {
        Ok(imported) => Ok(imported.names.names.iter().map(String::as_str).collect()),
        Err(error) => Err(error),
    }
}

/// How a table was read, or the refusal.
fn read_of(import: &Result<Table, ImportError>) -> Result<&HowRead, &ImportError> {
    import.as_ref().map(|imported| &imported.read)
}

/// The bytes of `text` in UTF-16, little endian after the mark `FF FE`.
fn utf16_little_endian(text: &str) -> Vec<u8> {
    let mut bytes = vec![0xFF, 0xFE];
    bytes.extend(text.encode_utf16().flat_map(u16::to_le_bytes));
    bytes
}

/// The bytes of `text` in UTF-16, big endian after the mark `FE FF`.
fn utf16_big_endian(text: &str) -> Vec<u8> {
    let mut bytes = vec![0xFE, 0xFF];
    bytes.extend(text.encode_utf16().flat_map(u16::to_be_bytes));
    bytes
}

/// `bytes` after the mark of UTF-8, `EF BB BF`.
fn with_utf8_mark(bytes: &[u8]) -> Vec<u8> {
    let mut marked = vec![0xEF, 0xBB, 0xBF];
    marked.extend_from_slice(bytes);
    marked
}

// The bytes and the encoding, "How it is verified" of
// docs/specs/text-files.md.

/// The Spanish Excel file in Windows-1252, `ó` the byte F3, `ñ` F1, `ú`
/// FA and `é` E9, with `\r\n`.
const SPANISH_WINDOWS_1252: &[u8] = b"Individuo;Poblaci\xF3n;Altura\r\n\
ind_001;Espa\xF1a;1,75\r\n\
ind_002;Espa\xF1a;1,82\r\n\
ind_003;Per\xFA;1,69\r\n\
ind_004;M\xE9xico;1,58\r\n";

/// The same file in UTF-8, as text.
const SPANISH: &str = "Individuo;Población;Altura\r\n\
ind_001;España;1,75\r\n\
ind_002;España;1,82\r\n\
ind_003;Perú;1,69\r\n\
ind_004;México;1,58\r\n";

/// The table of the Spanish file, read as `read`.
fn spanish_table(read: HowRead) -> Table {
    table(
        "Individuo",
        &["ind_001", "ind_002", "ind_003", "ind_004"],
        vec![
            column(
                "Población",
                2,
                texts(&[Some("España"), Some("España"), Some("Perú"), Some("México")]),
            ),
            column(
                "Altura",
                3,
                floats(&[Some(1.75), Some(1.82), Some(1.69), Some(1.58)]),
            ),
        ],
        read,
    )
}

#[test]
fn the_spanish_excel_file_in_windows_1252_is_read_as_windows_1252_with_semicolons_and_the_comma() {
    assert_eq!(
        import(SPANISH_WINDOWS_1252),
        Ok(spanish_table(read_as(
            FoundEncoding::Windows1252,
            Separator::Semicolon,
            DecimalMark::Comma,
            None
        )))
    );
}

#[test]
fn the_spanish_file_in_utf8_with_its_mark_is_read_as_utf8_and_the_same_table() {
    assert_eq!(
        import(&with_utf8_mark(SPANISH.as_bytes())),
        Ok(spanish_table(utf8(
            Separator::Semicolon,
            DecimalMark::Comma
        )))
    );
}

#[test]
fn the_spanish_file_in_utf8_without_a_mark_is_read_as_utf8() {
    assert_eq!(
        import(SPANISH.as_bytes()),
        Ok(spanish_table(utf8(
            Separator::Semicolon,
            DecimalMark::Comma
        )))
    );
}

#[test]
fn the_windows_1252_file_read_with_utf8_set_shows_the_replacement_character_from_line_1() {
    assert_eq!(
        import_with(SPANISH_WINDOWS_1252, encoding_set(Encoding::Utf8)),
        Ok(table(
            "Individuo",
            &["ind_001", "ind_002", "ind_003", "ind_004"],
            vec![
                column(
                    "Poblaci\u{FFFD}n",
                    2,
                    texts(&[
                        Some("Espa\u{FFFD}a"),
                        Some("Espa\u{FFFD}a"),
                        Some("Per\u{FFFD}"),
                        Some("M\u{FFFD}xico"),
                    ]),
                ),
                column(
                    "Altura",
                    3,
                    floats(&[Some(1.75), Some(1.82), Some(1.69), Some(1.58)]),
                ),
            ],
            read_as(
                FoundEncoding::Utf8,
                Separator::Semicolon,
                DecimalMark::Comma,
                Some(1)
            ),
        ))
    );
}

#[test]
fn the_spanish_file_in_utf16_little_endian_is_read_as_utf16_and_the_same_table() {
    assert_eq!(
        import(&utf16_little_endian(SPANISH)),
        Ok(spanish_table(read_as(
            FoundEncoding::Utf16,
            Separator::Semicolon,
            DecimalMark::Comma,
            None
        )))
    );
}

#[test]
fn the_spanish_file_in_utf16_big_endian_is_read_as_utf16_and_the_same_table() {
    assert_eq!(
        import(&utf16_big_endian(SPANISH)),
        Ok(spanish_table(read_as(
            FoundEncoding::Utf16,
            Separator::Semicolon,
            DecimalMark::Comma,
            None
        )))
    );
}

#[test]
fn a_utf16_file_is_read_as_utf16_with_windows_1252_set() {
    for bytes in [utf16_little_endian(SPANISH), utf16_big_endian(SPANISH)] {
        assert_eq!(
            import_with(&bytes, encoding_set(Encoding::Windows1252)),
            Ok(spanish_table(read_as(
                FoundEncoding::Utf16,
                Separator::Semicolon,
                DecimalMark::Comma,
                None
            )))
        );
    }
}

#[test]
fn a_utf16_file_with_one_byte_more_is_cut_short() {
    for mut bytes in [utf16_little_endian(SPANISH), utf16_big_endian(SPANISH)] {
        bytes.push(b'x');
        assert_eq!(import(&bytes), refused(Refusal::CutShort));
    }
}

#[test]
fn a_utf16_file_whose_last_character_is_the_first_half_of_one_written_in_four_is_cut_short() {
    let mut little_endian = utf16_little_endian(SPANISH);
    little_endian.extend_from_slice(&[0x3D, 0xD8]);
    let mut big_endian = utf16_big_endian(SPANISH);
    big_endian.extend_from_slice(&[0xD8, 0x3D]);

    assert_eq!(import(&little_endian), refused(Refusal::CutShort));
    assert_eq!(import(&big_endian), refused(Refusal::CutShort));
}

#[test]
fn a_half_of_a_character_in_the_middle_of_a_utf16_file_is_the_replacement_character_and_its_line() {
    // The first half of 😀, D83D, alone before the `;` of line 2.
    let mut bytes = utf16_little_endian("id;pop\r\nA");
    bytes.extend_from_slice(&[0x3D, 0xD8]);
    bytes.extend_from_slice(&utf16_little_endian(";P1\r\n")[2..]);

    assert_eq!(
        import(&bytes),
        Ok(table(
            "id",
            &["A\u{FFFD}"],
            vec![column("pop", 2, texts(&[Some("P1")]))],
            read_as(
                FoundEncoding::Utf16,
                Separator::Semicolon,
                DecimalMark::Point,
                Some(2)
            ),
        ))
    );
}

#[test]
fn a_utf16_file_of_its_mark_alone_is_empty() {
    assert_eq!(import(&[0xFF, 0xFE]), refused(Refusal::Empty));
    assert_eq!(import(&[0xFE, 0xFF]), refused(Refusal::Empty));
}

#[test]
fn a_text_with_one_byte_0_in_its_last_line_is_not_text() {
    assert_eq!(
        import(b"id,pop\nA,P1\nB,P\x002\n"),
        refused(Refusal::NotText)
    );
}

#[test]
fn the_bytes_00_01_02_03_are_not_text() {
    assert_eq!(import(&[0x00, 0x01, 0x02, 0x03]), refused(Refusal::NotText));
}

#[test]
fn a_byte_0_after_the_mark_of_utf8_is_not_text_whatever_the_encoding_set() {
    let bytes = with_utf8_mark(b"\x00id,pop\nA,P1\n");
    assert_eq!(import(&bytes), refused(Refusal::NotText));
    assert_eq!(
        import_with(&bytes, encoding_set(Encoding::Windows1252)),
        refused(Refusal::NotText)
    );
}

#[test]
fn a_bad_byte_in_line_3_of_a_utf8_file_with_its_mark_is_the_replacement_character_at_line_3() {
    let damaged = SPANISH.replacen("Espa\u{f1}a;1,82", "Espa\u{f1}a;1,8", 1);
    let mut bytes = with_utf8_mark(damaged.as_bytes());
    // The 2 of 1,82 on line 3, written as the byte FF, which UTF-8 never
    // holds.
    let place = bytes
        .windows(4)
        .position(|window| window == b"1,8\r")
        .unwrap();
    bytes.insert(place + 3, 0xFF);

    let import = import(&bytes);

    assert_eq!(
        read_of(&import),
        Ok(&read_as(
            FoundEncoding::Utf8,
            Separator::Semicolon,
            DecimalMark::Comma,
            Some(3)
        ))
    );
    assert_eq!(
        import.unwrap().columns[1].values,
        texts(&[
            Some("1,75"),
            Some("1,8\u{FFFD}"),
            Some("1,69"),
            Some("1,58")
        ])
    );
}

#[test]
fn the_utf8_file_with_its_mark_and_no_bad_byte_has_no_line_not_decoded() {
    assert_eq!(
        read_of(&import(&with_utf8_mark(SPANISH.as_bytes()))),
        Ok(&utf8(Separator::Semicolon, DecimalMark::Comma))
    );
}

#[test]
fn a_replacement_character_the_file_holds_itself_is_reported_with_its_line() {
    let import = import("id,pop\nA,P1\nB,Espa\u{FFFD}a\n".as_bytes());

    assert_eq!(
        read_of(&import),
        Ok(&read_as(
            FoundEncoding::Utf8,
            Separator::Comma,
            DecimalMark::Point,
            Some(3)
        ))
    );
}

#[test]
fn the_first_three_bytes_of_an_emoji_before_b_with_utf8_set_are_one_replacement_character_and_b() {
    assert_eq!(
        names_of(&import_with(
            b"id\nA\xF0\x9F\x98b\n",
            encoding_set(Encoding::Utf8)
        )),
        Ok(vec!["A\u{FFFD}b"])
    );
}

#[test]
fn a_text_of_ascii_alone_is_read_as_utf8() {
    assert_eq!(
        read_of(&import(b"id,pop\nA,P1\n")),
        Ok(&utf8(Separator::Comma, DecimalMark::Point))
    );
}

#[test]
fn the_mark_of_utf8_is_removed_with_windows_1252_set() {
    assert_eq!(
        import_with(
            &with_utf8_mark(b"id,pop\nA,Espa\xF1a\n"),
            encoding_set(Encoding::Windows1252)
        ),
        Ok(table(
            "id",
            &["A"],
            vec![column("pop", 2, texts(&[Some("España")]))],
            read_as(
                FoundEncoding::Windows1252,
                Separator::Comma,
                DecimalMark::Point,
                None
            ),
        ))
    );
}

#[test]
fn every_mark_at_the_start_of_the_text_is_removed() {
    let text = "\u{FEFF}\u{FEFF}\u{FEFF}\u{FEFF}id,pop\nA,P1\n";
    for bytes in [
        text.as_bytes().to_vec(),
        with_utf8_mark(text.as_bytes()),
        utf16_little_endian(text),
    ] {
        assert_eq!(
            import(&bytes).map(|imported| imported.names.header),
            Ok("id".to_owned())
        );
    }
}

#[test]
fn accents_a_character_outside_the_first_plane_an_emoji_and_a_line_break_are_read_in_each_encoding()
{
    let text = "id,pop\nPoblación,\"𝔸 😀\r\nx\"\n";
    let expected_names = Ok(vec!["Población"]);
    for bytes in [
        text.as_bytes().to_vec(),
        with_utf8_mark(text.as_bytes()),
        utf16_little_endian(text),
        utf16_big_endian(text),
    ] {
        let import = import(&bytes);
        assert_eq!(names_of(&import), expected_names);
        assert_eq!(
            import.unwrap().columns,
            vec![column("pop", 2, texts(&[Some("𝔸 😀\r\nx")]))]
        );
    }
}

/// The 32 characters of Windows-1252 for the bytes `80` to `9F`, the
/// table of `docs/specs/text-files.md`, "The bytes and the encoding".
const WINDOWS_1252_80_TO_9F: [char; 32] = [
    '\u{20AC}', '\u{0081}', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\u{008D}', '\u{017D}', '\u{008F}',
    '\u{0090}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}', '\u{0153}', '\u{009D}', '\u{017E}', '\u{0178}',
];

#[test]
fn each_of_the_256_bytes_between_a_and_b_read_as_windows_1252_is_its_character() {
    let options = TextOptions {
        encoding: Some(Encoding::Windows1252),
        separator: Some(Separator::Semicolon),
        decimal: None,
    };
    let read = read_as(
        FoundEncoding::Windows1252,
        Separator::Semicolon,
        DecimalMark::Point,
        None,
    );
    for byte in 0..=u8::MAX {
        let bytes = [b'h', b'\r', b'\n', b'a', byte, b'b', b'\r', b'\n'];
        let expected = match byte {
            0x00 => refused(Refusal::NotText),
            0x0A | 0x0D => Ok(table("h", &["a", "b"], vec![], read.clone())),
            0x3B => refused(Refusal::RaggedRow {
                line: 2,
                expected: 1,
                found: 2,
                separator: Separator::Semicolon,
            }),
            0x80..=0x9F => {
                let character = WINDOWS_1252_80_TO_9F[usize::from(byte - 0x80)];
                Ok(table(
                    "h",
                    &[&format!("a{character}b")],
                    vec![],
                    read.clone(),
                ))
            }
            _ => {
                let character = char::from(byte);
                Ok(table(
                    "h",
                    &[&format!("a{character}b")],
                    vec![],
                    read.clone(),
                ))
            }
        };
        assert_eq!(
            import_with(&bytes, options),
            expected,
            "the byte {byte:02X}"
        );
    }
}

// The table of texts of "How it is verified" of docs/specs/import.md, row
// by row.

#[test]
fn a_csv_of_commas_is_read_with_the_comma_and_the_point_and_pop_is_text() {
    assert_eq!(
        import(b"id,pop\nA,P1\nB,P2\nC,P1\n"),
        Ok(table(
            "id",
            &["A", "B", "C"],
            vec![column(
                "pop",
                2,
                texts(&[Some("P1"), Some("P2"), Some("P1")])
            )],
            utf8(Separator::Comma, DecimalMark::Point),
        ))
    );
}

#[test]
fn a_file_of_tabs_is_read_with_the_tab() {
    assert_eq!(
        import(b"id\tpop\nA\tP1\n"),
        Ok(table(
            "id",
            &["A"],
            vec![column("pop", 2, texts(&[Some("P1")]))],
            utf8(Separator::Tab, DecimalMark::Point),
        ))
    );
}

#[test]
fn a_file_of_semicolons_with_decimal_commas_is_read_with_the_comma_and_h_is_float() {
    assert_eq!(
        import(b"id;h\nA;1,5\nB;1,7\nC;1,9\n"),
        Ok(table(
            "id",
            &["A", "B", "C"],
            vec![column("h", 2, floats(&[Some(1.5), Some(1.7), Some(1.9)]))],
            utf8(Separator::Semicolon, DecimalMark::Comma),
        ))
    );
}

#[test]
fn a_file_that_mixes_one_point_with_two_commas_takes_the_comma_and_h_is_text() {
    assert_eq!(
        import(b"id;h\nA;1.5\nB;1,7\nC;1,9\n"),
        Ok(table(
            "id",
            &["A", "B", "C"],
            vec![column(
                "h",
                2,
                texts(&[Some("1.5"), Some("1,7"), Some("1,9")])
            )],
            utf8(Separator::Semicolon, DecimalMark::Comma),
        ))
    );
}

#[test]
fn the_names_001_002_and_003_stay_as_written_and_x_is_integer() {
    assert_eq!(
        import(b"id,x\n001,1\n002,2\n003,3\n"),
        Ok(table(
            "id",
            &["001", "002", "003"],
            vec![column("x", 2, integers(&[Some(1), Some(2), Some(3)]))],
            utf8(Separator::Comma, DecimalMark::Point),
        ))
    );
}

#[test]
fn an_empty_cell_and_na_are_missing() {
    assert_eq!(
        import(b"id,st\nA,case\nB,control\nC,\nD,NA\n"),
        Ok(table(
            "id",
            &["A", "B", "C", "D"],
            vec![column(
                "st",
                2,
                texts(&[Some("case"), Some("control"), None, None])
            )],
            utf8(Separator::Comma, DecimalMark::Point),
        ))
    );
}

#[test]
fn the_cells_1_01_and_001_are_the_integer_1() {
    assert_eq!(
        import(b"id,s\nA,1\nB,01\nC,001\n"),
        Ok(table(
            "id",
            &["A", "B", "C"],
            vec![column("s", 2, integers(&[Some(1), Some(1), Some(1)]))],
            utf8(Separator::Comma, DecimalMark::Point),
        ))
    );
}

#[test]
fn a_quoted_cell_holds_the_separator_and_a_doubled_quote() {
    assert_eq!(
        import(b"id,n\nA,\"x, y\"\nB,\"say \"\"hi\"\"\"\n"),
        Ok(table(
            "id",
            &["A", "B"],
            vec![column("n", 2, texts(&[Some("x, y"), Some("say \"hi\"")]))],
            utf8(Separator::Comma, DecimalMark::Point),
        ))
    );
}

#[test]
fn the_line_endings_crlf_and_cr_give_the_table_of_lf_the_blank_line_skipped() {
    let expected = Ok(table(
        "id",
        &["A", "B"],
        vec![column("n", 2, integers(&[Some(1), Some(2)]))],
        utf8(Separator::Comma, DecimalMark::Point),
    ));
    assert_eq!(import(b"id,n\nA,1\nB,2\n"), expected);
    assert_eq!(import(b"id,n\r\nA,1\r\n\r\nB,2\r\n"), expected);
    assert_eq!(import(b"id,n\rA,1\r\rB,2\r"), expected);
}

#[test]
fn the_empty_columns_of_trailing_separators_are_dropped() {
    assert_eq!(
        import(b"id;pop;;\nA;P1;;\n"),
        Ok(table(
            "id",
            &["A"],
            vec![column("pop", 2, texts(&[Some("P1")]))],
            utf8(Separator::Semicolon, DecimalMark::Point),
        ))
    );
}

#[test]
fn the_empty_cells_at_the_end_of_the_header_are_dropped_and_a_short_row_fits() {
    assert_eq!(
        import(b"id;pop;;\nA;P1\nB;P2;NA\n"),
        Ok(table(
            "id",
            &["A", "B"],
            vec![column("pop", 2, texts(&[Some("P1"), Some("P2")]))],
            utf8(Separator::Semicolon, DecimalMark::Point),
        ))
    );
}

#[test]
fn na_below_an_empty_cell_of_the_header_is_no_value_so_the_semicolon_fits_and_wins_the_tie() {
    assert_eq!(
        import(b"id,x;pop;;\nA,1;P1\nB,2;P2;NA\n"),
        Ok(table(
            "id,x",
            &["A,1", "B,2"],
            vec![column("pop", 2, texts(&[Some("P1"), Some("P2")]))],
            utf8(Separator::Semicolon, DecimalMark::Point),
        ))
    );
}

#[test]
fn a_value_after_the_run_of_empty_cells_of_the_header_is_an_unnamed_column_4() {
    assert_eq!(
        import(b"id;pop;;\nA;P1\nB;P2;;x\n"),
        refused(Refusal::UnnamedColumn { column: 4 })
    );
}

#[test]
fn a_value_in_the_run_of_empty_cells_of_the_header_is_an_unnamed_column_and_not_a_ragged_row() {
    assert_eq!(
        import(b"id;pop;;\na;1\nb;2;3\n"),
        refused(Refusal::UnnamedColumn { column: 3 })
    );
}

#[test]
fn a_short_row_under_a_header_that_ends_in_a_name_is_a_ragged_row() {
    assert_eq!(
        import(b"id;pop;;x\nA;P1;;1\nB;P2\n"),
        refused(Refusal::RaggedRow {
            line: 3,
            expected: 4,
            found: 2,
            separator: Separator::Semicolon,
        })
    );
}

#[test]
fn a_column_with_no_name_whose_cells_are_na_and_a_dash_is_dropped() {
    assert_eq!(
        import(b"id,,pop\nA,NA,P1\nB,-,P2\n"),
        Ok(table(
            "id",
            &["A", "B"],
            vec![column("pop", 3, texts(&[Some("P1"), Some("P2")]))],
            utf8(Separator::Comma, DecimalMark::Point),
        ))
    );
}

#[test]
fn a_text_whose_first_line_starts_as_a_vcf_is_a_variants_file() {
    for text in [
        "##fileformat=VCFv4.2\n#CHROM\tPOS\n",
        "#CHROM\tPOS\tID\n1\t10\tx\n",
        "\n##fileformat=VCFv4.2\n#CHROM\tPOS\n",
        " \t\r\n#CHROM\tPOS\n",
        "\u{FEFF}\r\n\r\n##fileformat=VCFv4.2\n",
    ] {
        assert_eq!(
            import(text.as_bytes()),
            refused(Refusal::VariantsFile),
            "{text:?}"
        );
    }
    assert_eq!(
        import_with(
            "\u{FEFF}##fileformat=VCFv4.3\n".as_bytes(),
            separator_set(Separator::Semicolon)
        ),
        refused(Refusal::VariantsFile)
    );
}

#[test]
fn a_row_shorter_than_the_header_is_a_ragged_row_on_its_line() {
    assert_eq!(
        import(b"id,pop\nA,P1\nB\n"),
        refused(Refusal::RaggedRow {
            line: 3,
            expected: 2,
            found: 1,
            separator: Separator::Comma,
        })
    );
}

#[test]
fn a_row_with_no_name_is_an_empty_individual_on_its_line() {
    assert_eq!(
        import(b"id,pop\n,P1\n"),
        refused(Refusal::EmptyIndividual { row: 2 })
    );
}

#[test]
fn an_individual_in_two_rows_is_a_duplicate_individual_with_its_two_lines() {
    assert_eq!(
        import(b"id,pop\nA,P1\nA,P2\n"),
        refused(Refusal::DuplicateIndividual {
            name: "A".to_owned(),
            first_row: 2,
            second_row: 3,
        })
    );
}

#[test]
fn two_columns_of_one_name_are_a_duplicate_column_with_their_places() {
    assert_eq!(
        import(b"id,pop,pop\nA,1,2\n"),
        refused(Refusal::DuplicateColumn {
            name: "pop".to_owned(),
            first_column: 2,
            second_column: 3,
        })
    );
}

#[test]
fn names_are_compared_exactly_so_pop_and_pop_in_capitals_are_two_columns() {
    assert_eq!(
        import(b"id,Pop,pop\nA,1,2\n"),
        Ok(table(
            "id",
            &["A"],
            vec![
                column("Pop", 2, integers(&[Some(1)])),
                column("pop", 3, integers(&[Some(2)])),
            ],
            utf8(Separator::Comma, DecimalMark::Point),
        ))
    );
}

#[test]
fn a_column_with_a_value_and_no_name_is_an_unnamed_column_by_its_place() {
    assert_eq!(
        import(b"id,,pop\nA,1,P1\n"),
        refused(Refusal::UnnamedColumn { column: 2 })
    );
}

#[test]
fn a_quote_never_closed_is_an_unclosed_quote_at_the_line_of_its_cell() {
    assert_eq!(
        import_with(b"id,pop\nA,\"P1\nB,P2\n", separator_set(Separator::Comma)),
        refused(Refusal::UnclosedQuote {
            line: 2,
            separator: Separator::Comma,
        })
    );
}

#[test]
fn a_header_alone_and_the_empty_text_are_empty() {
    assert_eq!(import(b"id,pop\n"), refused(Refusal::Empty));
    assert_eq!(import(b""), refused(Refusal::Empty));
}

#[test]
fn the_mark_at_the_start_of_the_text_is_not_part_of_the_first_name() {
    assert_eq!(
        import("\u{FEFF}id,pop\nA,P1\n".as_bytes()).map(|imported| imported.names.header),
        Ok("id".to_owned())
    );
}

#[test]
fn when_the_tab_and_the_semicolon_both_fit_with_two_cells_the_tab_is_taken() {
    assert_eq!(
        import(b"id;n\tx\nA;1\t2\n"),
        Ok(table(
            "id;n",
            &["A;1"],
            vec![column("x", 2, integers(&[Some(2)]))],
            utf8(Separator::Tab, DecimalMark::Point),
        ))
    );
}

#[test]
fn when_no_separator_fits_the_one_of_the_most_cells_in_the_header_is_used_and_the_row_refused() {
    assert_eq!(
        import(b"id,pop\nA,P1\nB,P2,P3\n"),
        refused(Refusal::RaggedRow {
            line: 3,
            expected: 2,
            found: 3,
            separator: Separator::Comma,
        })
    );
}

#[test]
fn the_line_of_a_row_counts_the_lines_inside_a_quoted_cell_above_it() {
    assert_eq!(
        import(b"id,n\nA,\"x\ny\"\nB,1,2\n"),
        refused(Refusal::RaggedRow {
            line: 4,
            expected: 2,
            found: 3,
            separator: Separator::Comma,
        })
    );
}

#[test]
fn a_file_of_one_column_is_read_with_the_comma_as_the_names_alone() {
    assert_eq!(
        import(b"only\nA\nB\n"),
        Ok(table(
            "only",
            &["A", "B"],
            vec![],
            utf8(Separator::Comma, DecimalMark::Point),
        ))
    );
}

#[test]
fn a_text_file_of_more_cells_than_the_limit_of_cells_is_read_since_its_bytes_bound_it() {
    let options = ImportOptions {
        max_bytes: MAX_BYTES,
        max_cells: 7,
        text: FOUND,
    };
    assert_eq!(
        import_table(b"id,x\nA,1\nB,2\nC,3\n", &options),
        Ok(table(
            "id",
            &["A", "B", "C"],
            vec![column("x", 2, integers(&[Some(1), Some(2), Some(3)]))],
            utf8(Separator::Comma, DecimalMark::Point),
        ))
    );
}

// The rest of popnei_web's table at readCsv, and the cases of the lines,
// the quotes, the separator and the decimal mark of
// docs/specs/text-files.md.

#[test]
fn what_follows_a_closing_quote_is_part_of_the_cell_and_a_quote_inside_a_cell_is_a_character() {
    assert_eq!(
        import(b"id,n\nA,\"x\"y\nB,a\"b\n").map(|imported| imported.columns),
        Ok(vec![column("n", 2, texts(&[Some("xy"), Some("a\"b")]))])
    );
}

#[test]
fn a_doubled_quote_after_a_cell_without_quotes_gives_the_cells_a_and_b_quote_c() {
    assert_eq!(
        import(b"id,n\na,\"b\"\"c\"\n"),
        Ok(table(
            "id",
            &["a"],
            vec![column("n", 2, texts(&[Some("b\"c")]))],
            utf8(Separator::Comma, DecimalMark::Point),
        ))
    );
}

#[test]
fn a_text_with_no_line_break_at_its_end_and_one_ending_in_crlf_give_the_same_table() {
    assert_eq!(
        import(b"id,n\r\nA,x\r\nB,y"),
        import(b"id,n\r\nA,x\r\nB,y\r\n")
    );
    assert_eq!(names_of(&import(b"id,n\r\nA,x\r\nB,y")), Ok(vec!["A", "B"]));
}

#[test]
fn an_unclosed_quote_on_line_5_after_a_quoted_cell_over_lines_2_and_3_is_at_line_5() {
    assert_eq!(
        import(b"id,n\nA,\"x\ny\"\nB,1\nC,\"z\nD,2\n"),
        refused(Refusal::UnclosedQuote {
            line: 5,
            separator: Separator::Comma,
        })
    );
}

#[test]
fn a_crlf_is_one_line_so_a_ragged_row_after_two_is_on_line_3() {
    assert_eq!(
        import(b"id,n\r\nA,1\r\nB\r\n"),
        refused(Refusal::RaggedRow {
            line: 3,
            expected: 2,
            found: 1,
            separator: Separator::Comma,
        })
    );
    assert_eq!(
        import(b"id,n\rA,1\rB\r"),
        refused(Refusal::RaggedRow {
            line: 3,
            expected: 2,
            found: 1,
            separator: Separator::Comma,
        })
    );
    assert_eq!(
        import(b"id,n\r\nA,\"x\r\ny\"\r\nB,1,2\r\n"),
        refused(Refusal::RaggedRow {
            line: 4,
            expected: 2,
            found: 3,
            separator: Separator::Comma,
        })
    );
}

#[test]
fn a_row_named_na_whose_other_cells_are_missing_is_kept_with_na_a_name() {
    assert_eq!(
        import(b"id,pop\nNA,NA\nB,P1\n").map(|imported| (imported.names.names, imported.columns)),
        Ok((
            vec!["NA".to_owned(), "B".to_owned()],
            vec![column("pop", 2, texts(&[None, Some("P1")]))]
        ))
    );
}

#[test]
fn spaces_at_the_ends_of_a_cell_are_removed_and_inside_quotes_kept() {
    assert_eq!(
        import(b"id,pop\nind_01, pop1 \nind_02, \" p 2 \"\n").map(|imported| imported.columns),
        Ok(vec![column(
            "pop",
            2,
            texts(&[Some("pop1"), Some(" p 2 ")])
        )])
    );
}

#[test]
fn the_spaces_before_a_quoted_cell_are_removed_first_so_a_space_quote_cell_is_one_cell() {
    assert_eq!(
        import(b"id,n\nA, \"x, y\"\n").map(|imported| imported.columns),
        Ok(vec![column("n", 2, texts(&[Some("x, y")]))])
    );
    assert_eq!(
        import_with(b"id,n\nA,\t\"x, y\"\n", separator_set(Separator::Comma))
            .map(|imported| imported.columns),
        Ok(vec![column("n", 2, texts(&[Some("x, y")]))])
    );
}

#[test]
fn tabs_at_the_ends_of_a_cell_are_removed_when_the_separator_is_not_a_tab() {
    assert_eq!(
        import_with(b"id;pop\nA;\tP1\t\n", separator_set(Separator::Semicolon))
            .map(|imported| imported.columns),
        Ok(vec![column("pop", 2, texts(&[Some("P1")]))])
    );
}

#[test]
fn with_the_tab_as_separator_spaces_are_removed_and_tabs_split() {
    assert_eq!(
        import(b"id\tpop\n A \t P1 \t\n").map(|imported| (imported.names.names, imported.columns)),
        Ok((
            vec!["A".to_owned()],
            vec![column("pop", 2, texts(&[Some("P1")]))]
        ))
    );
}

#[test]
fn the_spaces_after_a_closing_quote_are_removed() {
    assert_eq!(
        import(b"id,n\nA,\"x\" \n").map(|imported| imported.columns),
        Ok(vec![column("n", 2, texts(&[Some("x")]))])
    );
}

#[test]
fn a_quoted_cell_keeps_its_line_break() {
    assert_eq!(
        import(b"id,n\nA,\"x\r\ny\"\n").map(|imported| imported.columns),
        Ok(vec![column("n", 2, texts(&[Some("x\r\ny")]))])
    );
}

#[test]
fn na_and_a_dash_are_missing_quoted_or_not_and_na_in_lower_case_n_slash_a_and_nan_are_text() {
    assert_eq!(
        import(b"id,v\nA,\"NA\"\nB,-\nC,na\nD,N/A\nE,NaN\n").map(|imported| imported.columns),
        Ok(vec![column(
            "v",
            2,
            texts(&[None, None, Some("na"), Some("N/A"), Some("NaN")])
        )])
    );
}

#[test]
fn na_and_a_dash_in_the_first_column_are_names_and_na_in_the_header_a_name() {
    assert_eq!(
        import(b"NA,pop\nNA,P1\n-,P2\n"),
        Ok(table(
            "NA",
            &["NA", "-"],
            vec![column("pop", 2, texts(&[Some("P1"), Some("P2")]))],
            utf8(Separator::Comma, DecimalMark::Point),
        ))
    );
}

#[test]
fn the_first_column_is_kept_with_an_empty_name() {
    assert_eq!(
        import(b",pop\nA,P1\n").map(|imported| imported.names.header),
        Ok(String::new())
    );
}

#[test]
fn a_row_of_empty_cells_is_skipped_wherever_it_is_above_the_header_too() {
    assert_eq!(
        import(b";;\nid;pop\n;;\nA;P1\n\n"),
        Ok(table(
            "id",
            &["A"],
            vec![column("pop", 2, texts(&[Some("P1")]))],
            utf8(Separator::Semicolon, DecimalMark::Point),
        ))
    );
}

#[test]
fn a_row_with_more_cells_all_empty_past_the_header_is_read() {
    assert_eq!(
        import(b"id,pop\nA,P1,,\n").map(|imported| imported.columns),
        Ok(vec![column("pop", 2, texts(&[Some("P1")]))])
    );
}

#[test]
fn a_spanish_file_of_semicolons_is_found_and_the_comma_set_refuses_its_first_row_with_a_decimal() {
    let text = "Individuo;Población;Altura\nind_001;España;1,75\nind_002;Italia;1,82\n";
    assert_eq!(
        import(text.as_bytes()),
        Ok(table(
            "Individuo",
            &["ind_001", "ind_002"],
            vec![
                column("Población", 2, texts(&[Some("España"), Some("Italia")])),
                column("Altura", 3, floats(&[Some(1.75), Some(1.82)])),
            ],
            utf8(Separator::Semicolon, DecimalMark::Comma),
        ))
    );
    assert_eq!(
        import_with(text.as_bytes(), separator_set(Separator::Comma)),
        refused(Refusal::RaggedRow {
            line: 2,
            expected: 1,
            found: 2,
            separator: Separator::Comma,
        })
    );
}

#[test]
fn the_comma_set_on_a_file_of_semicolons_with_no_comma_reads_one_column() {
    assert_eq!(
        import_with(b"id;pop\nA;P1\nB;P2\n", separator_set(Separator::Comma)),
        Ok(table(
            "id;pop",
            &["A;P1", "B;P2"],
            vec![],
            utf8(Separator::Comma, DecimalMark::Point),
        ))
    );
}

#[test]
fn a_decimal_comma_set_with_the_comma_as_separator_reads_quoted_numbers() {
    let options = TextOptions {
        encoding: None,
        separator: Some(Separator::Comma),
        decimal: Some(DecimalMark::Comma),
    };
    assert_eq!(
        import_with(b"id,h\nA,\"1,5\"\nB,\"1,7\"\nC,\"1,9\"\n", options),
        Ok(table(
            "id",
            &["A", "B", "C"],
            vec![column("h", 2, floats(&[Some(1.5), Some(1.7), Some(1.9)]))],
            utf8(Separator::Comma, DecimalMark::Comma),
        ))
    );
}

#[test]
fn with_the_comma_as_separator_the_decimal_mark_found_is_the_point() {
    assert_eq!(
        import(b"id,h\nA,\"1,5\"\nB,\"1,7\"\nC,\"1,9\"\n"),
        Ok(table(
            "id",
            &["A", "B", "C"],
            vec![column(
                "h",
                2,
                texts(&[Some("1,5"), Some("1,7"), Some("1,9")])
            )],
            utf8(Separator::Comma, DecimalMark::Point),
        ))
    );
}

#[test]
fn a_decimal_mark_set_is_used_whatever_the_values() {
    let options = TextOptions {
        encoding: None,
        separator: None,
        decimal: Some(DecimalMark::Point),
    };
    assert_eq!(
        import_with(b"id;h\nA;1,5\nB;1,7\n", options),
        Ok(table(
            "id",
            &["A", "B"],
            vec![column("h", 2, texts(&[Some("1,5"), Some("1,7")]))],
            utf8(Separator::Semicolon, DecimalMark::Point),
        ))
    );
}

#[test]
fn a_file_of_whole_numbers_is_read_with_the_point() {
    assert_eq!(
        read_of(&import(b"id;s\nA;1\nB;2\nC;3\n")),
        Ok(&utf8(Separator::Semicolon, DecimalMark::Point))
    );
}

#[test]
fn as_many_numbers_with_a_comma_as_with_a_point_is_the_point() {
    assert_eq!(
        read_of(&import(b"id;h\nA;1,5\nB;1.5\n")),
        Ok(&utf8(Separator::Semicolon, DecimalMark::Point))
    );
}

#[test]
fn whole_numbers_are_not_numbers_written_with_a_point_so_the_comma_is_found() {
    let import = import(b"id;h;n\nA;1,5;3\nB;1,7;4\nC;1,9;5");
    assert_eq!(
        read_of(&import),
        Ok(&utf8(Separator::Semicolon, DecimalMark::Comma))
    );
    assert_eq!(
        import.unwrap().columns[0].values,
        floats(&[Some(1.5), Some(1.7), Some(1.9)])
    );
}

#[test]
fn the_first_column_is_not_counted_for_the_decimal_mark() {
    assert_eq!(
        read_of(&import(b"id;h\n1,1;1.5\n1,2;1.7\n1,3;x")),
        Ok(&utf8(Separator::Semicolon, DecimalMark::Point))
    );
}

#[test]
fn the_order_of_the_refusals_a_ragged_row_before_an_unnamed_column() {
    assert_eq!(
        import(b"id,,pop\nA,1,P1\nB,2\n"),
        refused(Refusal::RaggedRow {
            line: 3,
            expected: 3,
            found: 2,
            separator: Separator::Comma,
        })
    );
}

#[test]
fn the_order_of_the_refusals_an_unnamed_column_before_a_duplicate_one() {
    assert_eq!(
        import(b"id,pop,pop,\nA,1,2,3\n"),
        refused(Refusal::UnnamedColumn { column: 4 })
    );
}

#[test]
fn the_order_of_the_refusals_the_rows_in_the_order_of_the_file() {
    assert_eq!(
        import(b"id,pop\nA,P1\nA,P2\n,P3\n"),
        refused(Refusal::DuplicateIndividual {
            name: "A".to_owned(),
            first_row: 2,
            second_row: 3,
        })
    );
}

#[test]
fn the_order_of_the_refusals_a_duplicate_column_before_a_duplicate_individual() {
    assert_eq!(
        import(b"id,pop,pop\nA,P1,1\nA,P2,2"),
        refused(Refusal::DuplicateColumn {
            name: "pop".to_owned(),
            first_column: 2,
            second_column: 3,
        })
    );
}

#[test]
fn the_order_of_the_refusals_an_unclosed_quote_before_empty() {
    assert_eq!(
        import_with(b"id,\"pop\nA,P1", separator_set(Separator::Comma)),
        refused(Refusal::UnclosedQuote {
            line: 1,
            separator: Separator::Comma,
        })
    );
}

#[test]
fn the_order_of_the_refusals_a_variants_file_before_an_unclosed_quote() {
    assert_eq!(
        import(b"##fileformat=VCFv4.2\n\"x\n"),
        refused(Refusal::VariantsFile)
    );
}

#[test]
fn the_order_of_the_refusals_not_text_before_a_variants_file() {
    assert_eq!(
        import(b"##fileformat=VCFv4.2\n\x00\n"),
        refused(Refusal::NotText)
    );
}

#[test]
fn an_unclosed_quote_found_with_no_separator_set_names_the_separator_of_the_most_cells() {
    assert_eq!(
        import(b"id;pop\nA;\"P1\nB;P2\n"),
        refused(Refusal::UnclosedQuote {
            line: 2,
            separator: Separator::Semicolon,
        })
    );
}

#[test]
fn a_title_line_of_three_cells_over_the_header_is_an_unnamed_column_2() {
    assert_eq!(
        import(b"Tabla 1;;\nid;pop;h\nA;P1;1,5\n"),
        refused(Refusal::UnnamedColumn { column: 2 })
    );
}

#[test]
fn a_title_line_of_one_cell_is_read_with_the_comma_and_refused_at_its_first_comma_or_one_column() {
    assert_eq!(
        import(b"Tabla 1\nid;pop;h\nA;P1;1,5\n"),
        refused(Refusal::RaggedRow {
            line: 3,
            expected: 1,
            found: 2,
            separator: Separator::Comma,
        })
    );
    assert_eq!(
        import(b"Tabla 1\nid;pop\nA;P1\n"),
        Ok(table(
            "Tabla 1",
            &["id;pop", "A;P1"],
            vec![],
            utf8(Separator::Comma, DecimalMark::Point),
        ))
    );
}

#[test]
fn a_vcf_cut_at_a_line_of_genotypes_is_read_as_a_table_and_refused() {
    assert_eq!(
        import(b"1\t10\t.\tA\tT\t.\tPASS\t.\tGT\t0/1\n1\t20\t.\tG\tC\t.\tPASS\t.\tGT\t1/1\n"),
        refused(Refusal::DuplicateColumn {
            name: ".".to_owned(),
            first_column: 3,
            second_column: 6,
        })
    );
}

#[test]
fn a_line_of_a_vcf_below_the_first_is_a_cell_like_any_other() {
    assert_eq!(
        names_of(&import(b"id,pop\n#CHROM,P1\n")),
        Ok(vec!["#CHROM"])
    );
}

#[test]
fn a_separator_with_which_a_quote_is_never_closed_does_not_fit() {
    assert_eq!(
        import(b"id,x;y,z\nA,b;c,\"d"),
        Ok(table(
            "id,x",
            &["A,b"],
            vec![column("y,z", 2, texts(&[Some("c,\"d")]))],
            utf8(Separator::Semicolon, DecimalMark::Point),
        ))
    );
}

#[test]
fn the_blank_rows_are_skipped_when_the_separator_is_searched() {
    assert_eq!(
        import(b"id;h, m, cm\nA;1\n\nB;2"),
        Ok(table(
            "id",
            &["A", "B"],
            vec![column("h, m, cm", 2, integers(&[Some(1), Some(2)]))],
            utf8(Separator::Semicolon, DecimalMark::Point),
        ))
    );
}

#[test]
fn a_separator_fits_rows_longer_than_the_header_by_empty_cells() {
    assert_eq!(
        import(b"id;h, m, cm\nA;1;\nB;2"),
        Ok(table(
            "id",
            &["A", "B"],
            vec![column("h, m, cm", 2, integers(&[Some(1), Some(2)]))],
            utf8(Separator::Semicolon, DecimalMark::Point),
        ))
    );
}

#[test]
fn when_the_semicolon_and_the_comma_both_fit_with_two_cells_the_semicolon_is_taken() {
    assert_eq!(
        import(b"id;n,x\nA;1,2"),
        Ok(table(
            "id",
            &["A"],
            vec![column("n,x", 2, floats(&[Some(1.2)]))],
            utf8(Separator::Semicolon, DecimalMark::Comma),
        ))
    );
}

#[test]
fn the_separator_is_found_with_the_header_counted_without_its_empty_cells() {
    assert_eq!(
        import(b"a;b,c;;\nx;y,z\n"),
        Ok(table(
            "a",
            &["x"],
            vec![column("b,c", 2, texts(&[Some("y,z")]))],
            utf8(Separator::Semicolon, DecimalMark::Point),
        ))
    );
}

#[test]
fn a_ragged_row_is_measured_against_the_header_without_its_empty_cells() {
    assert_eq!(
        import(b"id;pop;;\nA;P1\nB\n"),
        refused(Refusal::RaggedRow {
            line: 3,
            expected: 2,
            found: 1,
            separator: Separator::Semicolon,
        })
    );
}

#[test]
fn a_column_with_no_name_and_one_value_among_its_missing_cells_is_an_unnamed_column() {
    assert_eq!(
        import(b"id,,pop\nA,NA,P1\nB,x,P2\n"),
        refused(Refusal::UnnamedColumn { column: 2 })
    );
}

#[test]
fn a_file_with_no_header_takes_its_first_individual_for_the_names_of_the_columns() {
    assert_eq!(
        import(b"s000,p0\ns001,p1\ns002,p0\n"),
        Ok(table(
            "s000",
            &["s001", "s002"],
            vec![column("p0", 2, texts(&[Some("p1"), Some("p0")]))],
            utf8(Separator::Comma, DecimalMark::Point),
        ))
    );
}

#[test]
fn a_text_file_read_in_a_build_with_csv_is_a_table_and_not_unreadable() {
    assert_eq!(names_of(&import(b"id,pop\nA,P1\n")), Ok(vec!["A"]));
}

// The round trip of a table written as a CSV, "How it is verified" of
// docs/specs/text-files.md.
#[cfg(test)]
mod round_trip {
    use super::{
        Column, ColumnValues, DecimalMark, NameColumn, Separator, Table, import, import_with,
        separator_set, utf8,
    };

    /// The number of tables the property is tried on.
    const NUM_TABLES: u32 = 3_000;

    /// The seed of the generator of the tables, fixed so that a failure is
    /// seen again.
    const SEED: u64 = 0x5EED_7AB1_E10C_5A1E;

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

        /// One of `choices`.
        fn one_of<'choice, T>(&mut self, choices: &'choice [T]) -> &'choice T {
            &choices[self.below(choices.len())]
        }
    }

    /// The characters a cell of a generated table is made of: accents, a
    /// character outside the first plane, an emoji, the three separators, a
    /// quote, the line breaks, a space and a tab, and letters and digits. No
    /// `#`, so that no first line is a VCF's.
    const CHARACTERS: [char; 21] = [
        'a', 'B', 'n', 'A', 'N', '1', '7', '.', '-', 'ó', 'ñ', '𝔸', '😀', ',', ';', '\t', '"',
        '\n', '\r', ' ', 'e',
    ];

    /// The characters of [`CHARACTERS`] but the three separators, of which
    /// half the tables are made, so that their separator can be found.
    const CHARACTERS_WITHOUT_SEPARATORS: [char; 18] = [
        'a', 'B', 'n', 'A', 'N', '1', '7', '.', '-', 'ó', 'ñ', '𝔸', '😀', '"', '\n', '\r', ' ', 'e',
    ];

    /// A text of 0 to 5 of `characters`.
    fn generated_text(generator: &mut Generator, characters: &[char]) -> String {
        let length = generator.below(6);
        (0..length).map(|_| *generator.one_of(characters)).collect()
    }

    /// A text that is a value of a text column: a generated text with an `x`
    /// at some place, which no number, whole number, boolean or missing value
    /// holds.
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "a text of at most 5 characters"
    )]
    fn generated_value(generator: &mut Generator, characters: &[char]) -> String {
        let mut text = generated_text(generator, characters);
        let place = text
            .char_indices()
            .map(|(index, _)| index)
            .nth(generator.below(text.chars().count() + 1))
            .unwrap_or(text.len());
        text.insert(place, 'x');
        text
    }

    /// A text of `characters`, not empty and none of `taken`.
    fn generated_name(generator: &mut Generator, characters: &[char], taken: &[String]) -> String {
        loop {
            let name = generated_text(generator, characters);
            if !name.is_empty() && !taken.contains(&name) {
                return name;
            }
        }
    }

    /// A generated table: its header, and its rows of the name of an
    /// individual and the other cells, `None` for a missing one.
    struct GeneratedTable {
        /// The names of the columns, the first that of the individuals.
        header: Vec<String>,
        /// Each row: the name of its individual and its other cells.
        rows: Vec<(String, Vec<Option<String>>)>,
    }

    /// A table of 1 to 5 columns and 1 to 6 rows, made of [`CHARACTERS`] or,
    /// for half the tables, of [`CHARACTERS_WITHOUT_SEPARATORS`].
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "numbers of columns and rows below 7"
    )]
    fn generated_table(generator: &mut Generator) -> GeneratedTable {
        let characters: &[char] = if generator.below(2) == 0 {
            &CHARACTERS
        } else {
            &CHARACTERS_WITHOUT_SEPARATORS
        };
        let num_columns = 1 + generator.below(5);
        let num_rows = 1 + generator.below(6);
        let mut header = Vec::new();
        for _ in 0..num_columns {
            let name = generated_name(generator, characters, &header);
            header.push(name);
        }
        let mut names = Vec::new();
        let mut rows = Vec::new();
        for _ in 0..num_rows {
            let name = generated_name(generator, characters, &names);
            names.push(name.clone());
            let cells = (1..num_columns)
                .map(|_| (generator.below(4) != 0).then(|| generated_value(generator, characters)))
                .collect();
            rows.push((name, cells));
        }
        GeneratedTable { header, rows }
    }

    /// The character of `separator`.
    fn separator_character(separator: Separator) -> char {
        match separator {
            Separator::Tab => '\t',
            Separator::Semicolon => ';',
            Separator::Comma => ',',
        }
    }

    /// `cell` as a CSV writes it with `separator`: in quotes, each quote
    /// doubled, when it holds the separator, a quote or a line break, or a
    /// space or a tab at its ends.
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

    /// The text of `generated` as a CSV with `separator`, each line ended by
    /// `line_end` but the last when `ends_with_break` is false.
    fn written_table(
        generated: &GeneratedTable,
        separator: Separator,
        line_end: &str,
        ends_with_break: bool,
    ) -> String {
        let separator_text = separator_character(separator).to_string();
        let header_line = generated
            .header
            .iter()
            .map(|name| written_cell(name, separator))
            .collect::<Vec<_>>()
            .join(&separator_text);
        let row_lines = generated.rows.iter().map(|(name, cells)| {
            std::iter::once(written_cell(name, separator))
                .chain(cells.iter().map(|cell| {
                    cell.as_deref()
                        .map_or_else(String::new, |text| written_cell(text, separator))
                }))
                .collect::<Vec<_>>()
                .join(&separator_text)
        });
        let mut text = std::iter::once(header_line)
            .chain(row_lines)
            .collect::<Vec<_>>()
            .join(line_end);
        if ends_with_break {
            text.push_str(line_end);
        }
        text
    }

    /// The table the import should give of `generated`, read with
    /// `separator`: every column text, since every value holds an `x`.
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "the index of a column from 1 to 4"
    )]
    fn expected_table(generated: &GeneratedTable, separator: Separator) -> Table {
        let columns = generated
            .header
            .iter()
            .enumerate()
            .skip(1)
            .map(|(index, name)| Column {
                name: name.clone(),
                number: u32::try_from(index + 1).unwrap(),
                values: ColumnValues::Text(
                    generated
                        .rows
                        .iter()
                        .map(|(_, cells)| cells[index - 1].clone())
                        .collect(),
                ),
            })
            .collect();
        Table {
            names: NameColumn {
                header: generated.header[0].clone(),
                number: 1,
                names: generated
                    .rows
                    .iter()
                    .map(|(name, _)| name.clone())
                    .collect(),
            },
            columns,
            read: utf8(separator, DecimalMark::Point),
        }
    }

    /// Whether no cell of `generated` holds any of the three separators.
    fn holds_no_separator(generated: &GeneratedTable) -> bool {
        let holds_one = |text: &str| text.contains(['\t', ';', ',']);
        !generated.header.iter().any(|name| holds_one(name))
            && !generated.rows.iter().any(|(name, cells)| {
                holds_one(name) || cells.iter().flatten().any(|cell| holds_one(cell))
            })
    }

    #[test]
    fn a_table_written_as_a_csv_reads_back_as_itself_with_its_separator_set_and_found() {
        let mut generator = Generator { state: SEED };
        let mut num_found = 0_u32;
        for _ in 0..NUM_TABLES {
            let generated = generated_table(&mut generator);
            let separator =
                *generator.one_of(&[Separator::Tab, Separator::Semicolon, Separator::Comma]);
            let line_end = *generator.one_of(&["\n", "\r\n", "\r"]);
            let ends_with_break = generator.below(2) == 0;
            let text = written_table(&generated, separator, line_end, ends_with_break);
            let expected = Ok(expected_table(&generated, separator));

            assert_eq!(
                import_with(text.as_bytes(), separator_set(separator)),
                expected,
                "{text:?}"
            );
            if generated.header.len() >= 2 && holds_no_separator(&generated) {
                num_found += 1;
                assert_eq!(import(text.as_bytes()), expected, "{text:?}");
            }
        }
        // The tables read with no separator set too, a count the report gives.
        assert!(num_found > 1_000, "{num_found} tables read with none set");
        println!("{NUM_TABLES} tables read with the separator set, {num_found} also with none set");
    }
}

// The owner's text files, "How it is verified" of
// docs/specs/text-files.md, each read with no option set.
#[cfg(test)]
mod owner_files {
    use super::{
        Column, DecimalMark, FoundEncoding, HowRead, NameColumn, PathBuf, Separator, Table, import,
        read_as, utf8,
    };

    /// The bytes of `file_name` in `tests/data/` at the root of the
    /// repository.
    fn owner_file(file_name: &str) -> Vec<u8> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/data")
            .join(file_name);
        std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
    }

    /// The names and the columns of a table, which two files of one sheet
    /// share whatever they were read with.
    fn names_and_columns(imported: Table) -> (NameColumn, Vec<Column>) {
        (imported.names, imported.columns)
    }

    #[test]
    #[ignore = "waits for tests/data/excel_es.csv, made by the owner"]
    fn excel_es_csv_is_read_as_windows_1252_with_semicolons_and_the_comma() {
        let imported = import(&owner_file("excel_es.csv")).unwrap();
        assert_eq!(
            imported.read,
            read_as(
                FoundEncoding::Windows1252,
                Separator::Semicolon,
                DecimalMark::Comma,
                None
            )
        );
        // The accents of the header and of a value read as the owner sees them
        // in Excel: none is U+FFFD or the two characters of a byte of UTF-8.
        let all_text = format!("{:?}", names_and_columns(imported));
        assert!(!all_text.contains(['\u{FFFD}', 'Ã']), "{all_text}");
    }

    #[test]
    #[ignore = "waits for tests/data/excel_es_utf8.csv, made by the owner"]
    fn excel_es_utf8_csv_is_read_as_utf8_with_semicolons_and_the_comma_and_the_table_of_excel_es() {
        let imported = import(&owner_file("excel_es_utf8.csv")).unwrap();
        assert_eq!(
            imported.read,
            utf8(Separator::Semicolon, DecimalMark::Comma)
        );
        assert_eq!(
            names_and_columns(imported),
            names_and_columns(import(&owner_file("excel_es.csv")).unwrap())
        );
    }

    #[test]
    #[ignore = "waits for tests/data/excel_mac.csv, made by the owner"]
    fn excel_mac_csv_gives_the_table_of_excel_es() {
        let imported = import(&owner_file("excel_mac.csv")).unwrap();
        println!("excel_mac.csv read as {:?}", imported.read);
        assert_eq!(
            names_and_columns(imported),
            names_and_columns(import(&owner_file("excel_es.csv")).unwrap())
        );
    }

    #[test]
    #[ignore = "waits for tests/data/excel_unicode.txt, made by the owner"]
    fn excel_unicode_txt_is_read_as_utf16_with_the_tab_and_the_table_of_excel_es() {
        let imported = import(&owner_file("excel_unicode.txt")).unwrap();
        let HowRead::Text(text_read) = &imported.read else {
            panic!("not read as a text file: {:?}", imported.read);
        };
        assert_eq!(
            (
                text_read.encoding,
                text_read.separator,
                text_read.undecoded_line
            ),
            (FoundEncoding::Utf16, Separator::Tab, None)
        );
        assert_eq!(
            names_and_columns(imported),
            names_and_columns(import(&owner_file("excel_es.csv")).unwrap())
        );
    }

    #[test]
    #[ignore = "waits for tests/data/libreoffice.csv, made by the owner"]
    fn libreoffice_csv_is_read_as_a_table() {
        let imported = import(&owner_file("libreoffice.csv")).unwrap();
        println!("libreoffice.csv read as {:?}", imported.read);
        let HowRead::Text(text_read) = &imported.read else {
            panic!("not read as a text file: {:?}", imported.read);
        };
        assert_eq!(text_read.undecoded_line, None);
    }
}
