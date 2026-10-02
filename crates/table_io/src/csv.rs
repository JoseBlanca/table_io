//! A text file, a CSV or a TSV, read into its rows of cells,
//! `docs/specs/text-files.md`: the bytes decoded, a variants file refused,
//! the separator found unless set, and the text split into lines and cells
//! by the quotes of RFC 4180. The rows then go to the table step, which
//! finds the decimal mark when it is not set and the separator does not
//! give it. It is behind the feature `csv`.
//!
//! A cell that needs no change of its text borrows it from the decoded
//! text, which is the bytes themselves for a file of valid UTF-8 with no
//! mark, so that such a file is held once (`docs/specs/text-files.md`,
//! "How it runs").

use std::borrow::Cow;

use crate::table::{Cell, HowReadSoFar, Origin, RowEnd, Rows, table_of_rows};
use crate::value::is_missing;
use crate::{
    DecimalMark, Encoding, Format, FoundEncoding, HowRead, ImportError, Refusal, Separator, Table,
    TextOptions, TextRead,
};

/// The mark of UTF-16 little endian, which Excel writes for "Unicode
/// Text".
const UTF16_LITTLE_ENDIAN_MARK: [u8; 2] = [0xFF, 0xFE];

/// The mark of UTF-16 big endian.
const UTF16_BIG_ENDIAN_MARK: [u8; 2] = [0xFE, 0xFF];

/// The mark of UTF-8, which Excel writes for "CSV UTF-8".
const UTF8_MARK: [u8; 3] = [0xEF, 0xBB, 0xBF];

/// The byte order mark as a character, which a file can hold after its
/// mark when it was saved twice with one.
const BYTE_ORDER_MARK: char = '\u{FEFF}';

/// The characters of Windows-1252 for the bytes `80` to `9F`, by the index
/// of the Encoding Standard, `index-windows-1252`, the five bytes it left
/// without a character being the control of Unicode of the same number
/// (`docs/specs/text-files.md`, "The bytes and the encoding"); every other
/// byte is the character of Unicode of its number.
const WINDOWS_1252_80_TO_9F: [char; 32] = [
    '\u{20AC}', '\u{0081}', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\u{008D}', '\u{017D}', '\u{008F}',
    '\u{0090}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}', '\u{0153}', '\u{009D}', '\u{017E}', '\u{0178}',
];

/// The starts of the first line of a variants file, a VCF: its first line,
/// and the header of its columns, which a VCF cut at the top starts with.
const VCF_STARTS: [&str; 2] = ["##fileformat=VCF", "#CHROM"];

/// The separators tried when none is set, in the order that settles a
/// tie: a tab is the least likely of the three to be inside a value.
const SEPARATORS_BY_PRECEDENCE: [Separator; 3] =
    [Separator::Tab, Separator::Semicolon, Separator::Comma];

/// The message of [`ImportError::Unreadable`] for a text with a line past
/// line 4,294,967,295, which no file within the limits of either
/// application has.
const LINE_PAST_U32: &str = "a line past line 4,294,967,295";

/// The table of the text file `bytes`, read with `options`, by
/// `docs/specs/text-files.md`.
///
/// # Errors
///
/// [`ImportError::Refused`], of the format text, for the first of: a file
/// in UTF-16 cut short in a character, [`Refusal::CutShort`]; a byte 0 in
/// a file that is not UTF-16, [`Refusal::NotText`]; a variants file,
/// [`Refusal::VariantsFile`]; a quote never closed,
/// [`Refusal::UnclosedQuote`]; then the refusals of the rows of the table
/// step. [`ImportError::Unreadable`] for a line past line 4,294,967,295.
pub(crate) fn table_of_text(bytes: &[u8], options: &TextOptions) -> Result<Table, ImportError> {
    let refused = |refusal| ImportError::Refused {
        format: Format::Text,
        refusal,
    };
    let decoded = decoded(bytes, options.encoding).map_err(refused)?;
    let text = decoded.text.trim_start_matches(BYTE_ORDER_MARK);
    if is_variants_file(text) {
        return Err(refused(Refusal::VariantsFile));
    }
    let separator = match options.separator {
        Some(separator) => separator,
        None => found_separator(text)?,
    };
    let rows = match rows_of(text, separator) {
        Ok(rows) => rows,
        Err(SplitStop::UnclosedQuote { line }) => {
            return Err(refused(Refusal::UnclosedQuote { line, separator }));
        }
        Err(SplitStop::LinePastU32) => {
            return Err(ImportError::Unreadable(LINE_PAST_U32.to_owned()));
        }
    };
    let undecoded_line = undecoded_line(text)?;
    let text_read = |decimal| {
        HowReadSoFar::Known(HowRead::Text(TextRead {
            encoding: decoded.encoding,
            separator,
            decimal,
            undecoded_line,
        }))
    };
    let read = match (options.decimal, separator) {
        (Some(decimal), _) => text_read(decimal),
        (None, Separator::Comma) => text_read(DecimalMark::Point),
        (None, Separator::Tab | Separator::Semicolon) => HowReadSoFar::DecimalToFind {
            encoding: decoded.encoding,
            separator,
            undecoded_line,
        },
    };
    table_of_rows(rows, read)
}

/// The text of a file and the encoding it was decoded with.
struct Decoded<'bytes> {
    /// The text, borrowed from the bytes when they are valid UTF-8 and
    /// were decoded with no change.
    text: Cow<'bytes, str>,
    /// The encoding used.
    encoding: FoundEncoding,
}

/// The order of the two bytes of a character of UTF-16.
#[derive(Debug, Clone, Copy)]
enum ByteOrder {
    /// The less significant byte first, after the mark `FF FE`.
    LittleEndian,
    /// The more significant byte first, after the mark `FE FF`.
    BigEndian,
}

/// The text of `bytes`, decoded by "The bytes and the encoding" of
/// `docs/specs/text-files.md` with `encoding`, the one the caller set, if
/// any: UTF-16 after its mark, whatever is set; UTF-8 after its mark when
/// none is set; and with none set and no mark, UTF-8 when the bytes are
/// valid UTF-8 and Windows-1252 when not. The mark is not in the text.
///
/// # Errors
///
/// [`Refusal::CutShort`] for UTF-16 that ends in the middle of a
/// character, and [`Refusal::NotText`] for any other file with a byte 0.
fn decoded(bytes: &[u8], encoding: Option<Encoding>) -> Result<Decoded<'_>, Refusal> {
    if let Some(units) = bytes.strip_prefix(&UTF16_LITTLE_ENDIAN_MARK) {
        return decoded_utf16(units, ByteOrder::LittleEndian);
    }
    if let Some(units) = bytes.strip_prefix(&UTF16_BIG_ENDIAN_MARK) {
        return decoded_utf16(units, ByteOrder::BigEndian);
    }
    if bytes.contains(&0) {
        return Err(Refusal::NotText);
    }
    let after_mark = bytes.strip_prefix(&UTF8_MARK);
    let body = after_mark.unwrap_or(bytes);
    let utf8_lossy = || Decoded {
        text: String::from_utf8_lossy(body),
        encoding: FoundEncoding::Utf8,
    };
    let windows_1252 = || Decoded {
        text: Cow::Owned(
            body.iter()
                .map(|&byte| windows_1252_character(byte))
                .collect(),
        ),
        encoding: FoundEncoding::Windows1252,
    };
    Ok(match (encoding, after_mark) {
        (Some(Encoding::Utf8), _) | (None, Some(_)) => utf8_lossy(),
        (Some(Encoding::Windows1252), _) => windows_1252(),
        (None, None) => match std::str::from_utf8(body) {
            Ok(text) => Decoded {
                text: Cow::Borrowed(text),
                encoding: FoundEncoding::Utf8,
            },
            Err(_) => windows_1252(),
        },
    })
}

/// The text of the bytes of UTF-16 `units`, after the mark, in
/// `byte_order`, a half of a character with no partner becoming U+FFFD.
///
/// # Errors
///
/// [`Refusal::CutShort`] when the bytes are odd in number or the last two
/// are the first half of a character written in four, a high surrogate.
fn decoded_utf16(units: &[u8], byte_order: ByteOrder) -> Result<Decoded<'static>, Refusal> {
    let (pairs, rest) = units.as_chunks::<2>();
    if !rest.is_empty() {
        return Err(Refusal::CutShort);
    }
    let unit_of = |pair: &[u8; 2]| match byte_order {
        ByteOrder::LittleEndian => u16::from_le_bytes(*pair),
        ByteOrder::BigEndian => u16::from_be_bytes(*pair),
    };
    let is_high_surrogate = |unit: u16| (0xD800..=0xDBFF).contains(&unit);
    if pairs.last().map(unit_of).is_some_and(is_high_surrogate) {
        return Err(Refusal::CutShort);
    }
    let text: String = char::decode_utf16(pairs.iter().map(unit_of))
        .map(|decoded_character| decoded_character.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect();
    Ok(Decoded {
        text: Cow::Owned(text),
        encoding: FoundEncoding::Utf16,
    })
}

/// The character of `byte` in Windows-1252.
fn windows_1252_character(byte: u8) -> char {
    byte.checked_sub(0x80)
        .and_then(|offset| WINDOWS_1252_80_TO_9F.get(usize::from(offset)))
        .copied()
        .unwrap_or(char::from(byte))
}

/// Whether `text` is a variants file: its first line that is not blank,
/// its spaces and tabs at the start left out, starts as a VCF's.
fn is_variants_file(text: &str) -> bool {
    let first_line = text.trim_start_matches([' ', '\t', '\r', '\n']);
    VCF_STARTS.iter().any(|start| first_line.starts_with(start))
}

/// The line of the first U+FFFD of `text`, counted from 1 as the lines of
/// the split are, or None when it has none.
///
/// # Errors
///
/// [`ImportError::Unreadable`] when that line is past line 4,294,967,295.
fn undecoded_line(text: &str) -> Result<Option<u32>, ImportError> {
    let Some(place) = text.find(char::REPLACEMENT_CHARACTER) else {
        return Ok(None);
    };
    let before = text.get(..place).unwrap_or_default().as_bytes();
    let num_breaks = before
        .iter()
        .enumerate()
        .filter(|&(index, &byte)| {
            byte == b'\n'
                || (byte == b'\r'
                    && index.checked_add(1).and_then(|next| before.get(next)) != Some(&b'\n'))
        })
        .count();
    num_breaks
        .checked_add(1)
        .and_then(|line| u32::try_from(line).ok())
        .map(Some)
        .ok_or_else(|| ImportError::Unreadable(LINE_PAST_U32.to_owned()))
}

/// The byte of `separator` in the text.
fn separator_byte(separator: Separator) -> u8 {
    match separator {
        Separator::Tab => b'\t',
        Separator::Semicolon => b';',
        Separator::Comma => b',',
    }
}

/// A cell as the split finds it, by the places of its bytes in the text,
/// its spaces at the ends left out.
#[derive(Debug, Clone, Copy)]
enum CellSpan {
    /// A cell that does not start with a quote: its text from `start` to
    /// `end`.
    Plain {
        /// The place of its first byte.
        start: usize,
        /// The place past its last byte.
        end: usize,
    },
    /// A cell that starts with a quote: what is inside its quotes, its
    /// doubled quotes not yet made one, and what follows the closing quote.
    Quoted {
        /// The place past the opening quote.
        inner_start: usize,
        /// The place of the closing quote, or the end of the text when the
        /// quote is never closed.
        inner_end: usize,
        /// The place past the closing quote.
        trail_start: usize,
        /// The place past the last byte that follows the closing quote,
        /// its spaces at the end left out.
        trail_end: usize,
    },
}

/// Why a split stopped before the end of the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SplitStop {
    /// A quote never closed, the cell it opens starting at `line`.
    UnclosedQuote {
        /// The line where the cell starts, from 1.
        line: u32,
    },
    /// A line past line 4,294,967,295.
    LinePastU32,
}

/// What a split gives its cells and the ends of its rows to.
trait SplitSink<'text> {
    /// The next cell of the row, at `span` of `text`.
    fn cell(&mut self, text: &'text str, span: CellSpan);
    /// The end of the row that started at `line`.
    fn row_end(&mut self, line: u32);
}

/// Splits `text` into rows of cells with `separator`, giving each cell
/// and the end of each row to `sink`, by "The lines, the quotes and the
/// cells" of `docs/specs/text-files.md`.
///
/// The lines end with `\r\n`, `\n` or `\r`, and are counted from 1, those
/// inside a quoted cell too; a text that ends with a line break has no
/// row after it, and the empty text no row. A cell that starts with `"`,
/// once its spaces are left out, goes on to the next `"` that is not
/// doubled; a `"` elsewhere is an ordinary character. Spaces, and tabs
/// when the separator is not the tab, are left out at the ends of a cell
/// and kept inside its quotes.
///
/// # Errors
///
/// [`SplitStop::UnclosedQuote`] for a quote never closed, once the cell it
/// opens, which takes the rest of the text, and the end of its row are
/// given to `sink`; [`SplitStop::LinePastU32`] for a line past line
/// 4,294,967,295.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "a place is at most the length of the text, of at most isize::MAX bytes, and \
              is moved by 1 or 2 only while it is below that length, so it fits a usize"
)]
fn split<'text>(
    text: &'text str,
    separator: Separator,
    sink: &mut impl SplitSink<'text>,
) -> Result<(), SplitStop> {
    let bytes = text.as_bytes();
    let length = bytes.len();
    if length == 0 {
        return Ok(());
    }
    let separator_byte = separator_byte(separator);
    let byte_at = |place: usize| bytes.get(place).copied();
    let is_blank = |byte: u8| byte == b' ' || (byte == b'\t' && separator != Separator::Tab);
    let ends_cell = |byte: u8| byte == separator_byte || byte == b'\n' || byte == b'\r';
    let end_without_blanks = |start: usize, end: usize| {
        let mut kept_end = end;
        while kept_end > start && byte_at(kept_end - 1).is_some_and(is_blank) {
            kept_end -= 1;
        }
        kept_end
    };
    let next_line = |line: u32| line.checked_add(1).ok_or(SplitStop::LinePastU32);
    let mut place = 0;
    let mut line: u32 = 1;
    loop {
        let row_line = line;
        loop {
            while byte_at(place).is_some_and(is_blank) {
                place += 1;
            }
            if byte_at(place) == Some(b'"') {
                let cell_line = line;
                let inner_start = place + 1;
                place = inner_start;
                let inner_end = loop {
                    match byte_at(place) {
                        None => {
                            sink.cell(
                                text,
                                CellSpan::Quoted {
                                    inner_start,
                                    inner_end: length,
                                    trail_start: length,
                                    trail_end: length,
                                },
                            );
                            sink.row_end(row_line);
                            return Err(SplitStop::UnclosedQuote { line: cell_line });
                        }
                        Some(b'"') => {
                            if byte_at(place + 1) == Some(b'"') {
                                place += 2;
                                continue;
                            }
                            break place;
                        }
                        Some(b'\r') => {
                            line = next_line(line)?;
                            if byte_at(place + 1) == Some(b'\n') {
                                place += 1;
                            }
                        }
                        Some(b'\n') => line = next_line(line)?,
                        Some(_) => {}
                    }
                    place += 1;
                };
                place += 1;
                let trail_start = place;
                while byte_at(place).is_some_and(|byte| !ends_cell(byte)) {
                    place += 1;
                }
                sink.cell(
                    text,
                    CellSpan::Quoted {
                        inner_start,
                        inner_end,
                        trail_start,
                        trail_end: end_without_blanks(trail_start, place),
                    },
                );
            } else {
                let start = place;
                while byte_at(place).is_some_and(|byte| !ends_cell(byte)) {
                    place += 1;
                }
                sink.cell(
                    text,
                    CellSpan::Plain {
                        start,
                        end: end_without_blanks(start, place),
                    },
                );
            }
            if byte_at(place) == Some(separator_byte) {
                place += 1;
                continue;
            }
            break;
        }
        sink.row_end(row_line);
        if place >= length {
            break;
        }
        if byte_at(place) == Some(b'\r') && byte_at(place + 1) == Some(b'\n') {
            place += 1;
        }
        place += 1;
        if place >= length {
            break;
        }
        line = next_line(line)?;
    }
    Ok(())
}

/// The parts of the text of a quoted cell: inside its quotes, its doubled
/// quotes not yet made one, and after its closing quote.
fn quoted_parts(
    text: &str,
    inner_start: usize,
    inner_end: usize,
    trail_start: usize,
    trail_end: usize,
) -> (&str, &str) {
    // The places are those of the split, each at an ASCII byte or the end
    // of the text, so at the boundary of a character; the default is
    // never taken.
    (
        text.get(inner_start..inner_end).unwrap_or_default(),
        text.get(trail_start..trail_end).unwrap_or_default(),
    )
}

/// The cell of the table at `span` of `text`: empty when it has no
/// character, else its text, borrowed when it needs no change, and with
/// its doubled quotes made one and what follows its closing quote added
/// when it was quoted.
fn cell_of_span(text: &str, span: CellSpan) -> Cell<'_> {
    let cell_text = match span {
        CellSpan::Plain { start, end } => Cow::Borrowed(text.get(start..end).unwrap_or_default()),
        CellSpan::Quoted {
            inner_start,
            inner_end,
            trail_start,
            trail_end,
        } => {
            let (inner, trail) = quoted_parts(text, inner_start, inner_end, trail_start, trail_end);
            if inner.contains("\"\"") {
                Cow::Owned(inner.replace("\"\"", "\"") + trail)
            } else if trail.is_empty() {
                Cow::Borrowed(inner)
            } else if inner.is_empty() {
                Cow::Borrowed(trail)
            } else {
                Cow::Owned([inner, trail].concat())
            }
        }
    };
    if cell_text.is_empty() {
        Cell::Empty
    } else {
        Cell::Text(cell_text)
    }
}

/// The rows of a text as the split makes them.
struct RowsSink<'text> {
    /// The cells of every row, row after row.
    cells: Vec<Cell<'text>>,
    /// Where each row ends among the cells, and its line.
    row_ends: Vec<RowEnd>,
}

impl<'text> SplitSink<'text> for RowsSink<'text> {
    fn cell(&mut self, text: &'text str, span: CellSpan) {
        self.cells.push(cell_of_span(text, span));
    }

    fn row_end(&mut self, line: u32) {
        self.row_ends.push(RowEnd {
            place: line,
            end: self.cells.len(),
        });
    }
}

/// The numbers of cells and of rows of a text, counted by a split that
/// makes no cell.
#[derive(Debug, Clone, Copy)]
struct CellCount {
    /// The number of cells.
    num_cells: usize,
    /// The number of rows.
    num_rows: usize,
}

impl<'text> SplitSink<'text> for CellCount {
    fn cell(&mut self, _text: &'text str, _span: CellSpan) {
        self.num_cells = self.num_cells.saturating_add(1);
    }

    fn row_end(&mut self, _line: u32) {
        self.num_rows = self.num_rows.saturating_add(1);
    }
}

/// The rows of `text` split with `separator`, each cell made once.
///
/// The cells are counted by a split before they are made, so that their
/// `Vec` is made once at its size: grown by doubling, the `Vec` of the
/// 9,761,300 cells of a CSV of 20,000,000 bytes of `0,` held 16,777,216
/// slots and, while it grew, the 8,388,608 before them too, and the import
/// of that file peaked at 618.7 MB natively, where it peaks at 391.0 MB
/// with the count (measured with a counting allocator on the owner's Mac,
/// release build, 2 October 2026).
///
/// # Errors
///
/// The [`SplitStop`] of [`split`].
fn rows_of(text: &str, separator: Separator) -> Result<Rows<'_>, SplitStop> {
    let mut count = CellCount {
        num_cells: 0,
        num_rows: 0,
    };
    split(text, separator, &mut count)?;
    let mut sink = RowsSink {
        cells: Vec::with_capacity(count.num_cells),
        row_ends: Vec::with_capacity(count.num_rows),
    };
    split(text, separator, &mut sink)?;
    let RowsSink { cells, row_ends } = sink;
    Ok(Rows {
        origin: Origin::Text { separator },
        first_column: 1,
        cells,
        row_ends,
    })
}

/// What a cell holds, as the search of the separator needs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellKind {
    /// No character.
    Empty,
    /// `NA` or `-`, a missing value outside the first column.
    Missing,
    /// Any other text.
    Value,
}

/// What the cell at `span` of `text` holds, without making its text but
/// for a quoted cell followed by more text, `"x"y`.
fn kind_of_span(text: &str, span: CellSpan) -> CellKind {
    let kind_of_text = |cell_text: &str| {
        if cell_text.is_empty() {
            CellKind::Empty
        } else if is_missing(cell_text) {
            CellKind::Missing
        } else {
            CellKind::Value
        }
    };
    match span {
        CellSpan::Plain { start, end } => kind_of_text(text.get(start..end).unwrap_or_default()),
        CellSpan::Quoted {
            inner_start,
            inner_end,
            trail_start,
            trail_end,
        } => {
            let (inner, trail) = quoted_parts(text, inner_start, inner_end, trail_start, trail_end);
            if inner.contains('"') {
                // A doubled quote, one `"` of the text of the cell.
                CellKind::Value
            } else if trail.is_empty() {
                kind_of_text(inner)
            } else if inner.is_empty() {
                kind_of_text(trail)
            } else {
                kind_of_text(&[inner, trail].concat())
            }
        }
    }
}

/// The header of a text split with one separator, as the search of the
/// separator counts it.
#[derive(Debug, Clone, Copy)]
struct HeaderCount {
    /// Its number of cells.
    num_cells: usize,
    /// The place past its last cell that is not empty, at least 1.
    named_end: usize,
}

/// The counts of the rows of a text split with one separator, made as
/// the split goes, without making the cells.
#[derive(Debug, Clone)]
struct SeparatorCount {
    /// The number of cells of the row being split.
    row_num_cells: usize,
    /// The place past the last cell of the row that is not empty, 0 when
    /// none is.
    row_named_end: usize,
    /// The place past the last cell of the row, below the header's
    /// number of cells, that holds a value, 0 when none does.
    row_value_end: usize,
    /// Whether every cell of the row past the header's number of cells is
    /// empty.
    is_row_empty_past_header: bool,
    /// The header, the first row that is not blank, once split.
    header: Option<HeaderCount>,
    /// The fewest cells of a row below the header that is not blank.
    min_row_cells: Option<usize>,
    /// The place past the last column, below the header's number of
    /// cells, where some row below the header holds a value.
    value_end: usize,
    /// Whether every row below the header has only empty cells past the
    /// header's number of cells.
    is_every_row_empty_past_header: bool,
}

impl SeparatorCount {
    /// The counts of no row.
    fn new() -> Self {
        Self {
            row_num_cells: 0,
            row_named_end: 0,
            row_value_end: 0,
            is_row_empty_past_header: true,
            header: None,
            min_row_cells: None,
            value_end: 0,
            is_every_row_empty_past_header: true,
        }
    }

    /// The number of cells of the header without the run of empty cells at
    /// its end whose columns hold no value in any row, the first cell
    /// always counted; 0 with no header.
    fn num_counted(&self) -> usize {
        self.header
            .map_or(0, |header| header.named_end.max(self.value_end))
    }

    /// Whether the separator fits the text: the header counted as two cells
    /// or more, and every row below it with as many cells or more, those
    /// past the whole header all empty. A quote never closed is told apart
    /// by the caller.
    fn fits(&self) -> bool {
        let num_counted = self.num_counted();
        num_counted >= 2
            && self.is_every_row_empty_past_header
            && self
                .min_row_cells
                .is_none_or(|min_row_cells| min_row_cells >= num_counted)
    }
}

impl<'text> SplitSink<'text> for SeparatorCount {
    fn cell(&mut self, text: &'text str, span: CellSpan) {
        let index = self.row_num_cells;
        let past_index = index.saturating_add(1);
        self.row_num_cells = past_index;
        let kind = kind_of_span(text, span);
        if kind != CellKind::Empty {
            self.row_named_end = past_index;
        }
        if let Some(header) = self.header {
            if index < header.num_cells {
                if kind == CellKind::Value {
                    self.row_value_end = past_index;
                }
            } else if kind != CellKind::Empty {
                self.is_row_empty_past_header = false;
            }
        }
    }

    fn row_end(&mut self, _line: u32) {
        let is_blank = self.row_named_end == 0;
        if !is_blank {
            match self.header {
                None => {
                    self.header = Some(HeaderCount {
                        num_cells: self.row_num_cells,
                        named_end: self.row_named_end.max(1),
                    });
                }
                Some(_) => {
                    self.min_row_cells = Some(
                        self.min_row_cells
                            .map_or(self.row_num_cells, |min_row_cells| {
                                min_row_cells.min(self.row_num_cells)
                            }),
                    );
                    self.value_end = self.value_end.max(self.row_value_end);
                    self.is_every_row_empty_past_header &= self.is_row_empty_past_header;
                }
            }
        }
        self.row_num_cells = 0;
        self.row_named_end = 0;
        self.row_value_end = 0;
        self.is_row_empty_past_header = true;
    }
}

/// The separator of `text` when none is set, by "The separator" of
/// `docs/specs/text-files.md`: of the three that fit, the one that gives
/// the header the most cells, a tie going by [`SEPARATORS_BY_PRECEDENCE`];
/// when none fits, the same over the three, the header counted from the
/// rows split up to a quote never closed; and the comma when every one
/// gives the header fewer than two cells.
///
/// # Errors
///
/// [`ImportError::Unreadable`] for a line past line 4,294,967,295.
fn found_separator(text: &str) -> Result<Separator, ImportError> {
    let mut tries = Vec::with_capacity(SEPARATORS_BY_PRECEDENCE.len());
    for separator in SEPARATORS_BY_PRECEDENCE {
        let mut count = SeparatorCount::new();
        let is_closed = match split(text, separator, &mut count) {
            Ok(()) => true,
            Err(SplitStop::UnclosedQuote { .. }) => false,
            Err(SplitStop::LinePastU32) => {
                return Err(ImportError::Unreadable(LINE_PAST_U32.to_owned()));
            }
        };
        tries.push((separator, count.num_counted(), is_closed && count.fits()));
    }
    let most_cells = |candidates: &mut dyn Iterator<Item = &(Separator, usize, bool)>| {
        candidates
            .fold(
                None,
                |best: Option<(Separator, usize)>, &(separator, num_counted, _)| match best {
                    Some((_, best_counted)) if best_counted >= num_counted => best,
                    Some(_) | None => Some((separator, num_counted)),
                },
            )
            .filter(|&(_, num_counted)| num_counted >= 2)
            .map(|(separator, _)| separator)
    };
    Ok(most_cells(&mut tries.iter().filter(|&&(_, _, fits)| fits))
        .or_else(|| most_cells(&mut tries.iter()))
        .unwrap_or(Separator::Comma))
}
