//! The tables of the round trip of "What reads back" of
//! `docs/specs/export.md`, made by a generator with a fixed seed, so that
//! the export of a CSV, `tests/export_csv.rs`, and of an xlsx,
//! `tests/export_xlsx.rs`, are tried on the same tables; and what both
//! need to make a refused table again and to check that a refusal is due.
//!
//! Each test crate that takes it is declared `#[cfg(test)]`, so that
//! clippy reads its `unwrap`s as those of a test.

use table_io::{
    CellPlace, Column, ColumnType, ColumnValues, DecimalMark, ExportRefusal, NameColumn,
    parse_boolean, parse_float, parse_integer,
};

/// The number of tables the property is tried on, each exported with
/// the 36 combinations of the choices of a CSV and as an xlsx.
pub const NUM_TABLES: u32 = 2_000;

/// The seed of the generator of the tables, fixed so that a failure is
/// seen again.
pub const SEED: u64 = 0xE4_7AB1_E0C5_7E57;

/// The most times a table refused is made again for one format;
/// each repair removes one value or character of the few in a table.
pub const MAX_REPAIRS: u32 = 200;

/// A generator of numbers, xorshift64*, which needs no dependency.
pub struct Generator {
    /// The state, never 0.
    pub state: u64,
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
    pub fn below(&mut self, below: usize) -> usize {
        usize::try_from(
            self.next_number()
                .checked_rem(u64::try_from(below).unwrap())
                .unwrap(),
        )
        .unwrap()
    }

    /// Whether a chance of 1 in `chances` came.
    pub fn one_in(&mut self, chances: usize) -> bool {
        self.below(chances) == 0
    }

    /// One of `choices`.
    pub fn one_of<'choice, T>(&mut self, choices: &'choice [T]) -> &'choice T {
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
    'a', 'B', 'n', 'A', 'N', '1', '7', '0', '.', '-', '+', 'e', 'E', 'ó', 'ñ', '€', 'ő', '𝔸', '😀',
    '"', ',', ';', '\t', '\n', '\r', ' ', '#', '\u{FFFD}', '\u{FFFE}',
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
pub fn generated_table(generator: &mut Generator) -> (NameColumn, Vec<Column>) {
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
                1 => {
                    ColumnValues::Float((0..num_rows).map(|_| generated_float(generator)).collect())
                }
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

/// The text of the cell at `place`: a name of the header, of an
/// individual, or a text value.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "a refusal names a column from 1 and a row from 0, of a table of a few"
)]
pub fn text_at<'table>(
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

/// The table made again without what `refusal` names: a name given
/// that of a repair, `fix` and a number, a value made missing, the
/// character that cannot be carried removed, or the spaces and tabs at
/// the ends of a text removed.
pub fn repaired(
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
        ExportRefusal::ErrorAsName { place } => *text_at(names, columns, *place) = fixed_name,
        ExportRefusal::ReadsAsMissing { place }
        | ExportRefusal::NotFinite { place }
        | ExportRefusal::IntegerTooLarge { place } => {
            make_missing(columns, *place);
        }
        ExportRefusal::CannotCarry { place, character } => {
            text_at(names, columns, *place).retain(|kept| kept != *character);
        }
        ExportRefusal::SpacesAtEnds { place } => {
            let text = text_at(names, columns, *place);
            *text = text.trim_matches([' ', '\t']).to_owned();
        }
        ExportRefusal::ReadsAsVariantsFile => names.header = fixed_name,
        ExportRefusal::FormatNotBuilt
        | ExportRefusal::NoIndividual
        | ExportRefusal::WrongLength { .. }
        | ExportRefusal::TextTooLong { .. }
        | ExportRefusal::TooLargeForSheet { .. } => {
            panic!("a refusal a generated table does not meet: {refusal:?}")
        }
    }
}

/// Whether `refusal` of `names` and `columns`, one that every format
/// gives, is due by the rule of the spec it names, or None for a refusal
/// of a format.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "a refusal of a value names a column from 2 and a row from 1"
)]
pub fn is_due_in_every_format(
    names: &NameColumn,
    columns: &[Column],
    refusal: &ExportRefusal,
) -> Option<bool> {
    let mut names_copy = names.clone();
    let mut columns_copy = columns.to_vec();
    let mut text_of = |place: CellPlace| text_at(&mut names_copy, &mut columns_copy, place).clone();
    let header_place = |column: u32| CellPlace { column, row: 0 };
    let name_place = |row: u32| CellPlace { column: 1, row };
    Some(match refusal {
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
        ExportRefusal::NotFinite { place } => {
            let row = usize::try_from(place.row).unwrap() - 1;
            match &columns[usize::try_from(place.column).unwrap() - 2].values {
                ColumnValues::Float(floats) => floats[row].is_some_and(|float| !float.is_finite()),
                ColumnValues::Integer(_) | ColumnValues::Boolean(_) | ColumnValues::Text(_) => {
                    false
                }
            }
        }
        ExportRefusal::FormatNotBuilt
        | ExportRefusal::NoIndividual
        | ExportRefusal::WrongLength { .. }
        | ExportRefusal::ReadsAsMissing { .. }
        | ExportRefusal::ErrorAsName { .. }
        | ExportRefusal::SpacesAtEnds { .. }
        | ExportRefusal::IntegerTooLarge { .. }
        | ExportRefusal::CannotCarry { .. }
        | ExportRefusal::TextTooLong { .. }
        | ExportRefusal::TooLargeForSheet { .. }
        | ExportRefusal::ReadsAsVariantsFile => return None,
    })
}

/// Whether every value of `values` is missing.
pub fn is_all_missing(values: &ColumnValues) -> bool {
    match values {
        ColumnValues::Integer(integers) => integers.iter().all(Option::is_none),
        ColumnValues::Float(floats) => floats.iter().all(Option::is_none),
        ColumnValues::Boolean(booleans) => booleans.iter().all(Option::is_none),
        ColumnValues::Text(texts) => texts.iter().all(Option::is_none),
    }
}

/// The type a text column of `texts`, not all missing, reads back as with
/// `decimal`, by "What reads back": the narrowest type of its values.
pub fn narrowest_type_of_texts(texts: &[Option<String>], decimal: DecimalMark) -> ColumnType {
    let present: Vec<&str> = texts.iter().flatten().map(String::as_str).collect();
    if present.iter().all(|text| parse_integer(text).is_some()) {
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
