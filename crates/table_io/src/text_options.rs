//! The options of a text file, a CSV or a TSV, that the caller gives with
//! each import, and what the import reports it read the file with
//! (`docs/specs/text-files.md`, "The Rust interface"). They are behind no
//! feature, since the import takes them whatever the file is and whichever
//! formats a build reads.

use crate::DecimalMark;

/// An encoding a caller can set for a text file. UTF-16 is not one: a file
/// is read as UTF-16 when it starts with its mark, whatever is set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    /// UTF-8.
    Utf8,
    /// Windows-1252, what Excel on Windows writes for "CSV" in Spanish.
    Windows1252,
}

/// An encoding a text file can be read with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoundEncoding {
    /// UTF-8.
    Utf8,
    /// UTF-16, little or big endian, found by its mark at the start of the
    /// file.
    Utf16,
    /// Windows-1252.
    Windows1252,
}

/// The character between the cells of a row of a text file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Separator {
    /// The tab, as in a TSV.
    Tab,
    /// `;`, which a Spanish Excel writes.
    Semicolon,
    /// `,`.
    Comma,
}

/// What the caller sets for a text file; each option that is `None` is
/// found from the file, which `Default` gives for all three.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextOptions {
    /// The encoding of the file.
    pub encoding: Option<Encoding>,
    /// The character between the cells.
    pub separator: Option<Separator>,
    /// The mark between the whole part and the decimals of a number.
    pub decimal: Option<DecimalMark>,
}

/// How a text file was read: each option as the caller set it or as the
/// import found it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextRead {
    /// The encoding the file was decoded with.
    pub encoding: FoundEncoding,
    /// The character between the cells.
    pub separator: Separator,
    /// The mark between the whole part and the decimals of a number.
    pub decimal: DecimalMark,
    /// The line of the first character that could not be decoded, and
    /// stands as U+FFFD, `�`, or of the first U+FFFD the file itself holds,
    /// counted from 1; `None` when the text has no U+FFFD.
    pub undecoded_line: Option<u32>,
}
