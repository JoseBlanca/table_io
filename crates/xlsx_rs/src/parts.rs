//! The parts of the zip xlsx_rs reads itself before calamine, as "What
//! xlsx_rs reads before calamine" of `docs/specs/read.md` gives them.
//!
//! An xlsx is a zip of parts, each an XML file: the workbook, which lists
//! the sheets and holds the settings of the file, the date system among
//! them; the relationships, which give where the other parts are; the table
//! of texts, which holds once each text the cells hold; the styles; and
//! the sheets. Each part carries a checksum of its bytes, which the zip
//! crate checks only once the part has been read to its end, and calamine
//! stops reading a sheet at its last cell; so xlsx_rs reads every part to
//! its end first, counting its bytes. calamine also holds four parts whole
//! when it opens the file, and every merged range of the sheet before
//! xlsx_rs sees one. xlsx_rs bounds, in that one reading, at least what
//! calamine could hold of each: it does not copy calamine's reading, since
//! every small difference between two readings let a file past a bound in
//! the review of 28 September 2026. The one part it finds as calamine
//! finds it is the workbook, for its date system.

use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Cursor, Read};

use quick_xml::Reader as XmlReader;
use quick_xml::events::attributes::AttrError;
use quick_xml::events::{BytesStart, Event};
use zip::ZipArchive;
use zip::result::ZipError;

use crate::ReadError;
use crate::attrs::{RawAttrIter, attribute_of, attributes_of};
use crate::date::DateSystem;

/// The bounds of the parts of a read, which a test lowers so as to pass
/// them with a file of a few KB.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PartBounds {
    /// The most bytes the parts may hold together once unzipped,
    /// [`crate::MAX_UNZIPPED_BYTES`] outside the tests.
    pub(crate) max_unzipped_bytes: u64,
    /// The most bytes each settings part, a part calamine reads whole other
    /// than a table of texts, may hold once unzipped,
    /// [`crate::MAX_SETTINGS_PART_BYTES`] outside the tests.
    pub(crate) max_settings_part_bytes: u64,
    /// The most bytes each table of texts may hold once unzipped,
    /// [`crate::MAX_TEXT_TABLE_BYTES`] outside the tests.
    pub(crate) max_text_table_bytes: u64,
    /// The most texts, and the largest `uniqueCount`, each table of texts
    /// may hold, [`crate::MAX_TEXTS`] outside the tests.
    pub(crate) max_texts: u64,
    /// The most bytes the paths of the sheets may be counted at,
    /// [`crate::MAX_SHEET_PATH_BYTES`] outside the tests.
    pub(crate) max_sheet_path_bytes: u64,
}

/// Reads every part of the zip `bytes` to its end, so that the zip crate
/// checks its checksum, and bounds what calamine could hold of each within
/// `bounds`, and the merged ranges of each within `max_cells`; then gives
/// the date system of the workbook. The zip is let go of before it
/// returns, so that calamine does not open the file while xlsx_rs holds
/// the list of its parts.
///
/// A workbook named by the package relationships but missing is given the
/// system of 1900; calamine then finds no sheet.
///
/// # Errors
///
/// [`ReadError::Unreadable`] with the zip crate's message for any error of
/// the zip, "Invalid checksum" for a part whose bytes are not those it was
/// saved with; "the file names no workbook" when the package
/// relationships, `_rels/.rels`, are missing or name no workbook, which
/// calamine refuses too; "the file unzips to more than … bytes" when the parts hold
/// more than `bounds.max_unzipped_bytes` bytes together; "a part of the
/// file is too large", "too much text" and "too many texts", for the
/// bounds of the parts calamine reads whole ("What xlsx_rs reads before
/// calamine", point 3); "a part of the file is not in UTF-8" for one of
/// them in another encoding (point 3); "the workbook lists too many sheets" when the
/// paths of the sheets are counted past `bounds.max_sheet_path_bytes`
/// (point 3); "too many merged ranges" for a part with more
/// merged ranges than `max_cells` (point 4); and "the part … cannot be read
/// as XML: …", with quick-xml's message, for package relationships or a
/// workbook whose XML cannot be read up to what xlsx_rs takes of it.
pub(crate) fn read_parts(
    bytes: &[u8],
    max_cells: u32,
    bounds: PartBounds,
) -> Result<DateSystem, ReadError> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(unreadable_of_zip_error)?;
    read_every_part(&mut archive, max_cells, bounds)?;
    let part_names = PartNames::of_archive(&archive);
    let Some(workbook_folder) = workbook_folder_of(&mut archive, &part_names)? else {
        return Err(ReadError::Unreadable(
            "the file names no workbook".to_owned(),
        ));
    };
    let workbook_path = format!("{workbook_folder}workbook.xml");
    date_system_of(&mut archive, &part_names, &workbook_path)
}

/// The zip of an xlsx held in memory.
type Archive<'bytes> = ZipArchive<Cursor<&'bytes [u8]>>;

/// The path of the package relationships, the part that gives the folder
/// of the workbook.
const PACKAGE_RELATIONSHIPS_PATH: &str = "_rels/.rels";

/// What calamine may read a part as, told by the end of its name, `\\`
/// read as `/` and ignoring case, in any folder: the part calamine reads is
/// always one of them, since its name is the folder of the workbook
/// followed by the usual name ("What xlsx_rs reads before calamine" of
/// `docs/specs/read.md`, point 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PartKind {
    /// `_rels/.rels`, the package relationships, which give the folder of
    /// the workbook.
    PackageRelationships,
    /// A name ending in `workbook.xml`: the workbook, which lists the
    /// sheets and the defined names.
    Workbook,
    /// A name ending in `workbook.xml.rels`: the relationships of the
    /// workbook, which give the part of each sheet.
    WorkbookRelationships,
    /// A name ending in `styles.xml`: the styles, where the formats of the
    /// numbers are.
    Styles,
    /// A name ending in `sharedStrings.xml`: a table of texts.
    TextTable,
    /// Any other part, a sheet among them.
    Other,
}

impl PartKind {
    /// The kind of the part named `zip_name` in the zip.
    fn of_zip_name(zip_name: &str) -> Self {
        let name = zip_name.replace('\\', "/").to_ascii_lowercase();
        if name == PACKAGE_RELATIONSHIPS_PATH {
            Self::PackageRelationships
        } else if name.ends_with("workbook.xml") {
            Self::Workbook
        } else if name.ends_with("workbook.xml.rels") {
            Self::WorkbookRelationships
        } else if name.ends_with("styles.xml") {
            Self::Styles
        } else if name.ends_with("sharedstrings.xml") {
            Self::TextTable
        } else {
            Self::Other
        }
    }

    /// The most bytes a part of this kind may hold unzipped, within
    /// `bounds`, and the message of the error past it; `None` for a part
    /// calamine does not hold whole.
    fn bound_of_bytes(self, bounds: &PartBounds) -> Option<PartBound> {
        match self {
            Self::PackageRelationships
            | Self::Workbook
            | Self::WorkbookRelationships
            | Self::Styles => Some(PartBound {
                max_bytes: bounds.max_settings_part_bytes,
                message: "a part of the file is too large",
            }),
            Self::TextTable => Some(PartBound {
                max_bytes: bounds.max_text_table_bytes,
                message: "too much text",
            }),
            Self::Other => None,
        }
    }
}

/// The most bytes a part may hold unzipped, and the message of the error
/// past it.
#[derive(Debug, Clone, Copy)]
struct PartBound {
    /// The most bytes.
    max_bytes: u64,
    /// The message of [`ReadError::Unreadable`] past them.
    message: &'static str,
}

/// The bytes the parts of a zip hold unzipped, counted as they are read.
struct UnzippedBytes {
    /// The bytes read so far, of every part.
    num_bytes: u64,
    /// The most they may be.
    max_bytes: u64,
}

/// A part of the zip as it is unzipped, its bytes counted as they pass,
/// and the first failure kept: an error of the zip, or a bound passed.
///
/// Once it has failed, it gives its error at each read, so that a reader
/// of XML over it stops, and the failure is taken from it rather than from
/// the reader of XML.
struct PartReader<'unzipped, Part> {
    /// The part, as the zip crate unzips it.
    part: Part,
    /// The bound of the bytes of this part, if it has one.
    bound: Option<PartBound>,
    /// The bytes of this part read so far.
    num_bytes: u64,
    /// The bytes of every part read so far.
    unzipped: &'unzipped mut UnzippedBytes,
    /// The merged ranges of this part, the bytes `mergeCell` right after
    /// `<` or `:`, counted so far.
    merged_ranges: ElementCounter,
    /// The most merged ranges a part may hold, `max_cells`.
    max_merged_ranges: u64,
    /// The sheets of this part, the bytes `sheet` right after `<` or `:`,
    /// counted so far in a workbook; `None` for another part.
    sheets: Option<ElementCounter>,
    /// The first failure, `None` while there is none.
    failure: Option<ReadError>,
}

impl<'unzipped, Part: Read> PartReader<'unzipped, Part> {
    /// A reader of `part`, whose bytes are held to `bound` and counted in
    /// `unzipped`, and whose merged ranges are held to `max_merged_ranges`.
    fn new(
        part: Part,
        bound: Option<PartBound>,
        unzipped: &'unzipped mut UnzippedBytes,
        max_merged_ranges: u64,
    ) -> Self {
        Self {
            part,
            bound,
            num_bytes: 0,
            unzipped,
            merged_ranges: ElementCounter::of_name(MERGED_RANGE_NAME),
            max_merged_ranges,
            sheets: None,
            failure: None,
        }
    }

    /// The same reader, which also counts the sheets of its part, a
    /// workbook.
    fn counting_sheets(mut self) -> Self {
        self.sheets = Some(ElementCounter::of_name(SHEET_NAME));
        self
    }

    /// The sheets counted so far, 0 in a part whose sheets are not counted.
    fn num_sheets(&self) -> u64 {
        self.sheets.as_ref().map_or(0, |sheets| sheets.num_elements)
    }

    /// What was counted in the part, once `read_result`, what the reading
    /// of the part to its end gave, is known.
    ///
    /// # Errors
    ///
    /// The failure of the part when it has one, and otherwise the error of
    /// `read_result`.
    fn finish<Copied>(self, read_result: io::Result<Copied>) -> Result<PartCounts, ReadError> {
        if let Some(failure) = self.failure {
            return Err(failure);
        }
        read_result.map_err(|io_error| ReadError::Unreadable(io_error.to_string()))?;
        Ok(PartCounts {
            num_sheets: self.num_sheets(),
        })
    }

    /// Keeps `failure` as the failure of the part, and gives the error of
    /// a read for it.
    fn fail(&mut self, failure: ReadError) -> io::Error {
        let io_error = io::Error::other(format!("{failure:?}"));
        self.failure = Some(failure);
        io_error
    }
}

impl<Part: Read> Read for PartReader<'_, Part> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if let Some(failure) = &self.failure {
            return Err(io::Error::other(format!("{failure:?}")));
        }
        let num_read = match self.part.read(buffer) {
            Ok(num_read) => num_read,
            Err(zip_io_error) => {
                return Err(self.fail(ReadError::Unreadable(zip_io_error.to_string())));
            }
        };
        let num_read_bytes = u64::try_from(num_read).unwrap_or(u64::MAX);
        self.num_bytes = self.num_bytes.saturating_add(num_read_bytes);
        self.unzipped.num_bytes = self.unzipped.num_bytes.saturating_add(num_read_bytes);
        if self.unzipped.num_bytes > self.unzipped.max_bytes {
            let message = format!(
                "the file unzips to more than {} bytes",
                text_of_count(self.unzipped.max_bytes)
            );
            return Err(self.fail(ReadError::Unreadable(message)));
        }
        if let Some(bound) = self.bound
            && self.num_bytes > bound.max_bytes
        {
            return Err(self.fail(ReadError::Unreadable(bound.message.to_owned())));
        }
        let bytes_read = buffer.get(..num_read).unwrap_or_default();
        self.merged_ranges.count_in(bytes_read);
        if let Some(sheets) = &mut self.sheets {
            sheets.count_in(bytes_read);
        }
        if self.merged_ranges.num_elements > self.max_merged_ranges {
            return Err(self.fail(ReadError::Unreadable("too many merged ranges".to_owned())));
        }
        Ok(num_read)
    }
}

/// What the reader of a part counted in it, besides what it bounds as it
/// reads.
struct PartCounts {
    /// The bytes `sheet` right after `<` or `:`, counted in a workbook, 0
    /// in another part.
    num_sheets: u64,
}

/// The nine bytes every element `mergeCell`, a merged range, has at the
/// start of its name, with a prefix or without.
const MERGED_RANGE_NAME: &[u8] = b"mergeCell";

/// The five bytes every element `sheet` of a workbook has at the start of
/// its name, with a prefix or without.
const SHEET_NAME: &[u8] = b"sheet";

/// What xlsx_rs counts of the paths of the sheets calamine keeps: for each
/// sheet the workbook lists, the folder of the workbook, taken from a tag
/// of `_rels/.rels`, followed by the target of the sheet's relationship, in
/// a tag of the relationships of the workbook; every sheet may name the
/// same relationship ("What xlsx_rs reads before calamine" of
/// `docs/specs/read.md`, point 3).
#[derive(Debug, Default)]
struct SheetPaths {
    /// The largest count of the bytes `sheet` right after `<` or `:` in a
    /// part whose name ends in `workbook.xml`.
    largest_num_sheets: u64,
    /// The two longest tags, from `<` to `>`, of the parts named
    /// `_rels/.rels` or ending in `workbook.xml.rels`, the longest of each
    /// part, in bytes; the longer first.
    longest_tags: [u64; 2],
}

impl SheetPaths {
    /// Counts a workbook of `num_sheets` sheets.
    fn add_workbook(&mut self, num_sheets: u64) {
        self.largest_num_sheets = self.largest_num_sheets.max(num_sheets);
    }

    /// Counts a part of relationships whose longest tag holds
    /// `num_tag_bytes` bytes.
    fn add_relationships(&mut self, num_tag_bytes: u64) {
        let [longest, second_longest] = self.longest_tags;
        self.longest_tags = if num_tag_bytes > longest {
            [num_tag_bytes, longest]
        } else {
            [longest, second_longest.max(num_tag_bytes)]
        };
    }

    /// The bytes the paths are counted at: the largest count of sheets
    /// times the sum of the two longest tags.
    fn num_bytes(&self) -> u64 {
        let [longest, second_longest] = self.longest_tags;
        self.largest_num_sheets
            .saturating_mul(longest.saturating_add(second_longest))
    }
}

/// A count of the times the bytes of a name of an element come right after
/// `<` or `:`, in bytes given a read at a time, a match split between two
/// reads counted too, and nothing of them held ("What xlsx_rs reads before
/// calamine" of `docs/specs/read.md`, point 4).
///
/// Every element of that name, with a prefix or without, has those bytes
/// at the start of its name, since quick-xml reads the bytes of a part as
/// they are; so the count is at least the elements calamine reads. It is
/// more for a longer name that starts with them, `mergeCells`, an end tag
/// with a prefix, `</x:mergeCell>`, and the bytes written as text.
struct ElementCounter {
    /// The bytes of the name.
    name: &'static [u8],
    /// The bytes of the name matched at the end of the bytes given so far,
    /// after a `<` or a `:`; `None` when the last bytes given start no
    /// match.
    num_matched: Option<usize>,
    /// The times the name was found.
    num_elements: u64,
}

impl ElementCounter {
    /// A count, at 0, of the element `name`, which holds neither `<` nor
    /// `:`.
    fn of_name(name: &'static [u8]) -> Self {
        Self {
            name,
            num_matched: None,
            num_elements: 0,
        }
    }

    /// Counts the name in `bytes`, the bytes that follow those given before.
    fn count_in(&mut self, bytes: &[u8]) {
        let mut rest = bytes;
        loop {
            let Some(num_matched) = self.num_matched else {
                let Some(mark_position) = rest.iter().position(|byte| is_name_mark(*byte)) else {
                    return;
                };
                rest = rest
                    .get(mark_position.saturating_add(1)..)
                    .unwrap_or_default();
                self.num_matched = Some(0);
                continue;
            };
            let name_left = self.name.get(num_matched..).unwrap_or_default();
            if let Some(after_name) = rest.strip_prefix(name_left) {
                self.num_elements = self.num_elements.saturating_add(1);
                self.num_matched = None;
                rest = after_name;
            } else if name_left.starts_with(rest) {
                // The bytes end inside the name: the next ones go on with it.
                self.num_matched = Some(num_matched.saturating_add(rest.len()));
                return;
            } else {
                // No match here; the name holds no mark, so the next match
                // starts at the next mark, which may be the first byte left.
                self.num_matched = None;
            }
        }
    }
}

/// Whether `byte` is one a name of an element comes right after: `<`, or
/// `:` after a prefix.
fn is_name_mark(byte: u8) -> bool {
    byte == b'<' || byte == b':'
}

/// Reads every part of `archive` to its end and discards what it reads, so
/// that the zip crate checks the checksum of each, counting the bytes of
/// each and of all within `bounds`, the texts of each table of texts, and
/// the merged ranges of each part within `max_cells`.
fn read_every_part(
    archive: &mut Archive<'_>,
    max_cells: u32,
    bounds: PartBounds,
) -> Result<(), ReadError> {
    let mut unzipped = UnzippedBytes {
        num_bytes: 0,
        max_bytes: bounds.max_unzipped_bytes,
    };
    let mut sheet_paths = SheetPaths::default();
    for part_index in 0..archive.len() {
        let part = archive
            .by_index(part_index)
            .map_err(unreadable_of_zip_error)?;
        let part_kind = PartKind::of_zip_name(part.name());
        let mut part_reader = PartReader::new(
            part,
            part_kind.bound_of_bytes(&bounds),
            &mut unzipped,
            u64::from(max_cells),
        );
        match part_kind {
            PartKind::TextTable => {
                read_text_table(part_reader, bounds.max_texts)?;
            }
            PartKind::PackageRelationships | PartKind::WorkbookRelationships => {
                sheet_paths.add_relationships(read_longest_tag(part_reader)?);
            }
            PartKind::Workbook => {
                let part_counts = read_xml_part(part_reader.counting_sheets(), |_, _| Ok(()))?;
                sheet_paths.add_workbook(part_counts.num_sheets);
            }
            PartKind::Styles => {
                read_xml_part(part_reader, |_, _| Ok(()))?;
            }
            PartKind::Other => {
                let copy_result = io::copy(&mut part_reader, &mut io::sink());
                part_reader.finish(copy_result)?;
            }
        }
    }
    if sheet_paths.num_bytes() > bounds.max_sheet_path_bytes {
        return Err(ReadError::Unreadable(
            "the workbook lists too many sheets".to_owned(),
        ));
    }
    Ok(())
}

/// Reads the XML of the part that `part_reader` unzips with quick-xml,
/// configured as calamine configures it, to the end of the part or to its
/// first error of XML, giving each event to `on_event` with the bytes it
/// took; then reads the rest of the part to its end, and gives what its
/// reader counted. An error of XML is not an error here: calamine refuses
/// a file at one before the end of what it reads of the part, and never
/// reads one after it.
///
/// # Errors
///
/// The error `on_event` gives, when it gives one, at once; the failure of
/// `part_reader`, an error of the zip or a bound of its bytes passed; and
/// [`ReadError::Unreadable`], "a part of the file is not in UTF-8", when
/// quick-xml decodes the part with another encoding, from a byte order mark
/// at its start or from the `encoding` of a declaration `<?xml … ?>`
/// ("What xlsx_rs reads before calamine" of `docs/specs/read.md`, point
/// 3). calamine decodes the texts of a part in the encoding it declares,
/// where a byte of windows-1252 may be 3 bytes of UTF-8, so every bound
/// counted in the bytes of the part would hold 3 times as much.
fn read_xml_part<Part: Read>(
    part_reader: PartReader<'_, Part>,
    mut on_event: impl FnMut(&Event<'_>, u64) -> Result<(), ReadError>,
) -> Result<PartCounts, ReadError> {
    let mut xml_reader = xml_reader_over(BufReader::new(part_reader));
    let mut buffer = Vec::new();
    loop {
        buffer.clear();
        let event_start = xml_reader.buffer_position();
        match xml_reader.read_event_into(&mut buffer) {
            Ok(Event::Eof) | Err(_) => break,
            Ok(event) => {
                let event_bytes = xml_reader.buffer_position().saturating_sub(event_start);
                on_event(&event, event_bytes)?;
            }
        }
    }
    let is_utf_8 = xml_reader.decoder().encoding().name() == "UTF-8";
    let mut buffered_part = xml_reader.into_inner();
    let copy_result = io::copy(&mut buffered_part, &mut io::sink());
    let part_counts = buffered_part.into_inner().finish(copy_result)?;
    if !is_utf_8 {
        return Err(ReadError::Unreadable(
            "a part of the file is not in UTF-8".to_owned(),
        ));
    }
    Ok(part_counts)
}

/// Reads the part of relationships that `part_reader` unzips as
/// [`read_xml_part`] does, and gives the bytes of its longest tag of an
/// element, from `<` to `>`, since a target and the folder calamine takes
/// from one are inside one tag.
///
/// # Errors
///
/// Those of [`read_xml_part`].
fn read_longest_tag<Part: Read>(part_reader: PartReader<'_, Part>) -> Result<u64, ReadError> {
    let mut longest_tag: u64 = 0;
    read_xml_part(part_reader, |event, event_bytes| {
        match event {
            Event::Start(_) | Event::Empty(_) | Event::End(_) => {
                longest_tag = longest_tag.max(event_bytes);
            }
            Event::Text(_)
            | Event::CData(_)
            | Event::Comment(_)
            | Event::Decl(_)
            | Event::PI(_)
            | Event::DocType(_)
            | Event::GeneralRef(_)
            | Event::Eof => {}
        }
        Ok(())
    })?;
    Ok(longest_tag)
}

/// The names of the parts of a zip, as calamine matches a name it looks
/// for against them (`build_zip_path_cache` and `cached_zip_path` of its
/// `utils.rs`): ignoring case, with `\` in a name of the zip read as `/`.
struct PartNames {
    /// Each name of the zip, with `\` turned into `/` and in lower case,
    /// and the name as the zip has it; of two names that are the same so,
    /// the later in the zip.
    zip_name_of_lower_case: HashMap<String, String>,
}

impl PartNames {
    /// The names of the parts of `archive`.
    fn of_archive(archive: &Archive<'_>) -> Self {
        let zip_name_of_lower_case = archive
            .file_names()
            .map(|zip_name| {
                (
                    zip_name.replace('\\', "/").to_ascii_lowercase(),
                    zip_name.to_owned(),
                )
            })
            .collect();
        Self {
            zip_name_of_lower_case,
        }
    }

    /// The name in the zip of the part calamine opens for `path`: the name
    /// that matches it ignoring case, or `path` itself when none does.
    fn zip_name_of<'names>(&'names self, path: &'names str) -> &'names str {
        self.zip_name_of_lower_case
            .get(&path.to_ascii_lowercase())
            .map_or(path, String::as_str)
    }
}

/// A reader of the XML of the part `path`, configured as calamine's
/// `xml_reader` configures its own, or `None` when the zip has no such part.
fn xml_reader_of<'archive>(
    archive: &'archive mut Archive<'_>,
    part_names: &PartNames,
    path: &str,
) -> Result<Option<XmlReader<impl BufRead + 'archive>>, ReadError> {
    let part = match archive.by_name(part_names.zip_name_of(path)) {
        Ok(part) => part,
        Err(ZipError::FileNotFound) => return Ok(None),
        Err(zip_error) => return Err(unreadable_of_zip_error(zip_error)),
    };
    Ok(Some(xml_reader_over(BufReader::new(part))))
}

/// A reader of the XML of `part`, configured as calamine's `xml_reader`
/// configures its own, so that the two meet the same elements.
fn xml_reader_over<Part: BufRead>(part: Part) -> XmlReader<Part> {
    let mut xml_reader = XmlReader::from_reader(part);
    let config = xml_reader.config_mut();
    config.check_end_names = false;
    config.trim_text(false);
    config.check_comments = false;
    config.expand_empty_elements = true;
    xml_reader
}

/// The folder of the workbook, such as `xl/`, as calamine's
/// `read_package_relationships` finds it ("What xlsx_rs reads before
/// calamine" of `docs/specs/read.md`, point 2): the elements of local name
/// `Relationship` after the first start of `Relationships` and before the
/// next end of an element of that local name; in each, `Type` and `Target`
/// read as calamine's `get_attrs!` reads them; and the target, its entities
/// read, of the last whose type ends in `/relationships/officeDocument` and
/// that has a target, up to its last `/`, with no `/` at its start. `None`
/// when there is no such part or relationship.
fn workbook_folder_of(
    archive: &mut Archive<'_>,
    part_names: &PartNames,
) -> Result<Option<String>, ReadError> {
    let Some(mut xml_reader) = xml_reader_of(archive, part_names, PACKAGE_RELATIONSHIPS_PATH)?
    else {
        return Ok(None);
    };
    let unreadable =
        |cause: &dyn std::fmt::Display| unreadable_xml_error_of(PACKAGE_RELATIONSHIPS_PATH, cause);
    let mut buffer = Vec::new();
    loop {
        buffer.clear();
        match xml_reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) if element.local_name().as_ref() == b"Relationships" => {
                break;
            }
            Ok(Event::Eof) => return Ok(None),
            Err(xml_error) => return Err(unreadable(&xml_error)),
            Ok(_) => {}
        }
    }
    let mut document_target: Option<String> = None;
    loop {
        buffer.clear();
        match xml_reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) if element.local_name().as_ref() == b"Relationship" => {
                let RawRelationship {
                    raw_type,
                    raw_target,
                } = raw_relationship_of(&element).map_err(|xml_error| unreadable(&xml_error))?;
                let is_office_document = raw_type
                    .is_some_and(|raw_type| raw_type.ends_with(b"/relationships/officeDocument"));
                if is_office_document && let Some(raw_target) = raw_target {
                    let decoded_target = xml_reader
                        .decoder()
                        .decode(&raw_target)
                        .map_err(|encoding_error| unreadable(&encoding_error))?;
                    let target = quick_xml::escape::unescape(&decoded_target)
                        .map_err(|escape_error| unreadable(&escape_error))?;
                    document_target = Some(target.into_owned());
                }
            }
            Ok(Event::End(element)) if element.local_name().as_ref() == b"Relationships" => break,
            Ok(Event::Eof) => break,
            Err(xml_error) => return Err(unreadable(&xml_error)),
            Ok(_) => {}
        }
    }
    Ok(document_target.map(|target| {
        let folder = target
            .rfind('/')
            .and_then(|last_slash| target.get(..=last_slash))
            .unwrap_or("");
        folder.strip_prefix('/').unwrap_or(folder).to_owned()
    }))
}

/// The attributes `Type` and `Target` of a relationship, as the XML
/// writes them, before their entities are read; either may be missing.
struct RawRelationship {
    /// The type of the relationship, a URL whose last part says what the
    /// target is, `officeDocument` for the workbook.
    raw_type: Option<Vec<u8>>,
    /// The path of the part it names.
    raw_target: Option<Vec<u8>>,
}

/// The attributes `Type` and `Target` of `element`, a relationship, read
/// as calamine's `get_attrs!` reads them.
fn raw_relationship_of(element: &BytesStart<'_>) -> Result<RawRelationship, AttrError> {
    let [raw_type, raw_target] = attributes_of(element, [b"Type", b"Target"])?;
    Ok(RawRelationship {
        raw_type: raw_type.map(<[u8]>::to_vec),
        raw_target: raw_target.map(<[u8]>::to_vec),
    })
}

/// Reads the table of texts that `part_reader` unzips as [`read_xml_part`]
/// does ("What xlsx_rs reads before calamine" of `docs/specs/read.md`,
/// point 3). It counts every element of local name
/// `si`, each a text, those inside another `si` and outside the root among
/// them, and reads every attribute `uniqueCount` of every element of local
/// name `sst`, the number of texts the table says it holds, for which
/// calamine reserves room when it meets the first `sst`. calamine counts
/// fewer: the `si` of the first `sst`, not those inside another, and
/// reserves room for the first `uniqueCount` of the first `sst`.
///
/// # Errors
///
/// [`ReadError::Unreadable`]: "too many texts" at the first `si` past
/// `max_texts`, or at a `uniqueCount` past it; and those of
/// [`read_xml_part`].
fn read_text_table<Part: Read>(
    part_reader: PartReader<'_, Part>,
    max_texts: u64,
) -> Result<(), ReadError> {
    let mut num_texts: u64 = 0;
    read_xml_part(part_reader, |event, _| {
        let Event::Start(element) = event else {
            return Ok(());
        };
        match element.local_name().as_ref() {
            b"si" => {
                num_texts = num_texts.saturating_add(1);
                if num_texts > max_texts {
                    return Err(too_many_texts());
                }
            }
            b"sst" if is_unique_count_past(element, max_texts) => {
                return Err(too_many_texts());
            }
            _ => {}
        }
        Ok(())
    })?;
    Ok(())
}

/// The error of a table of texts past [`crate::MAX_TEXTS`].
fn too_many_texts() -> ReadError {
    ReadError::Unreadable("too many texts".to_owned())
}

/// Whether an attribute `uniqueCount` of `sst`, an element of local name
/// `sst`, is a number past `max_texts`, its attributes read with
/// [`RawAttrIter`] up to the first it cannot read. A `uniqueCount` is a
/// number when it is one or more digits and nothing else, as calamine
/// reads it with `atoi_simd::parse::<usize, true, false>`, which takes any
/// zeros first and no sign; it is compared whatever its size, so one past
/// 4,294,967,295, which calamine in the wasm cannot read, is past too.
fn is_unique_count_past(sst: &BytesStart<'_>, max_texts: u64) -> bool {
    RawAttrIter::of_element(sst)
        .map_while(Result::ok)
        .filter(|(key, _)| *key == b"uniqueCount")
        .any(|(_, digits)| is_number_past(digits, max_texts))
}

/// Whether `digits` are one or more ASCII digits and nothing else, and
/// the number they write, any zeros first left out, is past `max_number`.
fn is_number_past(digits: &[u8], max_number: u64) -> bool {
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return false;
    }
    let significant_digits = digits
        .iter()
        .position(|digit| *digit != b'0')
        .and_then(|first_significant| digits.get(first_significant..))
        .unwrap_or_default();
    // A number of 20 digits or more passes any u64.
    if significant_digits.len() >= 20 {
        return true;
    }
    let number = significant_digits.iter().fold(0_u64, |number, digit| {
        number
            .saturating_mul(10)
            .saturating_add(u64::from(digit.saturating_sub(b'0')))
    });
    number > max_number
}

/// The date system of the workbook `path` ("What xlsx_rs reads before
/// calamine" of `docs/specs/read.md`, point 2): that of the last element
/// `workbookPr` that is a direct child of its root element, whatever the
/// namespace of either, 1904 when its attribute `date1904` is `1` or
/// `true`, as calamine reads that attribute; 1900 when there is no such
/// element or no such part. calamine takes the last element of that name
/// anywhere in the workbook, and Excel 365 writes one of the namespace
/// `x15` inside `extLst`, after the one of the root, with no `date1904`:
/// every workbook of 1904 Excel 365 saves would be read by calamine as one
/// of 1900.
///
/// The workbook is read up to the end of its root element, found by
/// counting the elements open.
///
/// # Errors
///
/// [`ReadError::Unreadable`]: "the part … cannot be read as XML: …", with
/// quick-xml's message, for a workbook whose XML cannot be read up to the
/// end of its root, and the zip crate's message for an error of the zip.
fn date_system_of(
    archive: &mut Archive<'_>,
    part_names: &PartNames,
    path: &str,
) -> Result<DateSystem, ReadError> {
    let Some(mut xml_reader) = xml_reader_of(archive, part_names, path)? else {
        return Ok(DateSystem::Excel1900);
    };
    let unreadable = |cause: &dyn std::fmt::Display| unreadable_xml_error_of(path, cause);
    let mut date_system = DateSystem::Excel1900;
    // The number of elements open: 1 inside the root, whose children are
    // the elements met at that depth.
    let mut depth: u32 = 0;
    let mut buffer = Vec::new();
    loop {
        buffer.clear();
        match xml_reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) => {
                if depth == 1 && element.local_name().as_ref() == b"workbookPr" {
                    date_system = date_system_of_element(&element)
                        .map_err(|xml_error| unreadable(&xml_error))?;
                }
                depth = depth.saturating_add(1);
            }
            Ok(Event::End(_)) => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(xml_error) => return Err(unreadable(&xml_error)),
            Ok(_) => {}
        }
    }
    Ok(date_system)
}

/// The date system `workbook_pr`, an element `workbookPr`, gives: 1904 when
/// its attribute `date1904` is `1` or `true`, read as calamine reads it, up
/// to the first attribute of that name.
fn date_system_of_element(workbook_pr: &BytesStart<'_>) -> Result<DateSystem, AttrError> {
    let is_1904 = matches!(
        attribute_of(workbook_pr, b"date1904")?,
        Some(b"1" | b"true")
    );
    Ok(if is_1904 {
        DateSystem::Excel1904
    } else {
        DateSystem::Excel1900
    })
}

/// The error of a part whose XML cannot be read, with `cause`.
fn unreadable_xml_error_of(path: &str, cause: &dyn std::fmt::Display) -> ReadError {
    ReadError::Unreadable(format!("the part {path} cannot be read as XML: {cause}"))
}

/// The error of xlsx_rs for an error of the zip crate: the file unreadable,
/// with the zip crate's message.
fn unreadable_of_zip_error(zip_error: ZipError) -> ReadError {
    ReadError::Unreadable(zip_error.to_string())
}

/// `count` written with a comma between each group of three digits, as
/// the messages of xlsx_rs write a number: `1,000,000,000`.
fn text_of_count(count: u64) -> String {
    let digits = count.to_string();
    let num_digits = digits.len();
    let mut text = String::with_capacity(num_digits.saturating_mul(2));
    for (index, digit) in digits.chars().enumerate() {
        let num_digits_after = num_digits.saturating_sub(index);
        if index > 0 && num_digits_after.is_multiple_of(3) {
            text.push(',');
        }
        text.push(digit);
    }
    text
}

#[cfg(test)]
mod tests {
    use crate::parts::{ElementCounter, MERGED_RANGE_NAME, text_of_count};

    /// The count of merged ranges in `bytes` given in two reads, split at
    /// `split`.
    fn merged_ranges_in(bytes: &[u8], split: usize) -> u64 {
        let (first, second) = bytes.split_at(split);
        let mut counter = ElementCounter::of_name(MERGED_RANGE_NAME);
        counter.count_in(first);
        counter.count_in(second);
        counter.num_elements
    }

    // The start of mergeCells, a longer name, is counted with the two
    // ranges; its end tag, with no prefix, is not.
    #[test]
    fn a_name_after_a_mark_is_counted_wherever_the_reads_split_it() {
        let bytes = br#"<mergeCells count="2"><mergeCell ref="A1:B1"/><x:mergeCell ref="A2:B2"/></mergeCells>"#;
        for split in 0..=bytes.len() {
            assert_eq!(merged_ranges_in(bytes, split), 3, "split at {split}");
        }
    }

    #[test]
    fn a_name_not_right_after_a_mark_is_not_counted() {
        let bytes = b"mergeCell <mergeCel <mergecell < mergeCell <<:mergeCell";
        for split in 0..=bytes.len() {
            assert_eq!(merged_ranges_in(bytes, split), 1, "split at {split}");
        }
    }

    // After a match that fails at a second mark, the next match starts at
    // that mark.
    #[test]
    fn a_name_after_two_marks_is_counted() {
        let bytes = b"<<mergeCell ref=\"A1:B1\"/>";
        for split in 0..=bytes.len() {
            assert_eq!(merged_ranges_in(bytes, split), 1, "split at {split}");
        }
    }

    #[test]
    fn a_name_given_a_byte_at_a_time_is_counted() {
        let mut counter = ElementCounter::of_name(b"sheet");
        for byte in b"<sheets><sheet/><x:sheet/></sheets>" {
            counter.count_in(&[*byte]);
        }
        assert_eq!(counter.num_elements, 3);
    }

    #[test]
    fn a_count_is_written_with_a_comma_between_groups_of_three_digits() {
        assert_eq!(text_of_count(0), "0");
        assert_eq!(text_of_count(999), "999");
        assert_eq!(text_of_count(4_000), "4,000");
        assert_eq!(text_of_count(123_456), "123,456");
        assert_eq!(text_of_count(1_000_000_000), "1,000,000,000");
    }
}
