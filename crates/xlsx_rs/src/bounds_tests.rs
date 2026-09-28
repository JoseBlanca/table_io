//! The messages of the bounds of "What xlsx_rs reads before calamine" of
//! `docs/specs/read.md`, tested with files of a few KB through
//! `read_first_sheet_within`, which takes the bounds as an argument: a file
//! that reaches the real bounds unzips to hundreds of MB. The files are
//! written by the helper of `tests/hand_written/`, which the tests of
//! `tests/` share, taken here by its path.

#[expect(
    dead_code,
    reason = "the tests of the bounds use only the zip of the parts and the workbooks of one sheet"
)]
#[path = "../tests/hand_written/mod.rs"]
mod hand_written;

use std::io::Cursor;

use rust_xlsxwriter::{Workbook, XlsxError};
use zip::ZipArchive;
use zip::result::ZipError;

use crate::bounds_tests::hand_written::{
    parts_of_1904_worksheet, parts_of_worksheet, shared_strings, xlsx_of_parts,
};
use crate::parts::PartBounds;
use crate::{
    MAX_SETTINGS_PART_BYTES, MAX_SHEET_PATH_BYTES, MAX_TEXT_BYTES, MAX_TEXT_TABLE_BYTES, MAX_TEXTS,
    MAX_UNZIPPED_BYTES, PART_BOUNDS, ReadError, Sheet, SheetCell, read_first_sheet_within,
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

/// The number of bytes the parts of `bytes`, an xlsx, hold unzipped, as
/// the zip's directory gives them.
fn unzipped_size_of(bytes: &[u8]) -> Result<u64, ZipError> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))?;
    let mut unzipped_size: u64 = 0;
    for part_index in 0..archive.len() {
        let part_size = archive.by_index(part_index)?.size();
        unzipped_size = unzipped_size.saturating_add(part_size);
    }
    Ok(unzipped_size)
}

/// The bounds of `read_first_sheet`, with `change` made to them.
fn bounds_with(change: impl FnOnce(&mut PartBounds)) -> PartBounds {
    let mut bounds = PART_BOUNDS;
    change(&mut bounds);
    bounds
}

#[test]
fn a_file_that_unzips_to_more_bytes_than_the_bound_is_unreadable() {
    let bytes = xlsx_of_individuals().unwrap();
    assert!(unzipped_size_of(&bytes).unwrap() > 4_000);

    let read = read_first_sheet_within(
        &bytes,
        MAX_SHEET_CELLS,
        bounds_with(|bounds| bounds.max_unzipped_bytes = 4_000),
    );

    assert_eq!(
        read,
        Err(ReadError::Unreadable(
            "the file unzips to more than 4,000 bytes".to_owned()
        ))
    );
}

#[test]
fn a_file_that_unzips_to_as_many_bytes_as_the_bound_is_read() {
    let bytes = xlsx_of_individuals().unwrap();
    let unzipped_size = unzipped_size_of(&bytes).unwrap();

    let bounded_read = read_first_sheet_within(
        &bytes,
        MAX_SHEET_CELLS,
        bounds_with(|bounds| bounds.max_unzipped_bytes = unzipped_size),
    );
    let one_byte_less = read_first_sheet_within(
        &bytes,
        MAX_SHEET_CELLS,
        bounds_with(|bounds| bounds.max_unzipped_bytes = unzipped_size - 1),
    );

    let sheet = bounded_read.unwrap();
    assert_eq!(sheet.cells[0], SheetCell::Text("id".to_owned()));
    assert_eq!(sheet.num_rows, 201);
    assert!(matches!(one_byte_less, Err(ReadError::Unreadable(_))));
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

/// The cells of the sheet of [`parts_with_texts`] with the texts `id` and
/// `pop`.
fn the_two_texts() -> Vec<SheetCell> {
    vec![
        SheetCell::Text("id".to_owned()),
        SheetCell::Text("pop".to_owned()),
    ]
}

/// What `read_first_sheet_within` gives for `parts`, zipped, within
/// `bounds`.
fn read_within(parts: &[(String, String)], bounds: PartBounds) -> Result<Sheet, ReadError> {
    read_first_sheet_within(&xlsx_of_parts(parts), MAX_SHEET_CELLS, bounds)
}

#[test]
fn a_table_of_texts_past_the_bound_of_its_bytes_is_too_much_text() {
    let shared_strings_xml = shared_strings(r#"uniqueCount="2""#, &["id", "pop"]);
    let num_bytes = u64::try_from(shared_strings_xml.len()).unwrap();
    let parts = parts_with_texts(shared_strings_xml);

    let past_bound = read_within(
        &parts,
        bounds_with(|bounds| bounds.max_text_table_bytes = num_bytes - 1),
    );
    let at_bound = read_within(
        &parts,
        bounds_with(|bounds| bounds.max_text_table_bytes = num_bytes),
    );

    assert_eq!(
        past_bound,
        Err(ReadError::Unreadable("too much text".to_owned()))
    );
    assert_eq!(at_bound.unwrap().cells, the_two_texts());
}

#[test]
fn a_table_of_texts_in_another_folder_is_held_to_the_bound_of_its_bytes() {
    let shared_strings_xml = shared_strings(r#"uniqueCount="2""#, &["id", "pop"]);
    let num_bytes = u64::try_from(shared_strings_xml.len()).unwrap();
    let mut parts = parts_with_texts(shared_strings_xml.clone());
    parts.push((
        "data/sharedStrings.xml".to_owned(),
        format!("{shared_strings_xml} "),
    ));

    let read = read_within(
        &parts,
        bounds_with(|bounds| bounds.max_text_table_bytes = num_bytes),
    );

    assert_eq!(read, Err(ReadError::Unreadable("too much text".to_owned())));
}

// The bounds read_first_sheet reads within, at the values of
// docs/specs/read.md: the tests at these values unzip hundreds of MB, and
// some are ignored, so that a bound given to the wrong field is caught here.
#[test]
fn the_bounds_of_read_first_sheet_are_those_of_the_spec() {
    assert_eq!(MAX_UNZIPPED_BYTES, 1_000_000_000);
    assert_eq!(MAX_SETTINGS_PART_BYTES, 50_000_000);
    assert_eq!(MAX_TEXT_BYTES, 200_000_000);
    assert_eq!(MAX_TEXT_TABLE_BYTES, 400_000_000);
    assert_eq!(MAX_TEXTS, 10_000_000);
    assert_eq!(MAX_SHEET_PATH_BYTES, 100_000_000);

    let PartBounds {
        max_unzipped_bytes,
        max_settings_part_bytes,
        max_text_table_bytes,
        max_texts,
        max_sheet_path_bytes,
    } = PART_BOUNDS;
    assert_eq!(max_unzipped_bytes, 1_000_000_000);
    assert_eq!(max_settings_part_bytes, 50_000_000);
    assert_eq!(max_text_table_bytes, 400_000_000);
    assert_eq!(max_texts, 10_000_000);
    assert_eq!(max_sheet_path_bytes, 100_000_000);
}

/// The parts of a workbook of the 1904 system whose sheet is one cell, A1,
/// the number 7: `_rels/.rels`, the workbook, its relationships, the styles
/// and the sheet.
fn parts_with_styles() -> Vec<(String, String)> {
    parts_of_1904_worksheet(
        r#"<sheetData><row r="1"><c r="A1"><v>7</v></c></row></sheetData>"#,
        &[],
    )
}

/// `xml` with `num_spaces` spaces more after its declaration, `<?xml ...
/// ?>`, which XML reads as it read `xml`.
fn padded(xml: &str, num_spaces: usize) -> String {
    let declaration_end = xml.find("?>").unwrap().checked_add(2).unwrap();
    let (declaration, rest) = xml.split_at(declaration_end);
    format!("{declaration}{}{rest}", " ".repeat(num_spaces))
}

/// The number of bytes of the part `part_name` of `parts`.
fn bytes_of_part(parts: &[(String, String)], part_name: &str) -> u64 {
    let (_, xml) = parts.iter().find(|(name, _)| name == part_name).unwrap();
    u64::try_from(xml.len()).unwrap()
}

#[test]
fn a_settings_part_past_the_bound_is_too_large_and_one_at_it_read() {
    for part_name in [
        "_rels/.rels",
        "xl/workbook.xml",
        "xl/_rels/workbook.xml.rels",
        "xl/styles.xml",
    ] {
        let parts = parts_with_styles();
        let num_bytes = bytes_of_part(&parts, part_name);
        let largest = parts
            .iter()
            .map(|(name, _)| bytes_of_part(&parts, name))
            .max()
            .unwrap();
        let mut padded_parts = parts.clone();
        for (name, xml) in &mut padded_parts {
            if name == part_name {
                *xml = padded(xml, usize::try_from(largest - num_bytes + 1).unwrap());
            }
        }

        let past_bound = read_within(
            &padded_parts,
            bounds_with(|bounds| bounds.max_settings_part_bytes = largest),
        );
        let at_bound = read_within(
            &padded_parts,
            bounds_with(|bounds| bounds.max_settings_part_bytes = largest + 1),
        );

        assert_eq!(
            past_bound,
            Err(ReadError::Unreadable(
                "a part of the file is too large".to_owned()
            )),
            "{part_name}"
        );
        assert_eq!(
            at_bound.unwrap().cells,
            [SheetCell::Number(7.0)],
            "{part_name}"
        );
    }
}

// Every part whose name ends as a settings part is held to the bound, `\`
// read as `/` and ignoring case, in any folder: the part calamine reads is
// one of them.
#[test]
fn a_settings_part_in_another_folder_is_held_to_the_bound() {
    for (part_name, copy_name) in [
        ("xl/workbook.xml", "data/workbook.xml"),
        ("xl/_rels/workbook.xml.rels", "data/_rels/workbook.xml.rels"),
        ("xl/styles.xml", "data/styles.xml"),
        ("xl/styles.xml", "DATA\\STYLES.XML"),
        ("xl/workbook.xml", "otherworkbook.xml"),
        ("_rels/.rels", "_RELS\\.RELS"),
    ] {
        let mut parts = parts_with_styles();
        let largest = parts
            .iter()
            .map(|(name, _)| bytes_of_part(&parts, name))
            .max()
            .unwrap();
        let (_, xml) = parts.iter().find(|(name, _)| name == part_name).unwrap();
        let copy = padded(xml, usize::try_from(largest).unwrap());
        parts.push((copy_name.to_owned(), copy));
        let num_copy_bytes = bytes_of_part(&parts, copy_name);

        let past_bound = read_within(
            &parts,
            bounds_with(|bounds| bounds.max_settings_part_bytes = num_copy_bytes - 1),
        );
        let at_bound = read_within(
            &parts,
            bounds_with(|bounds| bounds.max_settings_part_bytes = num_copy_bytes),
        );

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

/// What `read_first_sheet_within` gives for the sheet of the texts `id` and
/// `pop` whose table of texts is `shared_strings_xml`, with at most 2
/// texts.
fn read_within_two_texts(shared_strings_xml: String) -> Result<Sheet, ReadError> {
    read_within(
        &parts_with_texts(shared_strings_xml),
        bounds_with(|bounds| bounds.max_texts = 2),
    )
}

#[test]
fn a_table_of_texts_of_as_many_texts_as_the_bound_is_read() {
    let read = read_within_two_texts(shared_strings(r#"uniqueCount="2""#, &["id", "pop"]));

    assert_eq!(read.unwrap().cells, the_two_texts());
}

// calamine counts the si of the first sst, not those inside another si nor
// outside the root; xlsx_rs counts every one, and every uniqueCount.
#[test]
fn a_table_of_texts_of_more_texts_than_the_bound_is_too_many_texts() {
    let two_texts = shared_strings(r#"uniqueCount="2""#, &["id", "pop"]);
    for shared_strings_xml in [
        shared_strings("", &["id", "pop", "height"]),
        two_texts.replace("<t>pop</t>", "<t>pop</t><si/>"),
        two_texts.replace("</sst>", "</sst><si/>"),
        two_texts.replace(r#"uniqueCount="2""#, r#"uniqueCount="3""#),
        two_texts.replace(r#"uniqueCount="2""#, r#"uniqueCount="0003""#),
    ] {
        let read = read_within_two_texts(shared_strings_xml.clone());

        assert_eq!(
            read,
            Err(ReadError::Unreadable("too many texts".to_owned())),
            "{shared_strings_xml}"
        );
    }
}

/// The number of bytes of the longest tag of `xml`, from `<` to `>`.
fn longest_tag_of(xml: &str) -> u64 {
    xml.split('<')
        .skip(1)
        .map(|tag| u64::try_from(tag.find('>').unwrap().checked_add(2).unwrap()).unwrap())
        .max()
        .unwrap()
}

// The workbook of one sheet counts 2, its element sheets and its sheet,
// times the longest tags of _rels/.rels and of its relationships.
#[test]
fn the_paths_of_the_sheets_past_the_bound_are_too_many_sheets() {
    let parts =
        parts_of_worksheet(r#"<sheetData><row r="1"><c r="A1"><v>7</v></c></row></sheetData>"#);
    let longest_tag = |part_name: &str| {
        let (_, xml) = parts.iter().find(|(name, _)| name == part_name).unwrap();
        longest_tag_of(xml)
    };
    let sheet_path_bytes =
        2 * (longest_tag("_rels/.rels") + longest_tag("xl/_rels/workbook.xml.rels"));

    let at_bound = read_within(
        &parts,
        bounds_with(|bounds| bounds.max_sheet_path_bytes = sheet_path_bytes),
    );
    let past_bound = read_within(
        &parts,
        bounds_with(|bounds| bounds.max_sheet_path_bytes = sheet_path_bytes - 1),
    );

    assert_eq!(at_bound.unwrap().cells, [SheetCell::Number(7.0)]);
    assert_eq!(
        past_bound,
        Err(ReadError::Unreadable(
            "the workbook lists too many sheets".to_owned()
        ))
    );
}
