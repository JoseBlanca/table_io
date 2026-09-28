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
    /// The most bytes the table of texts may hold once unzipped,
    /// [`crate::MAX_TEXT_TABLE_BYTES`] outside the tests.
    pub(crate) max_text_table_bytes: u64,
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
    /// The workbook, or `None` when the package relationships do not name
    /// one, a file calamine refuses.
    workbook: Option<WorkbookRead>,
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
        let Some(workbook) = &self.workbook else {
            return Ok(());
        };
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
/// A file with no workbook where calamine looks for one, or whose package
/// relationships do not name it, is given the system of 1900 and is not
/// checked further: calamine then refuses the file, or finds no sheet in
/// it.
///
/// # Errors
///
/// [`ReadError::Unreadable`] with the zip crate's message for any error of
/// the zip, "Invalid checksum" for a part whose bytes are not those it was
/// saved with; "the file unzips to more than … bytes" when the parts hold
/// more than `bounds.max_unzipped_bytes` bytes together; "a part of the
/// file is too large", "too much text", "too many texts" and "the table of
/// texts says it holds more texts than it does", for the bounds of the
/// parts calamine reads whole ("What xlsx_rs reads before calamine",
/// point 3); and "the part … cannot be read as XML: …",
/// with quick-xml's message, for package relationships, a table of texts
/// or a workbook whose XML cannot be read up to what xlsx_rs takes of it.
pub(crate) fn read_parts(bytes: &[u8], bounds: PartBounds) -> Result<PartsRead<'_>, ReadError> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(unreadable_of_zip_error)?;
    let part_sizes = read_every_part(&mut archive, bounds.max_unzipped_bytes)?;
    let part_names = PartNames::of_archive(&archive);
    let part_size_of = |archive: &Archive<'_>, path: &str| {
        archive
            .index_for_name(part_names.zip_name_of(path))
            .and_then(|part_index| part_sizes.get(part_index).copied())
    };
    // Checked before it is read, as the other parts calamine reads whole.
    if part_size_of(&archive, PACKAGE_RELATIONSHIPS_PATH).is_some_and(is_past_settings_part_bytes) {
        return Err(too_large_part());
    }
    let Some(workbook_folder) = workbook_folder_of(&mut archive, &part_names)? else {
        return Ok(PartsRead {
            date_system: DateSystem::Excel1900,
            archive,
            part_names,
            workbook: None,
        });
    };
    let workbook_path = format!("{workbook_folder}workbook.xml");
    let text_table_path = format!("{workbook_folder}sharedStrings.xml");
    let settings_paths = [
        workbook_path.clone(),
        format!("{workbook_folder}_rels/workbook.xml.rels"),
        format!("{workbook_folder}styles.xml"),
    ];
    if settings_paths
        .iter()
        .filter_map(|path| part_size_of(&archive, path))
        .any(is_past_settings_part_bytes)
    {
        return Err(too_large_part());
    }
    if part_size_of(&archive, &text_table_path)
        .is_some_and(|num_bytes| num_bytes > bounds.max_text_table_bytes)
    {
        return Err(ReadError::Unreadable("too much text".to_owned()));
    }
    check_text_table(&mut archive, &part_names, &text_table_path)?;
    let (date_system, sheets) = read_workbook(&mut archive, &part_names, &workbook_path)?;
    Ok(PartsRead {
        date_system,
        archive,
        part_names,
        workbook: Some(WorkbookRead {
            folder: workbook_folder,
            sheets,
        }),
    })
}

/// The zip of an xlsx held in memory.
type Archive<'bytes> = ZipArchive<Cursor<&'bytes [u8]>>;

/// The path of the package relationships, the part that gives the folder
/// of the workbook.
const PACKAGE_RELATIONSHIPS_PATH: &str = "_rels/.rels";

/// Whether a part of `num_bytes` bytes unzipped, one calamine reads whole
/// when it opens the file other than the table of texts, passes
/// [`crate::MAX_SETTINGS_PART_BYTES`].
fn is_past_settings_part_bytes(num_bytes: u64) -> bool {
    num_bytes > crate::MAX_SETTINGS_PART_BYTES
}

/// The error of a part calamine reads whole that passes its bound.
fn too_large_part() -> ReadError {
    ReadError::Unreadable("a part of the file is too large".to_owned())
}

/// Reads every part of `archive` to its end and discards what it reads,
/// and gives the number of bytes of each part unzipped, in the order of
/// the zip.
fn read_every_part(
    archive: &mut Archive<'_>,
    max_unzipped_bytes: u64,
) -> Result<Vec<u64>, ReadError> {
    let mut unzipped_bytes: u64 = 0;
    let mut part_sizes = Vec::with_capacity(archive.len());
    for part_index in 0..archive.len() {
        let mut part = archive
            .by_index(part_index)
            .map_err(unreadable_of_zip_error)?;
        // One byte more than the room left, so that a part that passes the
        // bound is found without being read past it.
        let room_left = max_unzipped_bytes.saturating_sub(unzipped_bytes);
        let part_bytes = io::copy(
            &mut part.by_ref().take(room_left.saturating_add(1)),
            &mut io::sink(),
        )
        .map_err(|zip_io_error| ReadError::Unreadable(zip_io_error.to_string()))?;
        unzipped_bytes = unzipped_bytes.saturating_add(part_bytes);
        if unzipped_bytes > max_unzipped_bytes {
            return Err(ReadError::Unreadable(format!(
                "the file unzips to more than {} bytes",
                text_of_count(max_unzipped_bytes)
            )));
        }
        part_sizes.push(part_bytes);
    }
    Ok(part_sizes)
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
    let mut xml_reader = XmlReader::from_reader(BufReader::new(part));
    let config = xml_reader.config_mut();
    config.check_end_names = false;
    config.trim_text(false);
    config.check_comments = false;
    config.expand_empty_elements = true;
    Ok(Some(xml_reader))
}

/// The folder of the workbook, such as `xl/`, as calamine's
/// `read_package_relationships` finds it: from the target of the last
/// relationship of type `officeDocument` in `_rels/.rels`, up to its last
/// `/`, with no `/` at its start. `None` when there is no such part or
/// relationship, a file calamine refuses.
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
            Ok(Event::Eof) => return Ok(None),
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

/// Checks the table of texts `path` as calamine's `read_shared_strings`
/// reads it: the elements `si` of its root `sst`, each a text, those
/// inside another `si` not counted; and the attribute `uniqueCount` of
/// `sst`, the number of texts it says it holds, for which calamine reserves
/// room before it reads the first. A `uniqueCount` that is not a whole
/// number written in digits is let be, as calamine does; a part with no
/// root `sst`, or cut short, is let be too, for calamine to refuse.
///
/// # Errors
///
/// [`ReadError::Unreadable`]: "too many texts" past
/// [`crate::MAX_TEXTS`], at the first `si` past it; "the table of texts
/// says it holds more texts than it does" for a `uniqueCount` larger than
/// the texts; and "the part … cannot be read as XML: …".
fn check_text_table(
    archive: &mut Archive<'_>,
    part_names: &PartNames,
    path: &str,
) -> Result<(), ReadError> {
    let Some(mut xml_reader) = xml_reader_of(archive, part_names, path)? else {
        return Ok(());
    };
    let unreadable = |cause: &dyn std::fmt::Display| unreadable_xml_error_of(path, cause);
    let mut buffer = Vec::new();
    let unique_count = loop {
        buffer.clear();
        match xml_reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) if element.local_name().as_ref() == b"sst" => {
                break unique_count_of(&element).map_err(|xml_error| unreadable(&xml_error))?;
            }
            Ok(Event::Eof) => return Ok(()),
            Err(xml_error) => return Err(unreadable(&xml_error)),
            Ok(_) => {}
        }
    };
    let mut num_texts: u64 = 0;
    loop {
        buffer.clear();
        match xml_reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) if element.local_name().as_ref() == b"si" => {
                num_texts = num_texts.saturating_add(1);
                if num_texts > crate::MAX_TEXTS {
                    return Err(ReadError::Unreadable("too many texts".to_owned()));
                }
                // calamine reads a text up to the first end of an element of
                // the same name, and counts nothing inside it.
                let text_name = element.name().as_ref().to_vec();
                let mut text_buffer = Vec::new();
                loop {
                    text_buffer.clear();
                    match xml_reader.read_event_into(&mut text_buffer) {
                        Ok(Event::End(end)) if end.name().as_ref() == text_name.as_slice() => break,
                        Ok(Event::Eof) => return Ok(()),
                        Err(xml_error) => return Err(unreadable(&xml_error)),
                        Ok(_) => {}
                    }
                }
            }
            Ok(Event::End(element)) if element.local_name().as_ref() == b"sst" => break,
            Ok(Event::Eof) => return Ok(()),
            Err(xml_error) => return Err(unreadable(&xml_error)),
            Ok(_) => {}
        }
    }
    if unique_count.is_some_and(|unique_count| unique_count > num_texts) {
        return Err(ReadError::Unreadable(
            "the table of texts says it holds more texts than it does".to_owned(),
        ));
    }
    Ok(())
}

/// The attribute `uniqueCount` of `sst`, the root of a table of texts, as
/// calamine reads it, up to the first attribute of that name: a number
/// when it is written in digits alone, any zeros first among them, and
/// `None` when it is missing, holds anything else, or passes a `u64`.
fn unique_count_of(sst: &BytesStart<'_>) -> Result<Option<u64>, AttrError> {
    let Some(digits) = attribute_of(sst, b"uniqueCount")? else {
        return Ok(None);
    };
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return Ok(None);
    }
    Ok(std::str::from_utf8(digits)
        .ok()
        .and_then(|digits| digits.parse::<u64>().ok()))
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
