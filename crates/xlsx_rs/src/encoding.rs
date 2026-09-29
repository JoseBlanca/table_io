//! The rule of UTF-8 by the bytes of a part, as "What xlsx_rs reads before
//! calamine" of `docs/specs/read.md`, point 5, gives it: whether calamine
//! could decode a part in an encoding other than UTF-8, found in the bytes
//! of the part as they pass, a read of the zip at a time.
//!
//! calamine decodes the texts of a part in the encoding quick-xml finds for
//! it: from a byte order mark or the first bytes of the part, or from the
//! `encoding` of the first declaration `<?xml … ?>` whose encoding it knows,
//! wherever that declaration is in the part. A byte of windows-1252 can be
//! 3 bytes of UTF-8, so a bound counted in the bytes of such a part holds 3
//! times as much. The rule finds every part quick-xml could decode so, and
//! some it would not: every `encoding` of every declaration is checked,
//! also one inside a comment or the value of another attribute.

/// Whether the bytes of a part, given a read at a time, show an encoding
/// other than UTF-8: its first two bytes `FF FE`, `FE FF`, `00 3C` or
/// `3C 00`, or the bytes `<?xml` followed, before the next `?>`, by an
/// `encoding`, `=` with any spaces, tabs or line breaks around it, and a
/// value in either quote that, trimmed of spaces, tabs and line breaks, is
/// not `utf-8` or `utf8`, ignoring case, or that cannot be read so: a value
/// not in quotes, or not closed by its quote before `?>`.
///
/// Nothing of the bytes is held but what the next read needs, at most the
/// five bytes of a value: a match split between two reads is found, at any
/// distance from the `<?xml` it follows.
pub(crate) struct EncodingCheck {
    /// How much of the first two bytes of the part has been given.
    start: Start,
    /// Where the bytes given so far end, in the search for a declaration
    /// and its `encoding`.
    state: State,
}

impl EncodingCheck {
    /// A check of a part none of whose bytes has been given.
    pub(crate) fn new() -> Self {
        Self {
            start: Start::NoByte,
            state: State::OutsideDeclaration { num_matched: 0 },
        }
    }

    /// Checks `bytes`, the bytes of the part that follow those given
    /// before.
    pub(crate) fn check(&mut self, bytes: &[u8]) {
        self.check_start(bytes);
        let mut rest = bytes;
        loop {
            if self.is_other_encoding() {
                return;
            }
            if let State::OutsideDeclaration { num_matched: 0 } = self.state {
                // Most bytes of a part are outside a declaration, and only a
                // `<` can start one.
                let Some(mark_position) = rest.iter().position(|byte| *byte == b'<') else {
                    return;
                };
                rest = rest
                    .get(mark_position.saturating_add(1)..)
                    .unwrap_or_default();
                self.state = State::OutsideDeclaration { num_matched: 1 };
                continue;
            }
            let Some((byte, after_byte)) = rest.split_first() else {
                return;
            };
            self.state = self.state.after(*byte);
            rest = after_byte;
        }
    }

    /// Whether the bytes given so far show an encoding other than UTF-8.
    pub(crate) fn is_other_encoding(&self) -> bool {
        matches!(self.state, State::OtherEncoding)
    }

    /// Checks the first two bytes of the part among `bytes`, the bytes that
    /// follow those given before.
    fn check_start(&mut self, bytes: &[u8]) {
        let first_two_bytes = match (self.start, bytes) {
            (Start::Checked, _) | (Start::NoByte | Start::FirstByte(_), []) => return,
            (Start::NoByte, [first_byte]) => {
                self.start = Start::FirstByte(*first_byte);
                return;
            }
            (Start::NoByte, [first_byte, second_byte, ..]) => [*first_byte, *second_byte],
            (Start::FirstByte(first_byte), [second_byte, ..]) => [first_byte, *second_byte],
        };
        self.start = Start::Checked;
        if UTF_16_STARTS.contains(&first_two_bytes) {
            self.state = State::OtherEncoding;
        }
    }
}

/// The first two bytes of a part quick-xml may decode in UTF-16: the byte
/// order marks of UTF-16, little-endian and big-endian, and `<` in each.
const UTF_16_STARTS: [[u8; 2]; 4] = [[0xFF, 0xFE], [0xFE, 0xFF], [0x00, b'<'], [b'<', 0x00]];

/// The bytes a declaration starts with.
const DECLARATION_START: &[u8] = b"<?xml";

/// The bytes of the name of the attribute of a declaration that gives the
/// encoding of the part.
const ENCODING_NAME: &[u8] = b"encoding";

/// The values of `encoding` that name UTF-8, in lower case.
const UTF_8_NAMES: [&[u8]; 2] = [b"utf-8", b"utf8"];

/// How much of the first two bytes of a part has been given.
#[derive(Debug, Clone, Copy)]
enum Start {
    /// None of them.
    NoByte,
    /// The first, this byte.
    FirstByte(u8),
    /// Both, and they were checked.
    Checked,
}

/// Where the bytes given so far end, in the search for a declaration and
/// its `encoding`.
#[derive(Debug, Clone, Copy)]
enum State {
    /// Outside a declaration, the first `num_matched` bytes of `<?xml`
    /// matched at the end of the bytes given.
    OutsideDeclaration {
        /// The bytes of `<?xml` matched, 0 to 4.
        num_matched: usize,
    },
    /// Inside a declaration, after its `<?xml`, the first `num_matched`
    /// bytes of `encoding` matched at the end of the bytes given.
    InsideDeclaration {
        /// The bytes of `encoding` matched, 0 to 7.
        num_matched: usize,
        /// Whether the last byte given is `?`, which a `>` makes the end of
        /// the declaration.
        is_after_question_mark: bool,
    },
    /// After an `encoding` of a declaration and any spaces.
    BeforeEquals,
    /// After the `=` of an `encoding` and any spaces.
    BeforeValue,
    /// Inside the value of an `encoding`.
    InsideValue(Value),
    /// An encoding other than UTF-8 was found; nothing more is checked.
    OtherEncoding,
}

impl State {
    /// Where the bytes end once `byte` follows them.
    fn after(self, byte: u8) -> Self {
        match self {
            Self::OutsideDeclaration { num_matched } => {
                if DECLARATION_START.get(num_matched) == Some(&byte) {
                    let num_matched = num_matched.saturating_add(1);
                    if num_matched == DECLARATION_START.len() {
                        Self::start_of_declaration()
                    } else {
                        Self::OutsideDeclaration { num_matched }
                    }
                } else {
                    // `<?xml` holds only one `<`, its first byte, so a
                    // match that fails starts again at a `<`.
                    Self::OutsideDeclaration {
                        num_matched: usize::from(byte == b'<'),
                    }
                }
            }
            Self::InsideDeclaration {
                num_matched,
                is_after_question_mark,
            } => {
                if is_after_question_mark && byte == b'>' {
                    return Self::OutsideDeclaration { num_matched: 0 };
                }
                // `encoding` holds only one `e`, its first byte, so a match
                // that fails starts again at an `e`.
                let num_matched = if ENCODING_NAME.get(num_matched) == Some(&byte) {
                    num_matched.saturating_add(1)
                } else {
                    usize::from(byte == b'e')
                };
                if num_matched == ENCODING_NAME.len() {
                    Self::BeforeEquals
                } else {
                    Self::InsideDeclaration {
                        num_matched,
                        is_after_question_mark: byte == b'?',
                    }
                }
            }
            Self::BeforeEquals => {
                if is_space(byte) {
                    Self::BeforeEquals
                } else if byte == b'=' {
                    Self::BeforeValue
                } else {
                    // Not an attribute `encoding`: the byte is read as any
                    // other of the declaration, `?` of its end among them.
                    Self::start_of_declaration().after(byte)
                }
            }
            Self::BeforeValue => {
                if is_space(byte) {
                    Self::BeforeValue
                } else if byte == b'"' || byte == b'\'' {
                    Self::InsideValue(Value::opened_by(byte))
                } else {
                    Self::OtherEncoding
                }
            }
            Self::InsideValue(value) => value.after(byte),
            Self::OtherEncoding => Self::OtherEncoding,
        }
    }

    /// Inside a declaration, with nothing of `encoding` matched.
    fn start_of_declaration() -> Self {
        Self::InsideDeclaration {
            num_matched: 0,
            is_after_question_mark: false,
        }
    }
}

/// The value of an `encoding` read so far, as long as it can still be one
/// of [`UTF_8_NAMES`].
#[derive(Debug, Clone, Copy)]
struct Value {
    /// The quote that opened it, and closes it.
    quote: u8,
    /// Its bytes after any spaces first, in lower case, the first
    /// `num_bytes` of them.
    bytes: [u8; 5],
    /// The bytes of `bytes` read.
    num_bytes: usize,
    /// Whether spaces followed its bytes, after which only the quote can
    /// come.
    is_after_spaces: bool,
}

impl Value {
    /// A value opened by `quote`, with no byte.
    fn opened_by(quote: u8) -> Self {
        Self {
            quote,
            bytes: [0; 5],
            num_bytes: 0,
            is_after_spaces: false,
        }
    }

    /// The bytes of the value read.
    fn read_bytes(&self) -> &[u8] {
        self.bytes.get(..self.num_bytes).unwrap_or_default()
    }

    /// Where the bytes end once `byte` follows this value.
    fn after(self, byte: u8) -> State {
        if byte == self.quote {
            return if UTF_8_NAMES.contains(&self.read_bytes()) {
                State::start_of_declaration()
            } else {
                State::OtherEncoding
            };
        }
        if is_space(byte) {
            return State::InsideValue(Self {
                quote: self.quote,
                bytes: self.bytes,
                num_bytes: self.num_bytes,
                is_after_spaces: self.is_after_spaces || self.num_bytes > 0,
            });
        }
        if self.is_after_spaces {
            return State::OtherEncoding;
        }
        let value = {
            let mut bytes = self.bytes;
            let Some(slot) = bytes.get_mut(self.num_bytes) else {
                return State::OtherEncoding;
            };
            *slot = byte.to_ascii_lowercase();
            Self {
                quote: self.quote,
                bytes,
                num_bytes: self.num_bytes.saturating_add(1),
                is_after_spaces: false,
            }
        };
        if UTF_8_NAMES
            .iter()
            .any(|name| name.starts_with(value.read_bytes()))
        {
            State::InsideValue(value)
        } else {
            State::OtherEncoding
        }
    }
}

/// Whether `byte` is a space, a tab or a line break, what the rule trims.
fn is_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n')
}

#[cfg(test)]
mod tests {
    use crate::encoding::EncodingCheck;

    /// Whether `bytes`, given in reads of the lengths `split_lengths` and
    /// then the rest, show an encoding other than UTF-8.
    fn is_other_encoding_in_reads(bytes: &[u8], split_lengths: &[usize]) -> bool {
        let mut check = EncodingCheck::new();
        let mut rest = bytes;
        for split_length in split_lengths {
            let (read, after) = rest.split_at((*split_length).min(rest.len()));
            check.check(read);
            rest = after;
        }
        check.check(rest);
        check.is_other_encoding()
    }

    /// Asserts that `bytes` show an encoding other than UTF-8 or not, as
    /// `is_other_encoding` says, given whole, a byte at a time, and in two
    /// reads split at every byte.
    fn assert_found_at_every_split(bytes: &[u8], is_other_encoding: bool) {
        let text = String::from_utf8_lossy(bytes);
        assert_eq!(
            is_other_encoding_in_reads(bytes, &[]),
            is_other_encoding,
            "whole: {text}"
        );
        assert_eq!(
            is_other_encoding_in_reads(bytes, &vec![1; bytes.len()]),
            is_other_encoding,
            "a byte at a time: {text}"
        );
        for split in 0..=bytes.len() {
            assert_eq!(
                is_other_encoding_in_reads(bytes, &[split]),
                is_other_encoding,
                "split at {split}: {text}"
            );
        }
    }

    #[test]
    fn a_declaration_of_utf_8_is_utf_8() {
        for bytes in [
            &br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><a/>"#[..],
            br#"<?xml version="1.0" encoding='utf-8'?><a/>"#,
            br#"<?xml version="1.0" encoding = ' UTF8 '?><a/>"#,
            b"<?xml version=\"1.0\" encoding\r\n=\t\" utf-8\n\"?><a/>",
            br#"<?xml version="1.0"?><a/>"#,
            br#"<?xml?><a/>"#,
            b"<a/>",
            b"",
            b"<",
            b"\xEF\xBB\xBF<?xml version=\"1.0\" encoding=\"UTF-8\"?><a/>",
        ] {
            assert_found_at_every_split(bytes, false);
        }
    }

    #[test]
    fn a_declaration_of_another_encoding_is_found() {
        for bytes in [
            &br#"<?xml version="1.0" encoding="windows-1252"?><a/>"#[..],
            br#"<?xml version="1.0" encoding='ISO-8859-1'?><a/>"#,
            br#"<?xml encoding="UTF-16"?>"#,
            br#"<?xml version="1.0" encoding="unicode-1-1-utf-8"?>"#,
            b"\xEF\xBB\xBF<?xml version=\"1.0\" encoding=\"windows-1252\"?><a/>",
        ] {
            assert_found_at_every_split(bytes, true);
        }
    }

    // A value that is not utf-8 or utf8 once trimmed of spaces, tabs and
    // line breaks, or that cannot be read as a value in quotes before ?>.
    #[test]
    fn a_value_that_is_not_utf_8_or_cannot_be_read_is_found() {
        for bytes in [
            &br#"<?xml encoding=""?>"#[..],
            br#"<?xml encoding="utf"?>"#,
            br#"<?xml encoding="utf-88"?>"#,
            br#"<?xml encoding="utf -8"?>"#,
            br#"<?xml encoding="utf-8'?>"#,
            br#"<?xml encoding="utf-8?>"#,
            br#"<?xml encoding=utf-8?>"#,
            br#"<?xml encoding=?>"#,
            b"<?xml encoding=\"\x0Cutf-8\"?>",
        ] {
            assert_found_at_every_split(bytes, true);
        }
    }

    // Every encoding of every declaration is checked, one in the value of
    // another attribute or in a comment among them.
    #[test]
    fn every_encoding_of_every_declaration_is_checked() {
        for bytes in [
            &br#"<?xml version="1.0" foo="encoding='utf-8'" encoding="windows-1252"?>"#[..],
            br#"<?xml version="encoding=&quot;utf-8" encoding="windows-1252"?>"#,
            br#"<?xml encoding="utf-8" encoding="windows-1252"?>"#,
            br#"<?xml version="1.0" encoding="UTF-8"?><a><?xml version="1.0" encoding="windows-1252"?></a>"#,
            br#"<?xml version="1.0" encoding="UTF-8"?><a><!-- <?xml encoding="latin1"?> --></a>"#,
            br#"<a>x</a><?xml encoding="latin1"?>"#,
            b"<?xml\x0Cencoding=\"windows-1252\"?>",
            br#"<?xml-stylesheet encoding="latin1"?>"#,
            br#"<<?xml encoding="latin1"?>"#,
            br#"<?xm<?xml encoding="latin1"?>"#,
            br#"<?xml fooencoding="latin1"?>"#,
            br#"<?xml encoding x="1" encoding="latin1"?>"#,
            br#"<?xml version="1.0"encoding="windows-1252"?>"#,
        ] {
            assert_found_at_every_split(bytes, true);
        }
    }

    // What is not an encoding of a declaration: one after its ?>, one whose
    // name goes on or has no =, and one of a name written in capitals,
    // which quick-xml does not read either.
    #[test]
    fn an_encoding_outside_a_declaration_or_of_another_name_is_not_checked() {
        for bytes in [
            &br#"<?xml version="1.0"?><a encoding="windows-1252"/>"#[..],
            br#"<?xml version="1.0" encodingx="windows-1252"?>"#,
            br#"<?xml version="1.0" encoding?><a encoding="latin1"/>"#,
            br#"<?xml version="1.0" ENCODING="windows-1252"?>"#,
            br#"<?XML version="1.0" encoding="windows-1252"?>"#,
            br#"<? xml version="1.0" encoding="windows-1252"?>"#,
            br#"<?xml encoding="utf-8"?>?><a encoding="latin1"/>"#,
        ] {
            assert_found_at_every_split(bytes, false);
        }
    }

    #[test]
    fn a_part_that_starts_as_utf_16_is_found() {
        for bytes in [
            &b"\xFF\xFE<\x00?\x00x\x00"[..],
            b"\xFE\xFF\x00<\x00?",
            b"\x00<\x00?\x00x",
            b"<\x00?\x00x\x00",
            b"<\x00a\x00",
            b"\x00<",
        ] {
            assert_found_at_every_split(bytes, true);
        }
    }

    // Only the first two bytes are a mark of UTF-16.
    #[test]
    fn the_bytes_of_utf_16_after_the_start_are_not_found() {
        for bytes in [
            &b"<a>\xFF\xFE</a>"[..],
            b"<a>\x00<\x00</a>",
            b"\xFF",
            b"\x00",
        ] {
            assert_found_at_every_split(bytes, false);
        }
    }

    // A value of version of 1,000,000 bytes, or 1,000,000 spaces before =,
    // hid windows-1252 from a check that looked within a window.
    #[test]
    fn an_encoding_at_any_distance_from_its_declaration_is_found() {
        let long_version = [
            &br#"<?xml version=""#[..],
            &vec![b'1'; 1_000_000],
            br#"" encoding="windows-1252"?>"#,
        ]
        .concat();
        let long_spaces = [
            &br#"<?xml version="1.0" encoding"#[..],
            &vec![b' '; 1_000_000],
            br#"="windows-1252"?>"#,
        ]
        .concat();
        for bytes in [long_version, long_spaces] {
            for split_lengths in [
                &[][..],
                &[8],
                &[500_000],
                &[bytes.len() - 20],
                &[3, 4_096, 65_536, 1_000_000],
            ] {
                assert!(is_other_encoding_in_reads(&bytes, split_lengths));
            }
        }
    }
}
