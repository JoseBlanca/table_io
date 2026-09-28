//! The parts of the zip xlsx_rs reads itself before calamine, as "What
//! xlsx_rs reads before calamine" of `docs/specs/read.md` gives them.
//!
//! An xlsx is a zip of parts, each an XML file: the workbook, which lists
//! the sheets and holds the settings of the file, the date system among
//! them; the relationships, which give where the other parts are; the
//! sheets. Each part carries a checksum of its bytes, which the zip crate
//! checks only once the part has been read to its end; calamine stops
//! reading a sheet at its last cell, so xlsx_rs reads every part to its end
//! first, and counts the bytes. calamine also reads four parts whole when
//! it opens the file, the workbook, its relationships, the styles and the
//! table of texts, and reserves room for as many texts as the table says it
//! holds, so xlsx_rs bounds their sizes and the texts. A part is found as
//! calamine 0.36.1 finds it (`xlsx/mod.rs` and `utils.rs`), so that no file
//! passes these checks with a part calamine reads under another name.

use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Cursor, Read};

use quick_xml::Reader as XmlReader;
use quick_xml::events::attributes::AttrError;
use quick_xml::events::{BytesStart, Event};
use zip::ZipArchive;
use zip::result::ZipError;

use crate::ReadError;
use crate::attrs::{RawAttrIter, attribute_of, attributes_of, local_name_matches};
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
}

/// What xlsx_rs reads of the parts before calamine opens the file, and the
/// zip, kept for the merged ranges of the sheet calamine reads.
pub(crate) struct PartsRead<'bytes> {
    /// The date system of the workbook.
    pub(crate) date_system: DateSystem,
    /// The zip of the file.
    archive: Archive<'bytes>,
    /// The names of its parts.
    part_names: PartNames,
    /// The workbook the package relationships name.
    workbook: WorkbookRead,
}

/// What xlsx_rs reads of the workbook for the part of a sheet.
struct WorkbookRead {
    /// The folder of the workbook, such as `xl/`.
    folder: String,
    /// The sheets the workbook lists, in its order.
    sheets: Vec<WorkbookSheet>,
}

/// A sheet as the workbook lists it, an element `sheet`.
struct WorkbookSheet {
    /// Its name, as its tab shows it.
    name: String,
    /// The identifier of the relationship that gives its part, as the XML
    /// writes it, before its entities are read.
    raw_relationship_id: Vec<u8>,
}

impl PartsRead<'_> {
    /// Counts the merged ranges, the elements `mergeCell`, of the sheet
    /// `sheet_name`, in its first element `mergeCells`, as calamine's
    /// `merge_cells_by_sheet_name` finds them: its part is that of the first
    /// sheet of that name the workbook lists, given by the relationship of
    /// the workbook whose identifier the sheet names, relative to the folder
    /// of the workbook or, when it starts with `/`, to the root of the zip.
    /// A sheet whose part cannot be found so is let be, for calamine to
    /// refuse.
    ///
    /// calamine holds 16 bytes for each merged range before xlsx_rs sees
    /// one: 40,000,000 in a zip of 5.1 MB held 650 MB.
    ///
    /// # Errors
    ///
    /// [`ReadError::Unreadable`]: "too many merged ranges" for more than
    /// `max_merged_ranges`, at the first past it; "the part … cannot be read
    /// as XML: …", with quick-xml's message, for relationships of the
    /// workbook or a sheet whose XML cannot be read up to what xlsx_rs takes
    /// of it; and the zip crate's message for an error of the zip.
    pub(crate) fn check_merged_ranges(
        &mut self,
        sheet_name: &str,
        max_merged_ranges: u32,
    ) -> Result<(), ReadError> {
        let workbook = &self.workbook;
        let Some(sheet) = workbook
            .sheets
            .iter()
            .find(|workbook_sheet| workbook_sheet.name == sheet_name)
        else {
            return Ok(());
        };
        let relationships_path = format!("{}_rels/workbook.xml.rels", workbook.folder);
        let Some(target) = relationship_target_of(
            &mut self.archive,
            &self.part_names,
            &relationships_path,
            &sheet.raw_relationship_id,
        )?
        else {
            return Ok(());
        };
        let sheet_path = match target.strip_prefix('/') {
            Some(path_from_root) => path_from_root.to_owned(),
            None => format!("{}{target}", workbook.folder),
        };
        let Some(mut xml_reader) = xml_reader_of(&mut self.archive, &self.part_names, &sheet_path)?
        else {
            return Ok(());
        };
        let unreadable =
            |cause: &dyn std::fmt::Display| unreadable_xml_error_of(&sheet_path, cause);
        let mut buffer = Vec::new();
        loop {
            buffer.clear();
            match xml_reader.read_event_into(&mut buffer) {
                Ok(Event::Start(element)) if element.local_name().as_ref() == b"mergeCells" => {
                    break;
                }
                Ok(Event::Eof) => return Ok(()),
                Err(xml_error) => return Err(unreadable(&xml_error)),
                Ok(_) => {}
            }
        }
        let mut num_merged_ranges: u32 = 0;
        loop {
            buffer.clear();
            match xml_reader.read_event_into(&mut buffer) {
                Ok(Event::Start(element)) if element.local_name().as_ref() == b"mergeCell" => {
                    num_merged_ranges = num_merged_ranges.saturating_add(1);
                    if num_merged_ranges > max_merged_ranges {
                        return Err(ReadError::Unreadable("too many merged ranges".to_owned()));
                    }
                }
                Ok(Event::End(element)) if element.local_name().as_ref() == b"mergeCells" => {
                    return Ok(());
                }
                Ok(Event::Eof) => return Ok(()),
                Err(xml_error) => return Err(unreadable(&xml_error)),
                Ok(_) => {}
            }
        }
    }
}

/// Reads every part of the zip `bytes` to its end, so that the zip crate
/// checks its checksum; checks the size of the parts calamine reads whole
/// when it opens the file, and the number of texts of its table of texts;
/// and reads the date system of its workbook and the sheets it lists.
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
/// calamine", point 3); and "the part … cannot be read as XML: …", with
/// quick-xml's message, for package relationships or a workbook whose XML
/// cannot be read up to what xlsx_rs takes of it.
pub(crate) fn read_parts(bytes: &[u8], bounds: PartBounds) -> Result<PartsRead<'_>, ReadError> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(unreadable_of_zip_error)?;
    read_every_part(&mut archive, bounds)?;
    let part_names = PartNames::of_archive(&archive);
    let Some(workbook_folder) = workbook_folder_of(&mut archive, &part_names)? else {
        return Err(ReadError::Unreadable(
            "the file names no workbook".to_owned(),
        ));
    };
    let workbook_path = format!("{workbook_folder}workbook.xml");
    let (date_system, sheets) = read_workbook(&mut archive, &part_names, &workbook_path)?;
    Ok(PartsRead {
        date_system,
        archive,
        part_names,
        workbook: WorkbookRead {
            folder: workbook_folder,
            sheets,
        },
    })
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
    /// The first failure, `None` while there is none.
    failure: Option<ReadError>,
}

impl<'unzipped, Part: Read> PartReader<'unzipped, Part> {
    /// A reader of `part`, whose bytes are held to `bound` and counted in
    /// `unzipped`.
    fn new(part: Part, bound: Option<PartBound>, unzipped: &'unzipped mut UnzippedBytes) -> Self {
        Self {
            part,
            bound,
            num_bytes: 0,
            unzipped,
            failure: None,
        }
    }

    /// The error of the part, its failure when it has one, and otherwise
    /// the error of `read_result`, what the reading of the part gave.
    fn finish<Copied>(self, read_result: io::Result<Copied>) -> Result<(), ReadError> {
        if let Some(failure) = self.failure {
            return Err(failure);
        }
        read_result
            .map(|_| ())
            .map_err(|io_error| ReadError::Unreadable(io_error.to_string()))
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
        Ok(num_read)
    }
}

/// Reads every part of `archive` to its end and discards what it reads, so
/// that the zip crate checks the checksum of each, counting the bytes of
/// each and of all within `bounds`.
fn read_every_part(archive: &mut Archive<'_>, bounds: PartBounds) -> Result<(), ReadError> {
    let mut unzipped = UnzippedBytes {
        num_bytes: 0,
        max_bytes: bounds.max_unzipped_bytes,
    };
    for part_index in 0..archive.len() {
        let part = archive
            .by_index(part_index)
            .map_err(unreadable_of_zip_error)?;
        let part_kind = PartKind::of_zip_name(part.name());
        let mut part_reader =
            PartReader::new(part, part_kind.bound_of_bytes(&bounds), &mut unzipped);
        match part_kind {
            PartKind::TextTable => read_text_table(part_reader, bounds.max_texts)?,
            PartKind::PackageRelationships
            | PartKind::Workbook
            | PartKind::WorkbookRelationships
            | PartKind::Styles
            | PartKind::Other => {
                let copy_result = io::copy(&mut part_reader, &mut io::sink());
                part_reader.finish(copy_result)?;
            }
        }
    }
    Ok(())
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

/// Reads the table of texts that `part_reader` unzips with quick-xml,
/// configured as calamine configures it, to the end of the part or to its
/// first error of XML, and then the rest of its bytes, so that the part is
/// read to its end ("What xlsx_rs reads before calamine" of
/// `docs/specs/read.md`, point 3). It counts every element of local name
/// `si`, each a text, those inside another `si` and outside the root among
/// them, and reads every attribute `uniqueCount` of every element of local
/// name `sst`, the number of texts the table says it holds, for which
/// calamine reserves room when it meets the first `sst`. calamine counts
/// fewer: the `si` of the first `sst`, not those inside another, and
/// reserves room for the first `uniqueCount` of the first `sst`.
///
/// An error of XML is not an error here: calamine refuses a file at one
/// before `</sst>` and never reads one after it.
///
/// # Errors
///
/// [`ReadError::Unreadable`]: "too many texts" at the first `si` past
/// `max_texts`, or at a `uniqueCount` past it; and the failure of
/// `part_reader`, an error of the zip or a bound of its bytes passed.
fn read_text_table<Part: Read>(
    part_reader: PartReader<'_, Part>,
    max_texts: u64,
) -> Result<(), ReadError> {
    let mut xml_reader = xml_reader_over(BufReader::new(part_reader));
    let mut num_texts: u64 = 0;
    let mut buffer = Vec::new();
    loop {
        buffer.clear();
        match xml_reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) => match element.local_name().as_ref() {
                b"si" => {
                    num_texts = num_texts.saturating_add(1);
                    if num_texts > max_texts {
                        return Err(too_many_texts());
                    }
                }
                b"sst" if is_unique_count_past(&element, max_texts) => {
                    return Err(too_many_texts());
                }
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            Ok(_) => {}
        }
    }
    read_to_end(xml_reader)
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

/// Reads what is left of the part of `xml_reader` to its end, past where
/// the reader of XML stopped, and gives the failure of the part, if any.
///
/// # Errors
///
/// The failure of the part: an error of the zip, or a bound of its bytes
/// passed.
fn read_to_end<Part: Read>(
    xml_reader: XmlReader<BufReader<PartReader<'_, Part>>>,
) -> Result<(), ReadError> {
    let mut buffered_part = xml_reader.into_inner();
    let copy_result = io::copy(&mut buffered_part, &mut io::sink());
    buffered_part.into_inner().finish(copy_result)
}

/// The date system of the workbook `path`, and the sheets it lists.
///
/// The date system is that of the last element `workbookPr` that is a
/// direct child of its root element, whatever the namespace of either,
/// 1904 when its attribute `date1904` is `1` or `true`, as calamine reads
/// that attribute; 1900 when there is no such element or no such part.
/// calamine takes the last element of that name anywhere in the workbook,
/// and Excel 365 writes one of the namespace `x15` inside `extLst`, after
/// the one of the root, with no `date1904`: every workbook of 1904 Excel
/// 365 saves would be read by calamine as one of 1900.
///
/// The sheets are the elements `sheet` anywhere in the workbook, as
/// calamine's `read_workbook` lists them: the name from the last attribute
/// `name`, its entities read, and the relationship from the last attribute
/// whose name, after any prefix, is `id`. A sheet whose name cannot be
/// read is left out, since calamine refuses the file.
fn read_workbook(
    archive: &mut Archive<'_>,
    part_names: &PartNames,
    path: &str,
) -> Result<(DateSystem, Vec<WorkbookSheet>), ReadError> {
    let mut sheets = Vec::new();
    let Some(mut xml_reader) = xml_reader_of(archive, part_names, path)? else {
        return Ok((DateSystem::Excel1900, sheets));
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
                if element.local_name().as_ref() == b"sheet" {
                    let (raw_name, raw_relationship_id) =
                        raw_sheet_of(&element).map_err(|xml_error| unreadable(&xml_error))?;
                    let name =
                        xml_reader
                            .decoder()
                            .decode(&raw_name)
                            .ok()
                            .and_then(|decoded_name| {
                                quick_xml::escape::unescape(&decoded_name)
                                    .ok()
                                    .map(std::borrow::Cow::into_owned)
                            });
                    if let Some(name) = name {
                        sheets.push(WorkbookSheet {
                            name,
                            raw_relationship_id,
                        });
                    }
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
    Ok((date_system, sheets))
}

/// The attributes `name` and `id` of `sheet`, an element `sheet` of the
/// workbook, as the XML writes them, each the last of its name and empty
/// when missing, as calamine reads them; `id` with any prefix, `r:id` as
/// Excel writes it.
fn raw_sheet_of(sheet: &BytesStart<'_>) -> Result<(Vec<u8>, Vec<u8>), AttrError> {
    let mut raw_name = Vec::new();
    let mut raw_relationship_id = Vec::new();
    for attribute in RawAttrIter::of_element(sheet) {
        let (key, value) = attribute?;
        if key == b"name" {
            raw_name = value.to_vec();
        } else if local_name_matches(key, b"id") {
            raw_relationship_id = value.to_vec();
        }
    }
    Ok((raw_name, raw_relationship_id))
}

/// The target of the relationship of identifier `raw_relationship_id` in
/// the relationships of the workbook `path`, as calamine's
/// `read_relationships` reads them: the last relationship of that
/// identifier, its target decoded and its entities not read. `None` when
/// there is no such part or relationship, a file calamine refuses.
fn relationship_target_of(
    archive: &mut Archive<'_>,
    part_names: &PartNames,
    path: &str,
    raw_relationship_id: &[u8],
) -> Result<Option<String>, ReadError> {
    let Some(mut xml_reader) = xml_reader_of(archive, part_names, path)? else {
        return Ok(None);
    };
    let unreadable = |cause: &dyn std::fmt::Display| unreadable_xml_error_of(path, cause);
    let mut target = None;
    let mut buffer = Vec::new();
    loop {
        buffer.clear();
        match xml_reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) if element.local_name().as_ref() == b"Relationship" => {
                let RawWorkbookRelationship { raw_id, raw_target } =
                    raw_workbook_relationship_of(&element)
                        .map_err(|xml_error| unreadable(&xml_error))?;
                if raw_id.as_deref() == Some(raw_relationship_id) {
                    let decoded_target = xml_reader
                        .decoder()
                        .decode(raw_target.as_deref().unwrap_or_default())
                        .map_err(|encoding_error| unreadable(&encoding_error))?;
                    target = Some(decoded_target.into_owned());
                }
            }
            Ok(Event::End(element)) if element.local_name().as_ref() == b"Relationships" => break,
            Ok(Event::Eof) => break,
            Err(xml_error) => return Err(unreadable(&xml_error)),
            Ok(_) => {}
        }
    }
    Ok(target)
}

/// The attributes `Id` and `Target` of a relationship of the workbook, as
/// the XML writes them, before their entities are read; either may be
/// missing.
struct RawWorkbookRelationship {
    /// The identifier a sheet of the workbook names the relationship by.
    raw_id: Option<Vec<u8>>,
    /// The path of the part it names.
    raw_target: Option<Vec<u8>>,
}

/// The attributes `Id` and `Target` of `element`, a relationship of the
/// workbook, read as calamine's `get_attrs!` reads them with `Type`.
fn raw_workbook_relationship_of(
    element: &BytesStart<'_>,
) -> Result<RawWorkbookRelationship, AttrError> {
    let [raw_id, _, raw_target] = attributes_of(element, [b"Id", b"Type", b"Target"])?;
    Ok(RawWorkbookRelationship {
        raw_id: raw_id.map(<[u8]>::to_vec),
        raw_target: raw_target.map(<[u8]>::to_vec),
    })
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
    use crate::parts::text_of_count;

    #[test]
    fn a_count_is_written_with_a_comma_between_groups_of_three_digits() {
        assert_eq!(text_of_count(0), "0");
        assert_eq!(text_of_count(999), "999");
        assert_eq!(text_of_count(4_000), "4,000");
        assert_eq!(text_of_count(123_456), "123,456");
        assert_eq!(text_of_count(1_000_000_000), "1,000,000,000");
    }
}
