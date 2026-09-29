//! The parts of the zip xlsx_rs reads itself before calamine ("What xlsx_rs
//! reads before calamine" of `docs/specs/read.md`): every part read to its
//! end, so that a part whose checksum fails refuses the file, and the bytes
//! so read counted and bounded; the date system, from the `workbookPr` that
//! is a direct child of the root of the workbook, in the workbook that
//! `_rels/.rels` names; the bounds of the parts calamine holds whole, found
//! by the end of their names in any folder, each at its value; the merged
//! ranges counted in every part; the files of the review of 28 September
//! 2026 that trapped the package; and the bound of every part, with the
//! rule of UTF-8 by the bytes of a part. The messages of the bounds are
//! also tested with small bounds in `src/bounds_tests.rs`.

#[cfg(test)]
#[expect(
    dead_code,
    reason = "the texts here are in a table of texts, not written in the cells by hand"
)]
mod hand_written;

use std::error::Error;
use std::io::{Cursor, Write};
use std::ops::Range;

use rust_xlsxwriter::{Workbook, XlsxError};
use xlsx_rs::{ReadError, Sheet, SheetCell, read_first_sheet};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::hand_written::{
    parts_of_1904_worksheet, parts_of_worksheet, shared_strings, stored_zip, xlsx_of_parts,
};

/// `MAX_SHEET_CELLS` of popnei_web, the limit its light worker gives.
const MAX_SHEET_CELLS: u32 = 2_000_000;

/// An xlsx written by rust_xlsxwriter with a table of 200 individuals, each
/// with a name, a population and a height, under a header.
fn xlsx_of_individuals() -> Result<Vec<u8>, XlsxError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "id")?;
    worksheet.write_string(0, 1, "pop")?;
    worksheet.write_string(0, 2, "height")?;
    for row in 1..=200u32 {
        worksheet.write_string(row, 0, format!("ind{row}"))?;
        let population = if row.is_multiple_of(2) {
            "north"
        } else {
            "south"
        };
        worksheet.write_string(row, 1, population)?;
        worksheet.write_number(row, 2, f64::from(row) * 1.37)?;
    }
    workbook.save_to_buffer()
}

/// The positions in `bytes`, an xlsx, of the compressed data of the part
/// `part_name`, as the zip's directory gives them.
fn compressed_data_of(bytes: &[u8], part_name: &str) -> Result<Range<usize>, Box<dyn Error>> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))?;
    let part = archive.by_name(part_name)?;
    let start = usize::try_from(part.data_start().ok_or("no start of the data")?)?;
    let length = usize::try_from(part.compressed_size())?;
    let end = start
        .checked_add(length)
        .ok_or("the data ends past a usize")?;
    Ok(start..end)
}

/// The messages of the zip crate for a part whose compressed bytes are
/// damaged: its checksum, and the errors of its reader of deflate, the
/// compression of a zip.
const ZIP_MESSAGES_OF_DAMAGED_DATA: [&str; 3] = [
    "Invalid checksum",
    "corrupt deflate stream",
    "incomplete deflate stream",
];

// Every copy is either read with the cells of the unchanged file or refused
// with the message of the zip crate, before calamine reads a cell: calamine
// alone read such copies with cells missing, and xlsx_rs, before it read
// every part to its end, refused some with calamine's message of the XML
// and some as an empty sheet or a sheet too large.
#[test]
fn a_sheet_damaged_in_its_compressed_bytes_is_refused_by_the_zip_or_read_whole() {
    let bytes = xlsx_of_individuals().unwrap();
    let unchanged_sheet = read_first_sheet(&bytes, MAX_SHEET_CELLS).unwrap();
    assert_eq!(unchanged_sheet.num_rows, 201);
    let sheet_data = compressed_data_of(&bytes, "xl/worksheets/sheet1.xml").unwrap();
    assert!(sheet_data.len() > 1_000);

    let mut num_refused: u32 = 0;
    let mut other_reads: Vec<(usize, Result<Sheet, ReadError>)> = Vec::new();
    for position in sheet_data {
        let mut damaged_bytes = bytes.clone();
        damaged_bytes[position] = !damaged_bytes[position];
        let read = read_first_sheet(&damaged_bytes, MAX_SHEET_CELLS);
        let is_refused_by_the_zip = matches!(
            &read,
            Err(ReadError::Unreadable(message))
                if ZIP_MESSAGES_OF_DAMAGED_DATA.contains(&message.as_str())
        );
        if is_refused_by_the_zip {
            num_refused = num_refused.checked_add(1).unwrap();
        } else if read.as_ref() != Ok(&unchanged_sheet) {
            other_reads.push((position, read));
        }
    }

    assert_eq!(
        other_reads.len(),
        0,
        "{} copies read otherwise, the first {:?}",
        other_reads.len(),
        other_reads
            .first()
            .map(|(position, read)| (position, read.as_ref().err()))
    );
    assert!(num_refused > 1_000);
}

#[test]
fn the_first_500_bytes_of_an_xlsx_are_unreadable_with_the_zip_crates_message() {
    let bytes = xlsx_of_individuals().unwrap();

    let read = read_first_sheet(&bytes[..500], MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable(
            "invalid Zip archive: Could not find EOCD".to_owned()
        ))
    );
}

/// The sheet of one cell, A1, 43963 with the format `dd/mm/yyyy`, in a
/// workbook of the 1904 system built from its parts by
/// [`parts_of_1904_worksheet`], after `change_parts` has changed them; 43963
/// is 13 May 2024 in the system of 1904 and 12 May 2020 in that of 1900.
fn read_of_1904_date(
    change_parts: impl FnOnce(&mut Vec<(String, String)>),
) -> Result<Sheet, ReadError> {
    let mut parts = parts_of_1904_worksheet(
        r#"<sheetData><row r="1"><c r="A1" s="1"><v>43963</v></c></row></sheetData>"#,
        &["dd/mm/yyyy"],
    );
    change_parts(&mut parts);
    let files: Vec<(&str, &str)> = parts
        .iter()
        .map(|(name, xml)| (name.as_str(), xml.as_str()))
        .collect();
    read_first_sheet(&stored_zip(&files), MAX_SHEET_CELLS)
}

/// The XML of the part `part_name` of `parts`, or `None` when there is no
/// such part.
fn xml_of<'parts>(
    parts: &'parts mut [(String, String)],
    part_name: &str,
) -> Option<&'parts mut String> {
    parts
        .iter_mut()
        .find(|(name, _)| name == part_name)
        .map(|(_, xml)| xml)
}

/// The element `extLst` Excel 365 writes at the end of a workbook, with an
/// element `workbookPr` of the namespace `x15` whose attributes are
/// `x15_attributes`.
fn ext_lst_with_x15_workbook_pr(x15_attributes: &str) -> String {
    format!(
        r#"<extLst><ext uri="{{140A7094-0E35-4892-8432-C4D2E57EDEB5}}" xmlns:x15="http://schemas.microsoft.com/office/spreadsheetml/2010/11/main"><x15:workbookPr {x15_attributes}/></ext></extLst></workbook>"#
    )
}

#[test]
fn the_workbook_pr_of_the_root_gives_the_date_system_and_not_the_one_in_ext_lst() {
    let read = read_of_1904_date(|parts| {
        let workbook = xml_of(parts, "xl/workbook.xml").unwrap();
        *workbook = workbook.replace(
            "</workbook>",
            &ext_lst_with_x15_workbook_pr(r#"chartTrackingRefBase="1""#),
        );
    });

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Text("2024-05-13".to_owned())]
    );
}

#[test]
fn a_date1904_only_on_the_workbook_pr_in_ext_lst_leaves_the_system_of_1900() {
    let read = read_of_1904_date(|parts| {
        let workbook = xml_of(parts, "xl/workbook.xml").unwrap();
        *workbook = workbook
            .replace(r#"<workbookPr date1904="1"/>"#, "<workbookPr/>")
            .replace(
                "</workbook>",
                &ext_lst_with_x15_workbook_pr(r#"date1904="1""#),
            );
    });

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Text("2020-05-12".to_owned())]
    );
}

#[test]
fn a_date1904_of_true_is_the_system_of_1904() {
    let read = read_of_1904_date(|parts| {
        let workbook = xml_of(parts, "xl/workbook.xml").unwrap();
        *workbook = workbook.replace(r#"date1904="1""#, r#"date1904="true""#);
    });

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Text("2024-05-13".to_owned())]
    );
}

/// The date of [`read_of_1904_date`] whose `workbookPr` is replaced by
/// `workbook_properties`.
fn read_of_date_with_workbook_properties(workbook_properties: &str) -> Result<Sheet, ReadError> {
    read_of_1904_date(|parts| {
        for (name, xml) in parts.iter_mut() {
            if name == "xl/workbook.xml" {
                *xml = xml.replace(r#"<workbookPr date1904="1"/>"#, workbook_properties);
                assert!(xml.contains(workbook_properties));
            }
        }
    })
}

/// The cell of 43963 read in the system of 1904, 13 May 2024.
fn date_of_1904() -> Vec<SheetCell> {
    vec![SheetCell::Text("2024-05-13".to_owned())]
}

/// The cell of 43963 read in the system of 1900, 12 May 2020.
fn date_of_1900() -> Vec<SheetCell> {
    vec![SheetCell::Text("2020-05-12".to_owned())]
}

#[test]
fn a_workbook_pr_with_a_prefix_gives_the_date_system() {
    let read = read_of_date_with_workbook_properties(
        r#"<x:workbookPr xmlns:x="http://schemas.openxmlformats.org/spreadsheetml/2006/main" date1904="1"/>"#,
    );

    assert_eq!(read.unwrap().cells, date_of_1904());
}

#[test]
fn a_date1904_of_0_is_the_system_of_1900() {
    let read = read_of_date_with_workbook_properties(r#"<workbookPr date1904="0"/>"#);

    assert_eq!(read.unwrap().cells, date_of_1900());
}

#[test]
fn of_two_workbook_pr_children_of_the_root_the_last_gives_the_date_system() {
    let last_of_1900 =
        read_of_date_with_workbook_properties(r#"<workbookPr date1904="1"/><workbookPr/>"#);
    let last_of_1904 =
        read_of_date_with_workbook_properties(r#"<workbookPr/><workbookPr date1904="1"/>"#);

    assert_eq!(last_of_1900.unwrap().cells, date_of_1900());
    assert_eq!(last_of_1904.unwrap().cells, date_of_1904());
}

// calamine reads the first attribute of the name, with RawAttrIter.
#[test]
fn of_two_date1904_the_first_gives_the_date_system() {
    let first_of_1904 =
        read_of_date_with_workbook_properties(r#"<workbookPr date1904="1" date1904="0"/>"#);
    let first_of_1900 =
        read_of_date_with_workbook_properties(r#"<workbookPr date1904="0" date1904="1"/>"#);

    assert_eq!(first_of_1904.unwrap().cells, date_of_1904());
    assert_eq!(first_of_1900.unwrap().cells, date_of_1900());
}

// A file Excel saves as "Strict Open XML" has other namespaces for its
// parts and for the types of its relationships, those of
// http://purl.oclc.org/ooxml/; the parts of the package, `_rels/.rels`, keep
// theirs.
#[test]
fn a_strict_workbook_of_the_1904_system_gives_its_dates_in_that_system() {
    let read = read_of_1904_date(|parts| {
        for (_, xml) in parts.iter_mut() {
            *xml = xml
                .replace(
                    "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
                    "http://purl.oclc.org/ooxml/spreadsheetml/main",
                )
                .replace(
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
                    "http://purl.oclc.org/ooxml/officeDocument/relationships",
                );
        }
        assert!(
            xml_of(parts, "xl/workbook.xml")
                .unwrap()
                .contains("purl.oclc.org/ooxml/spreadsheetml")
        );
    });

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Text("2024-05-13".to_owned())]
    );
}

// calamine finds a part by its name ignoring case, with `\` in the names of
// the zip read as `/`, and so does xlsx_rs: a workbook it did not find
// would be given the system of 1900 while calamine read its sheet.
#[test]
fn a_workbook_whose_parts_are_named_in_capitals_and_with_backslashes_is_found() {
    let read = read_of_1904_date(|parts| {
        for (name, _) in parts.iter_mut() {
            *name = name.to_ascii_uppercase().replace('/', "\\");
        }
        assert!(xml_of(parts, "XL\\WORKBOOK.XML").is_some());
    });

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Text("2024-05-13".to_owned())]
    );
}

#[test]
fn a_workbook_in_the_folder_its_relationship_gives_is_found_there() {
    let read = read_of_1904_date(|parts| {
        for (name, _) in parts.iter_mut() {
            if let Some(name_in_folder) = name.strip_prefix("xl/") {
                *name = format!("book/{name_in_folder}");
            }
        }
        let package_relationships = xml_of(parts, "_rels/.rels").unwrap();
        *package_relationships = package_relationships.replace(
            r#"Target="xl/workbook.xml""#,
            r#"Target="/book/workbook.xml""#,
        );
    });

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Text("2024-05-13".to_owned())]
    );
}

/// The relationship of `_rels/.rels` of type `officeDocument` whose
/// target is `target`, with the attributes `type_attributes` in the place
/// of its `Type`.
fn office_document_relationship(id: &str, type_attributes: &str, target: &str) -> String {
    format!(r#"<Relationship Id="{id}" {type_attributes} Target="{target}"/>"#)
}

/// The attribute `Type` of a relationship of type `officeDocument`.
const OFFICE_DOCUMENT_TYPE: &str =
    r#"Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument""#;

/// The date of [`read_of_1904_date`] with a copy of its workbook, of the
/// system of 1900, in the folder `data/`, and `_rels/.rels` holding
/// `relationships` inside its element `Relationships` and `after_end` after
/// its end: the date is 2024-05-13 when the workbook of `xl/` is taken,
/// and 2020-05-12 when the copy is.
fn read_of_1904_date_with_package_relationships(
    relationships: &str,
    after_end: &str,
) -> Result<Sheet, ReadError> {
    read_of_1904_date(|parts| {
        let copies: Vec<(String, String)> = parts
            .iter()
            .filter_map(|(name, xml)| {
                let name_in_folder = name.strip_prefix("xl/")?;
                Some((
                    format!("data/{name_in_folder}"),
                    xml.replace(r#"<workbookPr date1904="1"/>"#, "<workbookPr/>"),
                ))
            })
            .collect();
        parts.extend(copies);
        for (name, xml) in parts.iter_mut() {
            if name == "_rels/.rels" {
                let head = xml
                    .split_once("<Relationship ")
                    .map_or("", |(head, _)| head)
                    .to_owned();
                assert!(head.ends_with('>'), "no relationship in {xml}");
                *xml = format!("{head}{relationships}</Relationships>{after_end}");
            }
        }
    })
}

#[test]
fn the_last_office_document_relationship_gives_the_workbook() {
    let read = read_of_1904_date_with_package_relationships(
        &[
            office_document_relationship("rId1", OFFICE_DOCUMENT_TYPE, "data/workbook.xml"),
            office_document_relationship("rId2", OFFICE_DOCUMENT_TYPE, "xl/workbook.xml"),
        ]
        .concat(),
        "",
    );

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Text("2024-05-13".to_owned())]
    );
}

// calamine's macro get_attrs! stops once it has found as many attributes
// as it looks for, a name written twice counted twice: this relationship
// has the type "x" and no target.
#[test]
fn a_relationship_with_its_type_written_twice_is_not_the_workbook() {
    let read = read_of_1904_date_with_package_relationships(
        &[
            office_document_relationship("rId1", OFFICE_DOCUMENT_TYPE, "xl/workbook.xml"),
            office_document_relationship(
                "rId2",
                &format!(r#"{OFFICE_DOCUMENT_TYPE} Type="x""#),
                "data/workbook.xml",
            ),
        ]
        .concat(),
        "",
    );

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Text("2024-05-13".to_owned())]
    );
}

#[test]
fn a_relationship_after_the_end_of_relationships_is_not_read() {
    let read = read_of_1904_date_with_package_relationships(
        &office_document_relationship("rId1", OFFICE_DOCUMENT_TYPE, "xl/workbook.xml"),
        &office_document_relationship("rId2", OFFICE_DOCUMENT_TYPE, "data/workbook.xml"),
    );

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Text("2024-05-13".to_owned())]
    );
}

// calamine takes a relationship whose type ends in
// /relationships/officeDocument, and not one that ends in officeDocument
// alone.
#[test]
fn a_relationship_whose_type_ends_in_office_document_alone_is_not_the_workbook() {
    let read = read_of_1904_date_with_package_relationships(
        &[
            office_document_relationship("rId1", OFFICE_DOCUMENT_TYPE, "xl/workbook.xml"),
            office_document_relationship(
                "rId2",
                r#"Type="http://example.com/officeDocument""#,
                "data/workbook.xml",
            ),
        ]
        .concat(),
        "",
    );

    assert_eq!(read.unwrap().cells, date_of_1904());
}

// Relationships and Relationship are matched by their local names, the
// names after any prefix.
#[test]
fn relationships_with_a_prefix_give_the_workbook() {
    let read = read_of_1904_date(|parts| {
        for (name, xml) in parts.iter_mut() {
            if name == "_rels/.rels" {
                *xml = xml
                    .replace("<Relationships xmlns=", "<pr:Relationships xmlns:pr=")
                    .replace("<Relationship ", "<pr:Relationship ")
                    .replace("</Relationships>", "</pr:Relationships>");
                assert!(xml.contains("<pr:Relationship "));
            }
        }
    });

    assert_eq!(read.unwrap().cells, date_of_1904());
}

// calamine reads the XML of the workbook without checking that an end tag
// names the element it ends.
#[test]
fn a_workbook_whose_end_tag_names_another_element_is_read() {
    let read = read_of_1904_date(|parts| {
        for (name, xml) in parts.iter_mut() {
            if name == "xl/workbook.xml" {
                *xml = xml.replace("</sheets>", "</sheetz>");
                assert!(xml.contains("</sheetz>"));
            }
        }
    });

    assert_eq!(read.unwrap().cells, date_of_1904());
}

// The workbook is read up to the end of its root, as calamine reads it up
// to </workbook>: a workbookPr after it is not read.
#[test]
fn a_workbook_pr_after_the_end_of_the_root_is_not_read() {
    let read = read_of_1904_date(|parts| {
        for (name, xml) in parts.iter_mut() {
            if name == "xl/workbook.xml" {
                xml.push_str("<x><workbookPr/></x>");
            }
        }
    });

    assert_eq!(read.unwrap().cells, date_of_1904());
}

// A workbook named by _rels/.rels but missing gives the system of 1900,
// and calamine then finds no sheet: the file names a workbook.
#[test]
fn a_workbook_named_but_missing_is_no_visible_worksheet() {
    let read = read_of_1904_date(|parts| parts.retain(|(name, _)| name != "xl/workbook.xml"));

    assert_eq!(
        read,
        Err(ReadError::Unreadable("no visible worksheet".to_owned()))
    );
}

#[test]
fn a_file_whose_package_relationships_name_no_workbook_is_unreadable() {
    let other_type = r#"Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties""#;
    for relationships in [
        String::new(),
        office_document_relationship("rId1", other_type, "xl/workbook.xml"),
        format!(r#"<Relationship Id="rId1" {OFFICE_DOCUMENT_TYPE}/>"#),
    ] {
        let read = read_of_1904_date_with_package_relationships(&relationships, "");

        assert_eq!(
            read,
            Err(ReadError::Unreadable(
                "the file names no workbook".to_owned()
            )),
            "{relationships}"
        );
    }
}

#[test]
fn a_file_with_no_package_relationships_is_unreadable() {
    let read = read_of_1904_date(|parts| parts.retain(|(name, _)| name != "_rels/.rels"));

    assert_eq!(
        read,
        Err(ReadError::Unreadable(
            "the file names no workbook".to_owned()
        ))
    );
}

/// The date of [`read_of_1904_date`] with the parts of `xl/` moved to the
/// folder `folder`, and the target of the workbook in `_rels/.rels`
/// written `target`, as the XML writes it.
fn read_of_1904_date_in_folder(folder: &str, target: &str) -> Result<Sheet, ReadError> {
    read_of_1904_date(|parts| {
        for (name, xml) in parts.iter_mut() {
            if let Some(name_in_folder) = name.strip_prefix("xl/") {
                *name = format!("{folder}{name_in_folder}");
            }
            if name == "_rels/.rels" {
                *xml = xml.replace(
                    r#"Target="xl/workbook.xml""#,
                    &format!(r#"Target="{target}""#),
                );
                assert!(xml.contains(target));
            }
        }
    })
}

// calamine looks for the workbook, the folder of the target followed by
// workbook.xml, ignoring case.
#[test]
fn a_target_of_the_workbook_in_capitals_is_found() {
    let read = read_of_1904_date_in_folder("xl/", "XL/Workbook.xml");

    assert_eq!(read.unwrap().cells, date_of_1904());
}

// calamine reads the entities of the target, as xlsx_rs does: the folder
// is a&b/, written a&amp;b/.
#[test]
fn the_entities_of_the_target_of_the_workbook_are_read() {
    let read = read_of_1904_date_in_folder("a&b/", "a&amp;b/workbook.xml");

    assert_eq!(read.unwrap().cells, date_of_1904());
}

/// `xml` with spaces after its declaration, `<?xml ... ?>`, up to
/// `num_bytes` bytes, which XML reads as it read `xml`; `None` when `xml`
/// has no declaration or more bytes than that.
fn padded(xml: &str, num_bytes: usize) -> Option<String> {
    let declaration_end = xml.find("?>")?.checked_add(2)?;
    let (declaration, rest) = xml.split_at(declaration_end);
    let num_spaces = num_bytes.checked_sub(xml.len())?;
    Some(format!("{declaration}{}{rest}", " ".repeat(num_spaces)))
}

/// The parts calamine reads whole when it opens a file, besides the table
/// of texts: the package relationships, which give the folder of the
/// workbook; the workbook; its relationships; and the styles.
const SETTINGS_PARTS: [&str; 4] = [
    "_rels/.rels",
    "xl/workbook.xml",
    "xl/_rels/workbook.xml.rels",
    "xl/styles.xml",
];

/// The sheet of one cell, A1, the number 7, in a workbook of the 1904
/// system whose part `part_name` is padded to `num_bytes` bytes;
/// `None` when there is no such part, or it has more bytes than that.
fn read_with_part_of(part_name: &str, num_bytes: usize) -> Option<Result<Sheet, ReadError>> {
    let mut parts = parts_of_1904_worksheet(
        r#"<sheetData><row r="1"><c r="A1"><v>7</v></c></row></sheetData>"#,
        &[],
    );
    let xml = xml_of(&mut parts, part_name)?;
    *xml = padded(xml, num_bytes)?;
    Some(read_first_sheet(&xlsx_of_parts(&parts), MAX_SHEET_CELLS))
}

#[test]
fn a_part_calamine_reads_whole_past_50_000_000_bytes_is_unreadable() {
    for part_name in SETTINGS_PARTS {
        let read = read_with_part_of(part_name, 50_000_001).unwrap();

        assert_eq!(
            read,
            Err(ReadError::Unreadable(
                "a part of the file is too large".to_owned()
            )),
            "{part_name}"
        );
    }
}

#[test]
fn a_part_calamine_reads_whole_of_50_000_000_bytes_is_read() {
    for part_name in SETTINGS_PARTS {
        let read = read_with_part_of(part_name, 50_000_000).unwrap();

        assert_eq!(read.unwrap().cells, [SheetCell::Number(7.0)], "{part_name}");
    }
}

/// The sheet of one cell, A1, the number 7, in a workbook of the 1904
/// system with a copy of its part `part_name` renamed `copy_name`, padded
/// to `num_bytes` bytes; `None` when there is no such part, or it has
/// more bytes than that.
fn read_with_copy_of_part(
    part_name: &str,
    copy_name: &str,
    num_bytes: usize,
) -> Option<Result<Sheet, ReadError>> {
    let mut parts = parts_of_1904_worksheet(
        r#"<sheetData><row r="1"><c r="A1"><v>7</v></c></row></sheetData>"#,
        &[],
    );
    let xml = xml_of(&mut parts, part_name)?.clone();
    parts.push((copy_name.to_owned(), padded(&xml, num_bytes)?));
    Some(read_first_sheet(&xlsx_of_parts(&parts), MAX_SHEET_CELLS))
}

// calamine reads the parts of the folder the package relationships give,
// which xlsx_rs does not look for: every part whose name ends as one of
// them is held to their bound, `\` read as `/` and ignoring case.
#[test]
#[ignore = "takes 19 s in cargo test, past 10 s; run before each release"]
fn a_part_named_as_one_calamine_reads_whole_in_another_folder_is_held_to_the_same_bound() {
    for (part_name, copy_name) in [
        ("xl/workbook.xml", "data/workbook.xml"),
        ("xl/_rels/workbook.xml.rels", "data/_rels/workbook.xml.rels"),
        ("xl/styles.xml", "data/styles.xml"),
        ("xl/styles.xml", "DATA\\STYLES.XML"),
        ("xl/workbook.xml", "otherworkbook.xml"),
        ("_rels/.rels", "_RELS\\.RELS"),
    ] {
        let past_bound = read_with_copy_of_part(part_name, copy_name, 50_000_001).unwrap();
        let at_bound = read_with_copy_of_part(part_name, copy_name, 50_000_000).unwrap();

        assert_eq!(
            past_bound,
            Err(ReadError::Unreadable(
                "a part of the file is too large".to_owned()
            )),
            "{copy_name}"
        );
        assert_eq!(
            at_bound.unwrap().cells,
            [SheetCell::Number(7.0)],
            "{copy_name}"
        );
    }
}

/// The sheet whose A1 and B1 are the texts 0 and 1 of a table of texts,
/// `xl/sharedStrings.xml` with `shared_strings_xml`, in the parts of
/// [`parts_of_worksheet`].
fn parts_with_texts(shared_strings_xml: String) -> Vec<(String, String)> {
    let mut parts = parts_of_worksheet(
        r#"<sheetData><row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c></row></sheetData>"#,
    );
    parts.push(("xl/sharedStrings.xml".to_owned(), shared_strings_xml));
    parts
}

/// The sheet of [`parts_with_texts`], its table of texts `"id"` and
/// `"pop"` with the attributes `sst_attributes`.
fn read_with_texts(sst_attributes: &str) -> Result<Sheet, ReadError> {
    let parts = parts_with_texts(shared_strings(sst_attributes, &["id", "pop"]));
    read_first_sheet(&xlsx_of_parts(&parts), MAX_SHEET_CELLS)
}

/// The cells of the sheet of [`read_with_texts`].
fn the_two_texts() -> Vec<SheetCell> {
    vec![
        SheetCell::Text("id".to_owned()),
        SheetCell::Text("pop".to_owned()),
    ]
}

/// A table of texts of `num_texts` texts, the first two `id` and `pop` and
/// the others empty, `<si/>`, with no `uniqueCount`.
fn shared_strings_of_num_texts(num_texts: usize) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><si><t>id</t></si><si><t>pop</t></si>{}</sst>"#,
        "<si/>".repeat(num_texts.saturating_sub(2))
    )
}

#[test]
#[ignore = "takes 12 s in cargo test, past 10 s; run before each release"]
fn a_table_of_more_than_10_000_000_texts_is_unreadable() {
    let bytes = xlsx_of_parts(&parts_with_texts(shared_strings_of_num_texts(10_000_001)));

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable("too many texts".to_owned()))
    );
}

#[test]
#[ignore = "takes 19 s in cargo test, past 10 s; run before each release"]
fn a_table_of_10_000_000_texts_is_read() {
    let bytes = xlsx_of_parts(&parts_with_texts(shared_strings_of_num_texts(10_000_000)));

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(read.unwrap().cells, the_two_texts());
}

// calamine reserves room for as many texts as the first uniqueCount of the
// first sst says, before it reads the first text, and whatever the texts
// that follow: 400,000,000 trapped the package.
#[test]
fn a_table_of_texts_whose_unique_count_passes_10_000_000_is_unreadable() {
    for unique_count in [
        "10000001",
        "400000000",
        "00010000001",
        "4294967296",
        "123456789012345678901234567890",
    ] {
        let read = read_with_texts(&format!(r#"uniqueCount="{unique_count}""#));

        assert_eq!(
            read,
            Err(ReadError::Unreadable("too many texts".to_owned())),
            "uniqueCount {unique_count}"
        );
    }
}

#[test]
fn a_table_of_texts_cut_short_after_a_unique_count_past_10_000_000_is_unreadable() {
    let shared_strings_xml = shared_strings(r#"uniqueCount="400000000""#, &["id", "pop"]);
    let cut_short = shared_strings_xml.replace("</sst>", "");

    let read = read_first_sheet(
        &xlsx_of_parts(&parts_with_texts(cut_short)),
        MAX_SHEET_CELLS,
    );

    assert_eq!(
        read,
        Err(ReadError::Unreadable("too many texts".to_owned()))
    );
}

// Every uniqueCount of every sst is read, and every si counted, in any part
// whose name ends in sharedStrings.xml: more than calamine can hold.
#[test]
fn a_unique_count_past_10_000_000_anywhere_in_a_table_of_texts_is_unreadable() {
    let two_texts = shared_strings(r#"uniqueCount="2""#, &["id", "pop"]);
    for shared_strings_xml in [
        two_texts.replace("</sst>", r#"</sst><sst uniqueCount="400000000"/>"#),
        two_texts.replace(
            r#"uniqueCount="2""#,
            r#"uniqueCount="2" uniqueCount="400000000""#,
        ),
        two_texts.replace("<si>", r#"<x:sst uniqueCount="400000000"/><si>"#),
    ] {
        let read = read_first_sheet(
            &xlsx_of_parts(&parts_with_texts(shared_strings_xml.clone())),
            MAX_SHEET_CELLS,
        );

        assert_eq!(
            read,
            Err(ReadError::Unreadable("too many texts".to_owned())),
            "{shared_strings_xml}"
        );
    }
}

#[test]
fn a_table_of_texts_in_another_folder_is_counted() {
    let mut parts = parts_with_texts(shared_strings(r#"uniqueCount="2""#, &["id", "pop"]));
    parts.push((
        "data/sharedStrings.xml".to_owned(),
        shared_strings(r#"uniqueCount="400000000""#, &[]),
    ));

    let read = read_first_sheet(&xlsx_of_parts(&parts), MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable("too many texts".to_owned()))
    );
}

// calamine reserves room for 10,000,000 texts, 240 MB natively, and reads
// the two there are.
#[test]
fn a_table_of_texts_whose_unique_count_is_10_000_000_is_read() {
    let read = read_with_texts(r#"uniqueCount="10000000""#);

    assert_eq!(read.unwrap().cells, the_two_texts());
}

// A uniqueCount larger than the texts is a file calamine reads, and the
// room it reserves is within the bound; one with a sign is not a number to
// calamine, which reserves nothing for it.
#[test]
fn a_table_of_texts_whose_unique_count_is_within_10_000_000_missing_or_not_a_number_is_read() {
    for sst_attributes in [
        r#"count="5" uniqueCount="2""#,
        r#"uniqueCount="1""#,
        r#"uniqueCount="1000""#,
        r#"uniqueCount="0010000000""#,
        r#"uniqueCount="0000000000000000010000000""#,
        "",
        r#"uniqueCount="many""#,
        r#"uniqueCount="-3""#,
        r#"uniqueCount="+400000000""#,
        r#"uniqueCount="""#,
        r#"uniquecount="400000000""#,
    ] {
        let read = read_with_texts(sst_attributes);

        assert_eq!(read.unwrap().cells, the_two_texts(), "{sst_attributes}");
    }
}

// calamine finds the table of texts by its name ignoring case, and so does
// xlsx_rs: a table it did not find would reach calamine unchecked.
#[test]
fn a_table_of_texts_named_in_capitals_is_checked() {
    let read_of_unique_count = |unique_count: &str| {
        let mut parts = parts_with_texts(shared_strings(
            &format!(r#"uniqueCount="{unique_count}""#),
            &["id", "pop"],
        ));
        for (name, _) in &mut parts {
            if name == "xl/sharedStrings.xml" {
                *name = "XL/SHAREDSTRINGS.XML".to_owned();
            }
        }
        read_first_sheet(&xlsx_of_parts(&parts), MAX_SHEET_CELLS)
    };

    assert_eq!(read_of_unique_count("2").unwrap().cells, the_two_texts());
    assert_eq!(
        read_of_unique_count("400000000"),
        Err(ReadError::Unreadable("too many texts".to_owned()))
    );
}

/// The sheet of one cell, A1, the number 7, with `num_merged_ranges`
/// merged ranges of two cells each outside its rectangle, D1:E1, D2:E2
/// and so on, in a part named `xl/worksheets/data.xml`, which the
/// relationship of the sheet names as `relationship_target`, read with a
/// limit of 4 cells.
fn read_with_merged_ranges(
    num_merged_ranges: u32,
    relationship_target: &str,
) -> Result<Sheet, ReadError> {
    let merge_cells: String = (1..=num_merged_ranges)
        .map(|row| format!(r#"<mergeCell ref="D{row}:E{row}"/>"#))
        .collect();
    let mut parts = parts_of_worksheet(&format!(
        r#"<sheetData><row r="1"><c r="A1"><v>7</v></c></row></sheetData><mergeCells count="{num_merged_ranges}">{merge_cells}</mergeCells>"#
    ));
    for (name, xml) in &mut parts {
        if name == "xl/worksheets/sheet1.xml" {
            *name = "xl/worksheets/data.xml".to_owned();
        }
        if name == "xl/_rels/workbook.xml.rels" {
            *xml = xml.replace(
                r#"Target="worksheets/sheet1.xml""#,
                &format!(r#"Target="{relationship_target}""#),
            );
        }
    }
    read_first_sheet(&xlsx_of_parts(&parts), 4)
}

// The count is of the bytes `mergeCell` after `<` or `:` in any part, so
// the start of the element `mergeCells` around the ranges counts too: 4
// ranges are 5 with a limit of 4 cells, and 3 ranges are 4.
#[test]
fn a_sheet_with_more_merged_ranges_than_the_limit_of_cells_is_unreadable() {
    for relationship_target in ["worksheets/data.xml", "/xl/worksheets/data.xml"] {
        let read = read_with_merged_ranges(4, relationship_target);

        assert_eq!(
            read,
            Err(ReadError::Unreadable("too many merged ranges".to_owned())),
            "{relationship_target}"
        );
    }
}

#[test]
fn a_sheet_with_as_many_merged_ranges_and_merge_cells_as_the_limit_of_cells_is_read() {
    for relationship_target in ["worksheets/data.xml", "/xl/worksheets/data.xml"] {
        let read = read_with_merged_ranges(3, relationship_target);

        assert_eq!(
            read.unwrap().cells,
            [SheetCell::Number(7.0)],
            "{relationship_target}"
        );
    }
}

/// The merged ranges of [`read_with_merged_ranges`], `num_merged_ranges`
/// of them, D1:E1, D2:E2 and so on, outside the rectangle of A1, in their
/// element `mergeCells`, with the prefix `prefix`, such as `x:`, on each.
fn merge_cells_of(num_merged_ranges: u32, prefix: &str) -> String {
    let merge_cells: String = (1..=num_merged_ranges)
        .map(|row| format!(r#"<{prefix}mergeCell ref="D{row}:E{row}"/>"#))
        .collect();
    format!(r#"<{prefix}mergeCells count="{num_merged_ranges}">{merge_cells}</{prefix}mergeCells>"#)
}

/// The parts of [`parts_of_worksheet`] with the sheet of one cell, A1, the
/// number 7, and a second sheet, `Decoy`, in `xl/worksheets/decoy.xml`,
/// named by the relationship `rId2`; the sheet `Sheet1` has 50 merged
/// ranges outside its rectangle, the decoy none.
fn parts_with_decoy_sheet() -> Vec<(String, String)> {
    let mut parts = parts_of_worksheet(&format!(
        r#"<sheetData><row r="1"><c r="A1"><v>7</v></c></row></sheetData>{}"#,
        merge_cells_of(50, "")
    ));
    parts.push((
        "xl/worksheets/decoy.xml".to_owned(),
        parts_of_one_number()
            .into_iter()
            .find(|(name, _)| name == "xl/worksheets/sheet1.xml")
            .map(|(_, xml)| xml)
            .unwrap_or_default(),
    ));
    for (name, xml) in &mut parts {
        if name == "xl/_rels/workbook.xml.rels" {
            *xml = xml.replace(
                "</Relationships>",
                r#"<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/decoy.xml"/></Relationships>"#,
            );
        }
    }
    parts
}

// calamine reads the content of a defined name as text, and lists no sheet
// in it; xlsx_rs listed this one, of the same name, and counted the merged
// ranges of the decoy, none.
#[test]
fn a_sheet_listed_inside_a_defined_name_does_not_hide_the_merged_ranges_of_the_sheet_read() {
    let mut parts = parts_with_decoy_sheet();
    let workbook = xml_of(&mut parts, "xl/workbook.xml").unwrap();
    *workbook = workbook.replace(
        "<sheets>",
        r#"<definedNames><definedName name="decoy"><sheet name="Sheet1" sheetId="2" r:id="rId2"/></definedName></definedNames><sheets>"#,
    );

    let read = read_first_sheet(&xlsx_of_parts(&parts), 10);

    assert_eq!(
        read,
        Err(ReadError::Unreadable("too many merged ranges".to_owned()))
    );
}

// calamine reads the workbook up to the end tag named workbook; xlsx_rs
// stopped where a stray end tag closed its count of elements open, before
// the sheets, and found no sheet to count. The element <z> left open keeps
// quick-xml from refusing </workbook> as an end tag with nothing open.
#[test]
fn a_stray_end_tag_before_the_sheets_does_not_hide_the_merged_ranges_of_the_sheet_read() {
    let mut parts = parts_with_decoy_sheet();
    let workbook = xml_of(&mut parts, "xl/workbook.xml").unwrap();
    *workbook = workbook
        .replace("<sheets>", "</x><sheets>")
        .replace("</workbook>", "<z></workbook>");

    let read = read_first_sheet(&xlsx_of_parts(&parts), 10);

    assert_eq!(
        read,
        Err(ReadError::Unreadable("too many merged ranges".to_owned()))
    );
}

// A sheet that is not read, hidden here, is counted too: every part is.
#[test]
fn another_sheet_with_more_merged_ranges_than_the_limit_of_cells_is_unreadable() {
    let mut parts = parts_with_decoy_sheet();
    for (name, xml) in &mut parts {
        if name == "xl/workbook.xml" {
            *xml = xml.replace(
                "</sheets>",
                r#"<sheet name="Hidden" sheetId="2" state="hidden" r:id="rId2"/></sheets>"#,
            );
        }
        if name == "xl/worksheets/sheet1.xml" {
            *xml = xml.replace(&merge_cells_of(50, ""), "");
        }
        if name == "xl/worksheets/decoy.xml" {
            *xml = xml.replace(
                "</worksheet>",
                &format!("{}</worksheet>", merge_cells_of(50, "")),
            );
        }
    }

    let read = read_first_sheet(&xlsx_of_parts(&parts), 10);

    assert_eq!(
        read,
        Err(ReadError::Unreadable("too many merged ranges".to_owned()))
    );
}

#[test]
fn merged_ranges_with_a_prefix_are_counted() {
    let mut parts = parts_with_decoy_sheet();
    for (_, xml) in &mut parts {
        *xml = xml
            .replace(&merge_cells_of(50, ""), &merge_cells_of(50, "x:"))
            .replace(
                "<worksheet xmlns=",
                r#"<worksheet xmlns:x="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns="#,
            );
    }
    assert!(parts.iter().any(|(_, xml)| xml.contains("<x:mergeCell ")));

    let read = read_first_sheet(&xlsx_of_parts(&parts), 10);

    assert_eq!(
        read,
        Err(ReadError::Unreadable("too many merged ranges".to_owned()))
    );
}

/// A part of a zip, a name and its bytes, given as `head`, then `middle`
/// written `num_middles` times, then `tail`, so that a part of hundreds of
/// MB is deflated without being held whole.
struct RepeatedPart<'part> {
    name: &'part str,
    head: &'part str,
    middle: &'part str,
    num_middles: u64,
    tail: &'part str,
}

/// A zip of `parts`, each deflated, the compression Excel uses.
fn deflated_zip(parts: &[RepeatedPart<'_>]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .large_file(true);
    for part in parts {
        writer.start_file(part.name, options)?;
        writer.write_all(part.head.as_bytes())?;
        // The middle written 4,096 times at once, so that the writer is not
        // called once for each of millions of elements.
        let middles_of_a_chunk = part.middle.repeat(4_096);
        let num_chunks = part.num_middles / 4_096;
        for _ in 0..num_chunks {
            writer.write_all(middles_of_a_chunk.as_bytes())?;
        }
        for _ in 0..part.num_middles % 4_096 {
            writer.write_all(part.middle.as_bytes())?;
        }
        writer.write_all(part.tail.as_bytes())?;
    }
    Ok(writer.finish()?.into_inner())
}

/// The parts of [`parts_of_worksheet`] with the sheet of one cell, A1, the
/// number 7, as the [`RepeatedPart`]s of [`deflated_zip`], none repeated.
fn parts_of_one_number() -> Vec<(String, String)> {
    parts_of_worksheet(r#"<sheetData><row r="1"><c r="A1"><v>7</v></c></row></sheetData>"#)
}

/// `parts` as [`RepeatedPart`]s that repeat nothing.
fn unrepeated(parts: &[(String, String)]) -> Vec<RepeatedPart<'_>> {
    parts
        .iter()
        .map(|(name, xml)| RepeatedPart {
            name,
            head: xml,
            middle: "",
            num_middles: 0,
            tail: "",
        })
        .collect()
}

/// A form feed, the byte `0C`, which calamine's reader of attributes takes
/// for a space before the name of an attribute, and quick-xml's for part
/// of the name.
const FORM_FEED: &str = "\x0C";

// The three files of the review of 28 September 2026 that trapped the
// package, each with a form feed before the name of an attribute, which
// calamine read and xlsx_rs, reading the attributes with quick-xml, did not.
#[test]
fn a_unique_count_after_a_form_feed_is_read_as_calamine_reads_it() {
    let mut parts = parts_with_texts(shared_strings(
        &format!(r#"count="2" {FORM_FEED}uniqueCount="400000000""#),
        &["id", "pop"],
    ));
    parts.sort_by_key(|(name, _)| name == "xl/sharedStrings.xml");
    let bytes = deflated_zip(&unrepeated(&parts)).unwrap();

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable("too many texts".to_owned()))
    );
}

#[test]
fn a_type_of_the_package_relationships_after_a_form_feed_is_read_as_calamine_reads_it() {
    let mut parts = parts_with_texts(shared_strings(
        r#"count="2" uniqueCount="400000000""#,
        &["id", "pop"],
    ));
    let package_relationships = xml_of(&mut parts, "_rels/.rels").unwrap();
    *package_relationships =
        package_relationships.replace(r#" Type="#, &format!(r#" {FORM_FEED}Type="#));
    assert!(package_relationships.contains(FORM_FEED));
    let bytes = deflated_zip(&unrepeated(&parts)).unwrap();

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable("too many texts".to_owned()))
    );
}

// The same file with nothing else hostile: xlsx_rs finds the workbook, and
// its date system, only by reading the Type as calamine does.
#[test]
fn a_type_after_a_form_feed_gives_the_workbook_and_its_date_system() {
    let read = read_of_1904_date(|parts| {
        for (name, xml) in parts.iter_mut() {
            if name == "_rels/.rels" {
                *xml = xml.replace(" Type=", &format!(" {FORM_FEED}Type="));
                assert!(xml.contains(FORM_FEED));
            }
        }
    });

    assert_eq!(read.unwrap().cells, date_of_1904());
}

/// The xlsx of [`parts_of_one_number`] whose sheet has a form feed before
/// the attribute `name` in the workbook, and `num_merged_ranges` merged
/// ranges, each `D1:E1`, outside its rectangle, deflated.
fn xlsx_of_form_feed_name(num_merged_ranges: u64) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut parts = parts_of_one_number();
    let workbook = xml_of(&mut parts, "xl/workbook.xml").ok_or("no workbook")?;
    *workbook = workbook.replace(r#" name="#, &format!(r#" {FORM_FEED}name="#));
    let sheet = xml_of(&mut parts, "xl/worksheets/sheet1.xml").ok_or("no sheet")?;
    let (sheet_head, sheet_tail) = sheet
        .split_once("</worksheet>")
        .ok_or("no end of the sheet")?;
    let sheet_head = format!("{sheet_head}<mergeCells>");
    let sheet_tail = format!("</mergeCells></worksheet>{sheet_tail}");
    let mut repeated_parts = unrepeated(&parts);
    let merged_part = repeated_parts
        .iter_mut()
        .find(|part| part.name == "xl/worksheets/sheet1.xml")
        .ok_or("no sheet")?;
    merged_part.head = &sheet_head;
    merged_part.middle = r#"<mergeCell ref="D1:E1"/>"#;
    merged_part.num_middles = num_merged_ranges;
    merged_part.tail = &sheet_tail;
    deflated_zip(&repeated_parts)
}

#[test]
fn a_name_of_the_sheet_after_a_form_feed_is_read_as_calamine_reads_it() {
    let bytes = xlsx_of_form_feed_name(50).unwrap();

    let read = read_first_sheet(&bytes, 10);

    assert_eq!(
        read,
        Err(ReadError::Unreadable("too many merged ranges".to_owned()))
    );
}

// The file of the review held 36,000,000 merged ranges, which calamine
// held in 583 MB natively.
#[test]
#[ignore = "takes 15 to 55 s in cargo test, deflating 864 MB of XML, past 10 s; run before each release"]
fn a_name_of_the_sheet_after_a_form_feed_with_36_000_000_merged_ranges_is_unreadable() {
    let bytes = xlsx_of_form_feed_name(36_000_000).unwrap();

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable("too many merged ranges".to_owned()))
    );
}

/// The parts of [`parts_of_one_number`] whose workbook lists `num_sheets`
/// sheets, `s1`, `s2` and so on, each naming the relationship `rId1`, and
/// whose relationship `rId1` has the target `target` and the attributes
/// `other_attributes` after it.
fn parts_with_sheets_of_one_target(
    num_sheets: u32,
    target: &str,
    other_attributes: &str,
) -> Vec<(String, String)> {
    let sheets: String = (1..=num_sheets)
        .map(|sheet| format!(r#"<sheet name="s{sheet}" sheetId="{sheet}" r:id="rId1"/>"#))
        .collect();
    let mut parts = parts_of_one_number();
    for (name, xml) in &mut parts {
        if name == "xl/workbook.xml" {
            *xml = xml.replace(r#"<sheet name="Sheet1" sheetId="1" r:id="rId1"/>"#, &sheets);
        }
        if name == "xl/_rels/workbook.xml.rels" {
            *xml = xml.replace(
                r#"Target="worksheets/sheet1.xml""#,
                &format!(r#"Target="{target}"{other_attributes}"#),
            );
        }
    }
    parts
}

/// The number of bytes of the longest tag of `xml`, from `<` to `>`.
fn longest_tag_of(xml: &str) -> usize {
    xml.split('<')
        .skip(1)
        .map(|tag| tag.find('>').map_or(0, |end| end.saturating_add(2)))
        .max()
        .unwrap_or(0)
}

// calamine keeps the path of the part of each sheet the workbook lists, the
// folder of the workbook followed by the target of the sheet's
// relationship: 2,000 sheets naming one target of 500 KB, a zip of 8,586
// bytes, took 1.0 GB natively in the spec's review of 28 September 2026.
#[test]
fn a_workbook_of_2_000_sheets_naming_one_target_of_500_kb_is_unreadable() {
    let target = format!("worksheets/{}sheet1.xml", "a/".repeat(250_000));
    let parts = parts_with_sheets_of_one_target(2_000, &target, "");

    let read = read_first_sheet(&xlsx_of_parts(&parts), MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable(
            "the workbook lists too many sheets".to_owned()
        ))
    );
}

/// The sheet of [`parts_with_sheets_of_one_target`] with 999 sheets, which
/// with the element `sheets` around them are 1,000 counted, and the tag of
/// its relationship padded with an attribute so that it and the longest tag
/// of `_rels/.rels` hold `sum_of_tags` bytes together; the part
/// `first_part_name` first in the zip, the others in their usual order.
fn read_with_1_000_sheets_and_tags_of(
    sum_of_tags: usize,
    first_part_name: &str,
) -> Result<Sheet, ReadError> {
    let bare_parts = parts_with_sheets_of_one_target(999, "worksheets/sheet1.xml", r#" pad="""#);
    let tag_bytes_of = |part_name: &str| {
        bare_parts
            .iter()
            .find(|(name, _)| name == part_name)
            .map_or(0, |(_, xml)| longest_tag_of(xml))
    };
    let bare_sum =
        tag_bytes_of("_rels/.rels").saturating_add(tag_bytes_of("xl/_rels/workbook.xml.rels"));
    let padding = "a".repeat(sum_of_tags.saturating_sub(bare_sum));
    let mut parts = parts_with_sheets_of_one_target(
        999,
        "worksheets/sheet1.xml",
        &format!(r#" pad="{padding}""#),
    );
    parts.sort_by_key(|(name, _)| name != first_part_name);
    read_first_sheet(&xlsx_of_parts(&parts), MAX_SHEET_CELLS)
}

// The largest count of the bytes sheet after < or : in a workbook, 1,000,
// times the sum of the two longest tags of the relationships, 100,000 or
// 100,001 bytes: 100,000,000 bytes, MAX_SHEET_PATH_BYTES, read, and
// 100,001,000 refused. The longer tag, of the relationships of the
// workbook, comes after the shorter one in the zip and before it.
#[test]
fn the_paths_of_the_sheets_are_bounded_at_100_000_000_bytes() {
    for first_part_name in ["[Content_Types].xml", "xl/_rels/workbook.xml.rels"] {
        let at_bound = read_with_1_000_sheets_and_tags_of(100_000, first_part_name);
        let past_bound = read_with_1_000_sheets_and_tags_of(100_001, first_part_name);

        assert_eq!(at_bound.unwrap().name, "s1", "{first_part_name}");
        assert_eq!(
            past_bound,
            Err(ReadError::Unreadable(
                "the workbook lists too many sheets".to_owned()
            )),
            "{first_part_name}"
        );
    }
}

// calamine decodes the texts of a part in the encoding it declares, and
// one declared windows-1252 turns a byte 80 into €, 3 bytes of UTF-8, so
// every bound counted in the bytes of the part would hold 3 times as much.
#[test]
fn a_part_calamine_reads_whole_declared_in_windows_1252_is_unreadable() {
    for part_name in [
        "_rels/.rels",
        "xl/workbook.xml",
        "xl/_rels/workbook.xml.rels",
        "xl/styles.xml",
        "xl/sharedStrings.xml",
    ] {
        let mut parts = parts_of_1904_worksheet(
            r#"<sheetData><row r="1"><c r="A1"><v>7</v></c></row></sheetData>"#,
            &[],
        );
        parts.push((
            "xl/sharedStrings.xml".to_owned(),
            shared_strings(r#"uniqueCount="1""#, &["id"]),
        ));
        let xml = xml_of(&mut parts, part_name).unwrap();
        *xml = xml.replace(r#"encoding="UTF-8""#, r#"encoding="windows-1252""#);
        assert!(xml.contains("windows-1252"), "{part_name}");

        let read = read_first_sheet(&xlsx_of_parts(&parts), MAX_SHEET_CELLS);

        assert_eq!(
            read,
            Err(ReadError::Unreadable(
                "a part of the file is not in UTF-8".to_owned()
            )),
            "{part_name}"
        );
    }
}

/// The xlsx of [`parts_with_texts`] whose table of texts, of `id` and
/// `pop`, is written as `bytes_of_xml` gives the bytes of its XML.
fn xlsx_with_texts_as(bytes_of_xml: impl Fn(&str) -> Vec<u8>) -> Vec<u8> {
    let parts = parts_with_texts(shared_strings(r#"uniqueCount="2""#, &["id", "pop"]));
    let files: Vec<(&str, Vec<u8>)> = parts
        .iter()
        .map(|(name, xml)| {
            if name == "xl/sharedStrings.xml" {
                (name.as_str(), bytes_of_xml(xml))
            } else {
                (name.as_str(), xml.as_bytes().to_vec())
            }
        })
        .collect();
    stored_zip(&files)
}

#[test]
fn a_table_of_texts_that_starts_with_the_byte_order_mark_of_utf_16_is_unreadable() {
    let bytes = xlsx_with_texts_as(|xml| {
        let mut utf_16 = vec![0xFF, 0xFE];
        utf_16.extend(xml.encode_utf16().flat_map(u16::to_le_bytes));
        utf_16
    });

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable(
            "a part of the file is not in UTF-8".to_owned()
        ))
    );
}

#[test]
fn a_table_of_texts_that_starts_with_the_byte_order_mark_of_utf_8_is_read() {
    let bytes = xlsx_with_texts_as(|xml| {
        let mut utf_8 = vec![0xEF, 0xBB, 0xBF];
        utf_8.extend(xml.as_bytes());
        utf_8
    });

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(read.unwrap().cells, the_two_texts());
}

/// The parts of [`parts_with_texts`], its table of texts `shared_strings_xml`
/// and its sheet with `merge_cells` after its cells, with every part of
/// `xl/` moved to `data/`, where `_rels/.rels` names the workbook.
fn parts_in_data_folder(shared_strings_xml: String, merge_cells: &str) -> Vec<(String, String)> {
    let mut parts = parts_with_texts(shared_strings_xml);
    for (name, xml) in &mut parts {
        if name == "xl/worksheets/sheet1.xml" {
            *xml = xml.replace("</sheetData>", &format!("</sheetData>{merge_cells}"));
        }
        if let Some(name_in_folder) = name.strip_prefix("xl/") {
            *name = format!("data/{name_in_folder}");
        }
        if name == "_rels/.rels" {
            *xml = xml.replace(
                r#"Target="xl/workbook.xml""#,
                r#"Target="data/workbook.xml""#,
            );
        }
    }
    parts
}

// calamine reads the parts of the folder the package relationships give,
// here data/, and so they are checked as those of xl/.
#[test]
fn a_workbook_in_a_folder_data_has_its_parts_checked_as_those_of_xl() {
    let two_texts = shared_strings(r#"uniqueCount="2""#, &["id", "pop"]);
    let read_in_data = |shared_strings_xml: String, merge_cells: &str, max_cells: u32| {
        let parts = parts_in_data_folder(shared_strings_xml, merge_cells);
        assert!(parts.iter().all(|(name, _)| !name.starts_with("xl/")));
        read_first_sheet(&xlsx_of_parts(&parts), max_cells)
    };

    let read = read_in_data(two_texts.clone(), "", MAX_SHEET_CELLS);
    let unique_count = read_in_data(
        shared_strings(r#"uniqueCount="400000000""#, &["id", "pop"]),
        "",
        MAX_SHEET_CELLS,
    );
    let merged_ranges = read_in_data(two_texts, &merge_cells_of(50, ""), 10);

    assert_eq!(read.unwrap().cells, the_two_texts());
    assert_eq!(
        unique_count,
        Err(ReadError::Unreadable("too many texts".to_owned()))
    );
    assert_eq!(
        merged_ranges,
        Err(ReadError::Unreadable("too many merged ranges".to_owned()))
    );
}

#[test]
fn a_workbook_in_a_folder_data_has_its_settings_parts_held_to_their_bound() {
    for part_name in [
        "data/workbook.xml",
        "data/_rels/workbook.xml.rels",
        "data/styles.xml",
    ] {
        let mut parts =
            parts_in_data_folder(shared_strings(r#"uniqueCount="2""#, &["id", "pop"]), "");
        parts.push((
            "data/styles.xml".to_owned(),
            parts_of_1904_worksheet("", &[])
                .into_iter()
                .find(|(name, _)| name == "xl/styles.xml")
                .map(|(_, xml)| xml)
                .unwrap(),
        ));
        let xml = xml_of(&mut parts, part_name).unwrap();
        *xml = padded(xml, 50_000_001).unwrap();

        let read = read_first_sheet(&xlsx_of_parts(&parts), MAX_SHEET_CELLS);

        assert_eq!(
            read,
            Err(ReadError::Unreadable(
                "a part of the file is too large".to_owned()
            )),
            "{part_name}"
        );
    }
}

/// A zip of the parts of [`parts_of_one_number`] and parts
/// `docProps/padding<i>.xml` of spaces whose sizes make the parts unzip to
/// `num_unzipped_bytes` bytes together, deflated; each padding part holds
/// at most 250,000,000 bytes, within the bound of every part.
fn xlsx_unzipping_to(num_unzipped_bytes: u64) -> Result<Vec<u8>, Box<dyn Error>> {
    const MAX_PADDING_PART_BYTES: u64 = 250_000_000;
    let parts = parts_of_one_number();
    let parts_bytes: u64 = parts
        .iter()
        .map(|(_, xml)| u64::try_from(xml.len()).unwrap_or(u64::MAX))
        .sum();
    let mut num_padding_bytes = num_unzipped_bytes
        .checked_sub(parts_bytes)
        .ok_or("fewer bytes than the parts")?;
    let mut padding_names = Vec::new();
    let mut padding_sizes = Vec::new();
    while num_padding_bytes > 0 {
        let padding_size = num_padding_bytes.min(MAX_PADDING_PART_BYTES);
        padding_names.push(format!("docProps/padding{}.xml", padding_names.len()));
        padding_sizes.push(padding_size);
        num_padding_bytes = num_padding_bytes.saturating_sub(padding_size);
    }
    let mut repeated_parts = unrepeated(&parts);
    for (name, padding_size) in padding_names.iter().zip(padding_sizes) {
        repeated_parts.push(RepeatedPart {
            name,
            head: "",
            middle: " ",
            num_middles: padding_size,
            tail: "",
        });
    }
    deflated_zip(&repeated_parts)
}

#[test]
#[ignore = "takes 70 s in cargo test, deflating and unzipping 2 GB, past 10 s; run before each release"]
fn a_file_that_unzips_to_1_000_000_000_bytes_is_read_and_one_byte_more_refused() {
    let at_bound = read_first_sheet(&xlsx_unzipping_to(1_000_000_000).unwrap(), MAX_SHEET_CELLS);
    let past_bound = read_first_sheet(&xlsx_unzipping_to(1_000_000_001).unwrap(), MAX_SHEET_CELLS);

    assert_eq!(at_bound.unwrap().cells, [SheetCell::Number(7.0)]);
    assert_eq!(
        past_bound,
        Err(ReadError::Unreadable(
            "the file unzips to more than 1,000,000,000 bytes".to_owned()
        ))
    );
}

/// A zip of the parts of [`parts_with_texts`] whose table of texts, of `id`
/// and `pop`, holds `num_bytes` bytes, padded inside its root with
/// elements `<x/>` among spaces, which calamine reads and lets be;
/// deflated.
fn xlsx_with_text_table_of(num_bytes: u64) -> Result<Vec<u8>, Box<dyn Error>> {
    let shared_strings_xml = shared_strings(r#"uniqueCount="2""#, &["id", "pop"]);
    let (head, tail) = shared_strings_xml
        .split_once("</sst>")
        .ok_or("no end of the table")?;
    let tail = format!("</sst>{tail}");
    let middle = format!("{}<x/>", " ".repeat(4_092));
    let bare_bytes = u64::try_from(head.len().saturating_add(tail.len()))?;
    let padding_bytes = num_bytes
        .checked_sub(bare_bytes)
        .ok_or("fewer bytes than the table")?;
    let num_middles = padding_bytes / 4_096;
    let spaces = " ".repeat(usize::try_from(padding_bytes % 4_096)?);
    let head = format!("{head}{spaces}");
    let parts = parts_with_texts(String::new());
    let mut repeated_parts = unrepeated(&parts);
    let text_table = repeated_parts
        .iter_mut()
        .find(|part| part.name == "xl/sharedStrings.xml")
        .ok_or("no table of texts")?;
    text_table.head = &head;
    text_table.middle = &middle;
    text_table.num_middles = num_middles;
    text_table.tail = &tail;
    deflated_zip(&repeated_parts)
}

#[test]
#[ignore = "takes 31 s in cargo test, deflating and reading 600 MB, past 10 s; run before each release"]
fn a_table_of_texts_of_300_000_000_bytes_is_read_and_one_byte_more_refused() {
    let at_bound = read_first_sheet(
        &xlsx_with_text_table_of(300_000_000).unwrap(),
        MAX_SHEET_CELLS,
    );
    let past_bound = read_first_sheet(
        &xlsx_with_text_table_of(300_000_001).unwrap(),
        MAX_SHEET_CELLS,
    );

    assert_eq!(at_bound.unwrap().cells, the_two_texts());
    assert_eq!(
        past_bound,
        Err(ReadError::Unreadable("too much text".to_owned()))
    );
}

/// A zip of the parts of [`parts_of_worksheet`] whose sheet is one cell,
/// A1, written as `cell_head`, then `num_text_bytes` bytes `a`, then
/// `cell_tail`, deflated; its sheet holds `num_text_bytes` bytes more than
/// [`bytes_of_sheet_around`] gives.
fn xlsx_of_one_long_cell(
    cell_head: &str,
    num_text_bytes: u64,
    cell_tail: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let parts = parts_of_worksheet(&format!(
        r#"<sheetData><row r="1">{cell_head}{TEXT_MARK}{cell_tail}</row></sheetData>"#
    ));
    let (_, sheet_xml) = parts
        .iter()
        .find(|(name, _)| name == "xl/worksheets/sheet1.xml")
        .ok_or("no sheet")?;
    let (head, tail) = sheet_xml.split_once(TEXT_MARK).ok_or("no mark")?;
    let mut repeated_parts = unrepeated(&parts);
    let sheet = repeated_parts
        .iter_mut()
        .find(|part| part.name == "xl/worksheets/sheet1.xml")
        .ok_or("no sheet")?;
    sheet.head = head;
    sheet.middle = "a";
    sheet.num_middles = num_text_bytes;
    sheet.tail = tail;
    deflated_zip(&repeated_parts)
}

/// Where [`xlsx_of_one_long_cell`] writes the text of its cell.
const TEXT_MARK: &str = "TEXT_OF_THE_CELL";

/// The bytes of the sheet of [`xlsx_of_one_long_cell`] around the text of
/// its cell, written as `cell_head` and `cell_tail`.
fn bytes_of_sheet_around(cell_head: &str, cell_tail: &str) -> Result<u64, Box<dyn Error>> {
    let parts = parts_of_worksheet(&format!(
        r#"<sheetData><row r="1">{cell_head}{cell_tail}</row></sheetData>"#
    ));
    let (_, sheet_xml) = parts
        .iter()
        .find(|(name, _)| name == "xl/worksheets/sheet1.xml")
        .ok_or("no sheet")?;
    Ok(u64::try_from(sheet_xml.len())?)
}

/// The text of a cell written in the cell, before and after its text.
const INLINE_TEXT_HEAD: &str = r#"<c r="A1" t="inlineStr"><is><t>"#;
const INLINE_TEXT_TAIL: &str = "</t></is></c>";

/// A zip of the sheet of one cell, A1, whose text makes the part of the
/// sheet `num_bytes` bytes long.
fn xlsx_of_sheet_of(num_bytes: u64) -> Result<Vec<u8>, Box<dyn Error>> {
    let num_text_bytes = num_bytes
        .checked_sub(bytes_of_sheet_around(INLINE_TEXT_HEAD, INLINE_TEXT_TAIL)?)
        .ok_or("fewer bytes than the sheet")?;
    xlsx_of_one_long_cell(INLINE_TEXT_HEAD, num_text_bytes, INLINE_TEXT_TAIL)
}

// The sheet at the bound is read by calamine, whose one cell of about
// 300,000,000 bytes of text is past the 200,000,000 bytes of text a read may
// hold.
#[test]
#[ignore = "takes 16 s in cargo test, deflating and reading 600 MB, past 10 s; run before each release"]
fn a_sheet_of_300_000_000_bytes_is_too_much_text_and_one_byte_more_too_large() {
    let at_bound = read_first_sheet(&xlsx_of_sheet_of(300_000_000).unwrap(), MAX_SHEET_CELLS);
    let past_bound = read_first_sheet(&xlsx_of_sheet_of(300_000_001).unwrap(), MAX_SHEET_CELLS);

    assert_eq!(
        at_bound,
        Err(ReadError::Unreadable("too much text".to_owned()))
    );
    assert_eq!(
        past_bound,
        Err(ReadError::Unreadable(
            "a part of the file is too large".to_owned()
        ))
    );
}

// The file of the review of the code of 28 September 2026 that trapped the
// package, written again: one cell of a text, t="str", of 999,000,000
// bytes `a` and an entity, which unzips to less than 1,000,000,000 bytes.
// The zip written here is 972,791 bytes; the review's own was 972,578.
#[test]
#[ignore = "takes 17 s in cargo test, deflating 999 MB, past 10 s; run before each release"]
fn a_cell_of_999_000_000_bytes_is_too_large() {
    let bytes =
        xlsx_of_one_long_cell(r#"<c r="A1" t="str"><v>"#, 999_000_000, "&amp;</v></c>").unwrap();

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable(
            "a part of the file is too large".to_owned()
        ))
    );
}

/// A zip of the parts of [`parts_of_one_number`] and a part
/// `docProps/padding.xml` of `num_bytes` spaces, deflated.
fn xlsx_with_padding_part_of(num_bytes: u64) -> Result<Vec<u8>, Box<dyn Error>> {
    let parts = parts_of_one_number();
    let mut repeated_parts = unrepeated(&parts);
    repeated_parts.push(RepeatedPart {
        name: "docProps/padding.xml",
        head: "",
        middle: " ",
        num_middles: num_bytes,
        tail: "",
    });
    deflated_zip(&repeated_parts)
}

// calamine never opens docProps/padding.xml; it is held to the bound of
// every part all the same, as the stricter cases of the spec say.
#[test]
#[ignore = "takes 14 s in cargo test, deflating and reading 600 MB, past 10 s; run before each release"]
fn a_part_other_than_the_sheet_of_300_000_000_bytes_is_read_and_one_byte_more_too_large() {
    let at_bound = read_first_sheet(
        &xlsx_with_padding_part_of(300_000_000).unwrap(),
        MAX_SHEET_CELLS,
    );
    let past_bound = read_first_sheet(
        &xlsx_with_padding_part_of(300_000_001).unwrap(),
        MAX_SHEET_CELLS,
    );

    assert_eq!(at_bound.unwrap().cells, [SheetCell::Number(7.0)]);
    assert_eq!(
        past_bound,
        Err(ReadError::Unreadable(
            "a part of the file is too large".to_owned()
        ))
    );
}

/// The parts calamine reads whole when it opens a file, the table of texts
/// among them, in a workbook of the 1904 system whose sheet is one cell,
/// A1, the number 7, with a table of texts of one text.
fn parts_with_every_part_calamine_holds() -> Vec<(String, String)> {
    let mut parts = parts_of_1904_worksheet(
        r#"<sheetData><row r="1"><c r="A1"><v>7</v></c></row></sheetData>"#,
        &[],
    );
    parts.push((
        "xl/sharedStrings.xml".to_owned(),
        shared_strings(r#"uniqueCount="1""#, &["id"]),
    ));
    parts
}

/// The five parts calamine reads whole when it opens a file.
const PARTS_CALAMINE_HOLDS: [&str; 5] = [
    "_rels/.rels",
    "xl/workbook.xml",
    "xl/_rels/workbook.xml.rels",
    "xl/styles.xml",
    "xl/sharedStrings.xml",
];

// quick-xml takes the encoding of the first declaration whose encoding it
// knows, and so would read these in UTF-8; the rule of UTF-8 by the bytes
// checks every declaration, and refuses them ("What xlsx_rs reads before
// calamine" of docs/specs/read.md, point 5).
#[test]
fn a_part_calamine_reads_whole_with_a_second_declaration_of_windows_1252_is_unreadable() {
    for part_name in PARTS_CALAMINE_HOLDS {
        let mut parts = parts_with_every_part_calamine_holds();
        let xml = xml_of(&mut parts, part_name).unwrap();
        *xml = xml.replacen(
            "?>",
            r#"?><?xml version="1.0" encoding="windows-1252"?>"#,
            1,
        );

        let read = read_first_sheet(&xlsx_of_parts(&parts), MAX_SHEET_CELLS);

        assert_eq!(
            read,
            Err(ReadError::Unreadable(
                "a part of the file is not in UTF-8".to_owned()
            )),
            "{part_name}"
        );
    }
}

// A declaration inside a comment is found too, one of the files the rule
// cannot tell from one in another encoding.
#[test]
fn a_part_calamine_reads_whole_with_a_declaration_in_a_comment_is_unreadable() {
    for part_name in PARTS_CALAMINE_HOLDS {
        let mut parts = parts_with_every_part_calamine_holds();
        let xml = xml_of(&mut parts, part_name).unwrap();
        *xml = xml.replacen("?>", r#"?><!-- <?xml encoding="latin1"?> -->"#, 1);

        let read = read_first_sheet(&xlsx_of_parts(&parts), MAX_SHEET_CELLS);

        assert_eq!(
            read,
            Err(ReadError::Unreadable(
                "a part of the file is not in UTF-8".to_owned()
            )),
            "{part_name}"
        );
    }
}

// The bytes 3C 00 at the start, `<` in UTF-16 little-endian, with no byte
// order mark and no declaration.
#[test]
fn a_table_of_texts_that_starts_as_utf_16_with_no_mark_is_unreadable() {
    let bytes = xlsx_with_texts_as(|xml| {
        let (_, after_declaration) = xml.split_once("?>").unwrap();
        after_declaration
            .trim_start()
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect()
    });

    let read = read_first_sheet(&bytes, MAX_SHEET_CELLS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable(
            "a part of the file is not in UTF-8".to_owned()
        ))
    );
}

#[test]
fn an_image_declared_in_iso_8859_1_in_a_file_otherwise_read_is_read() {
    let mut parts = parts_of_one_number();
    parts.push((
        "xl/media/a.svg".to_owned(),
        r#"<?xml version="1.0" encoding="iso-8859-1"?><svg xmlns="http://www.w3.org/2000/svg"/>"#
            .to_owned(),
    ));

    let read = read_first_sheet(&xlsx_of_parts(&parts), MAX_SHEET_CELLS);

    assert_eq!(read.unwrap().cells, [SheetCell::Number(7.0)]);
}

// The workbook and the table of texts are refused in another encoding, so
// their reads show the value is taken for UTF-8; the sheet would only be
// counted three times.
#[test]
fn a_part_declared_in_utf8_with_spaces_in_its_value_is_read() {
    for part_name in [
        "xl/workbook.xml",
        "xl/sharedStrings.xml",
        "xl/worksheets/sheet1.xml",
    ] {
        let mut parts = parts_with_every_part_calamine_holds();
        let xml = xml_of(&mut parts, part_name).unwrap();
        *xml = xml.replace(r#"encoding="UTF-8""#, "encoding = ' UTF8 '");
        assert!(xml.contains("' UTF8 '"), "{part_name}");

        let read = read_first_sheet(&xlsx_of_parts(&parts), MAX_SHEET_CELLS);

        assert_eq!(read.unwrap().cells, [SheetCell::Number(7.0)], "{part_name}");
    }
}

/// Where [`xlsx_with_declaration_between_rows`] writes its spaces.
const SPACES_MARK: &str = "SPACES_BETWEEN_THE_ROWS";

/// A zip of the parts of [`parts_of_worksheet`] whose sheet holds A1, 7,
/// and A2, 8, with `declaration` and spaces between the two rows that make
/// the part of the sheet `num_bytes` bytes long, deflated. The first
/// declaration of the sheet names no encoding, so that quick-xml takes the
/// encoding of `declaration`: after one that names an encoding it takes no
/// later one.
fn xlsx_with_declaration_between_rows(
    declaration: &str,
    num_bytes: u64,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut parts = parts_of_worksheet(&format!(
        r#"<sheetData><row r="1"><c r="A1"><v>7</v></c></row>{declaration}{SPACES_MARK}<row r="2"><c r="A2"><v>8</v></c></row></sheetData>"#
    ));
    let sheet_xml = xml_of(&mut parts, "xl/worksheets/sheet1.xml").ok_or("no sheet")?;
    *sheet_xml = sheet_xml.replacen(r#" encoding="UTF-8""#, "", 1);
    if !sheet_xml.starts_with(r#"<?xml version="1.0" standalone="yes"?>"#) {
        return Err("the first declaration still names an encoding".into());
    }
    let (_, sheet_xml) = parts
        .iter()
        .find(|(name, _)| name == "xl/worksheets/sheet1.xml")
        .ok_or("no sheet")?;
    let (head, tail) = sheet_xml.split_once(SPACES_MARK).ok_or("no mark")?;
    let num_spaces = num_bytes
        .checked_sub(u64::try_from(head.len().saturating_add(tail.len()))?)
        .ok_or("fewer bytes than the sheet")?;
    let mut repeated_parts = unrepeated(&parts);
    let sheet = repeated_parts
        .iter_mut()
        .find(|part| part.name == "xl/worksheets/sheet1.xml")
        .ok_or("no sheet")?;
    sheet.head = head;
    sheet.middle = " ";
    sheet.num_middles = num_spaces;
    sheet.tail = tail;
    deflated_zip(&repeated_parts)
}

// A sheet in UTF-8 whose first declaration names no encoding, with a
// second declaration of windows-1252 between two rows, gave calamine the
// second row decoded in windows-1252, in a trial of 29 September 2026; it
// is counted three times against the bound of 300,000,000 bytes, and so is
// one whose declaration is behind a decoy or a value of version of
// 1,000,000 bytes.
#[test]
#[ignore = "takes 15 s in cargo test, deflating and reading 600 MB, past 10 s; run before each release"]
fn a_sheet_with_a_second_declaration_of_windows_1252_is_bounded_at_100_000_000_bytes() {
    let version_of_1_000_000_bytes = format!(
        r#"<?xml version="{}" encoding="windows-1252"?>"#,
        "1".repeat(1_000_000)
    );
    for declaration in [
        r#"<?xml version="1.0" encoding="windows-1252"?>"#,
        r#"<?xml version="1.0" foo="encoding='utf-8'" encoding="windows-1252"?>"#,
        &version_of_1_000_000_bytes,
    ] {
        let at_bound = read_first_sheet(
            &xlsx_with_declaration_between_rows(declaration, 100_000_000).unwrap(),
            MAX_SHEET_CELLS,
        );
        let past_bound = read_first_sheet(
            &xlsx_with_declaration_between_rows(declaration, 100_000_001).unwrap(),
            MAX_SHEET_CELLS,
        );

        assert_eq!(
            at_bound.unwrap().cells,
            [SheetCell::Number(7.0), SheetCell::Number(8.0)],
            "{}",
            declaration.get(..60).unwrap_or(declaration)
        );
        assert_eq!(
            past_bound,
            Err(ReadError::Unreadable(
                "a part of the file is too large".to_owned()
            )),
            "{}",
            declaration.get(..60).unwrap_or(declaration)
        );
    }
}

/// A zip of the parts of [`parts_of_one_number`] whose sheet the
/// relationships of the workbook name `worksheets/sharedStrings.xml`, a
/// name that ends as a table of texts does, padded with spaces after its
/// declaration to `num_bytes` bytes, deflated.
fn xlsx_with_sheet_named_as_a_table_of_texts(num_bytes: u64) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut parts = parts_of_one_number();
    for (name, xml) in &mut parts {
        if name == "xl/worksheets/sheet1.xml" {
            "xl/worksheets/sharedStrings.xml".clone_into(name);
        }
        if name == "xl/_rels/workbook.xml.rels" {
            *xml = xml.replace("worksheets/sheet1.xml", "worksheets/sharedStrings.xml");
        }
    }
    let (_, sheet_xml) = parts
        .iter()
        .find(|(name, _)| name == "xl/worksheets/sharedStrings.xml")
        .ok_or("no sheet")?;
    let declaration_end = sheet_xml
        .find("?>")
        .and_then(|position| position.checked_add(2))
        .ok_or("no declaration")?;
    let (head, tail) = sheet_xml.split_at(declaration_end);
    let num_spaces = num_bytes
        .checked_sub(u64::try_from(sheet_xml.len())?)
        .ok_or("fewer bytes than the sheet")?;
    let mut repeated_parts = unrepeated(&parts);
    let sheet = repeated_parts
        .iter_mut()
        .find(|part| part.name == "xl/worksheets/sharedStrings.xml")
        .ok_or("no sheet")?;
    sheet.head = head;
    sheet.middle = " ";
    sheet.num_middles = num_spaces;
    sheet.tail = tail;
    deflated_zip(&repeated_parts)
}

// The relationships of the workbook can give a sheet any name, one that
// ends as a table of texts does among them; the bound of every part holds
// it all the same.
#[test]
#[ignore = "takes 26 s in cargo test, deflating and reading 600 MB, past 10 s; run before each release"]
fn a_sheet_named_as_a_table_of_texts_of_300_000_001_bytes_is_refused() {
    let at_bound = read_first_sheet(
        &xlsx_with_sheet_named_as_a_table_of_texts(300_000_000).unwrap(),
        MAX_SHEET_CELLS,
    );
    let past_bound = read_first_sheet(
        &xlsx_with_sheet_named_as_a_table_of_texts(300_000_001).unwrap(),
        MAX_SHEET_CELLS,
    );

    assert_eq!(at_bound.unwrap().cells, [SheetCell::Number(7.0)]);
    assert_eq!(
        past_bound,
        Err(ReadError::Unreadable("too much text".to_owned()))
    );
}
