//! The parts of the zip xlsx_rs reads itself before calamine, as "What
//! xlsx_rs reads before calamine" of `docs/specs/read.md` gives them.
//!
//! An xlsx is a zip of parts, each an XML file: the workbook, which lists
//! the sheets and holds the settings of the file, the date system among
//! them; the relationships, which give where the other parts are; the
//! sheets. Each part carries a checksum of its bytes, which the zip crate
//! checks only once the part has been read to its end; calamine stops
//! reading a sheet at its last cell, so xlsx_rs reads every part to its end
//! first, and counts the bytes. A part is found as calamine 0.36.1 finds
//! it (`xlsx/mod.rs` and `utils.rs`), so that no file passes these checks
//! with a part calamine reads under another name.

use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Cursor, Read};

use quick_xml::Reader as XmlReader;
use quick_xml::events::attributes::AttrError;
use quick_xml::events::{BytesStart, Event};
use zip::ZipArchive;
use zip::result::ZipError;

use crate::ReadError;
use crate::date::DateSystem;

/// What xlsx_rs reads of the parts before calamine opens the file.
pub(crate) struct PartsRead {
    /// The date system of the workbook.
    pub(crate) date_system: DateSystem,
}

/// Reads every part of the zip `bytes` to its end, so that the zip crate
/// checks its checksum, and the date system of its workbook.
///
/// A file with no workbook where calamine looks for one, or whose package
/// relationships do not name it, is given the system of 1900: calamine
/// then refuses the file, or finds no sheet in it.
///
/// # Errors
///
/// [`ReadError::Unreadable`] with the zip crate's message for any error of
/// the zip, "Invalid checksum" for a part whose bytes are not those it was
/// saved with; "the file unzips to more than … bytes" when the parts hold
/// more than `max_unzipped_bytes` bytes together; and "the part … cannot be
/// read as XML: …", with quick-xml's message, for package relationships or
/// a workbook whose XML cannot be read up to what xlsx_rs takes of it.
pub(crate) fn read_parts(bytes: &[u8], max_unzipped_bytes: u64) -> Result<PartsRead, ReadError> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(unreadable_of_zip_error)?;
    read_every_part(&mut archive, max_unzipped_bytes)?;
    let part_names = PartNames::of_archive(&archive);
    let date_system = match workbook_folder_of(&mut archive, &part_names)? {
        Some(workbook_folder) => date_system_of(
            &mut archive,
            &part_names,
            &format!("{workbook_folder}workbook.xml"),
        )?,
        None => DateSystem::Excel1900,
    };
    Ok(PartsRead { date_system })
}

/// The zip of an xlsx held in memory.
type Archive<'bytes> = ZipArchive<Cursor<&'bytes [u8]>>;

/// Reads every part of `archive` to its end and discards what it reads.
fn read_every_part(archive: &mut Archive<'_>, max_unzipped_bytes: u64) -> Result<(), ReadError> {
    let mut unzipped_bytes: u64 = 0;
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
    const PATH: &str = "_rels/.rels";
    let Some(mut xml_reader) = xml_reader_of(archive, part_names, PATH)? else {
        return Ok(None);
    };
    let unreadable = |cause: &dyn std::fmt::Display| unreadable_xml_error_of(PATH, cause);
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
/// as calamine's `get_attrs!` reads them: up to the attribute that makes
/// both found.
fn raw_relationship_of(element: &BytesStart<'_>) -> Result<RawRelationship, AttrError> {
    let mut raw_type = None;
    let mut raw_target = None;
    let mut num_found: u8 = 0;
    for attribute in element.attributes().with_checks(false) {
        let attribute = attribute?;
        match attribute.key.as_ref() {
            b"Type" => {
                raw_type = Some(attribute.value.into_owned());
                num_found = num_found.saturating_add(1);
            }
            b"Target" => {
                raw_target = Some(attribute.value.into_owned());
                num_found = num_found.saturating_add(1);
            }
            _ => {}
        }
        if num_found == 2 {
            break;
        }
    }
    Ok(RawRelationship {
        raw_type,
        raw_target,
    })
}

/// The date system of the workbook `path`: that of the last element
/// `workbookPr` that is a direct child of its root element, whatever the
/// namespace of either, 1904 when its attribute `date1904` is `1` or
/// `true`, as calamine reads that attribute; 1900 when there is no such
/// element or no such part.
///
/// calamine takes the last element of that name anywhere in the workbook,
/// and Excel 365 writes one of the namespace `x15` inside `extLst`, after
/// the one of the root, with no `date1904`: every workbook of 1904 Excel
/// 365 saves would be read by calamine as one of 1900.
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
    for attribute in workbook_pr.attributes().with_checks(false) {
        let attribute = attribute?;
        if attribute.key.as_ref() == b"date1904" {
            let is_1904 = matches!(attribute.value.as_ref(), b"1" | b"true");
            return Ok(if is_1904 {
                DateSystem::Excel1904
            } else {
                DateSystem::Excel1900
            });
        }
    }
    Ok(DateSystem::Excel1900)
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
