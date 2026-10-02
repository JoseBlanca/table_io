//! The messages of the bounds of "What table_io reads before calamine" of
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
    parts_of_1904_worksheet, parts_of_worksheet, shared_strings, stored_zip, xlsx_of_parts,
};
use crate::parts::PartBounds;
use crate::{
    MAX_PART_BYTES, MAX_SETTINGS_PART_BYTES, MAX_SHEET_PATH_BYTES, MAX_TEXT_BYTES, MAX_TEXTS,
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

// A table of texts is held to the bound of every part, with a message of
// its own.
#[test]
fn a_table_of_texts_past_the_bound_of_its_bytes_is_too_much_text() {
    let parts = parts_with_texts(shared_strings(r#"uniqueCount="2""#, &["id", "pop"]));
    let (padded_parts, largest) = with_largest_part(&parts, "xl/sharedStrings.xml");

    let past_bound = read_within(
        &padded_parts,
        bounds_with(|bounds| bounds.max_part_bytes = largest),
    );
    let at_bound = read_within(
        &padded_parts,
        bounds_with(|bounds| bounds.max_part_bytes = largest + 1),
    );

    assert_eq!(
        past_bound,
        Err(ReadError::Unreadable("too much text".to_owned()))
    );
    assert_eq!(at_bound.unwrap().cells, the_two_texts());
}

#[test]
fn a_table_of_texts_in_another_folder_is_held_to_the_bound_of_its_bytes() {
    let mut parts = parts_with_texts(shared_strings(r#"uniqueCount="2""#, &["id", "pop"]));
    parts.push((
        "data/sharedStrings.xml".to_owned(),
        shared_strings(r#"uniqueCount="2""#, &["id", "pop"]),
    ));
    let (padded_parts, largest) = with_largest_part(&parts, "data/sharedStrings.xml");

    let read = read_within(
        &padded_parts,
        bounds_with(|bounds| bounds.max_part_bytes = largest),
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
    assert_eq!(MAX_TEXTS, 10_000_000);
    assert_eq!(MAX_SHEET_PATH_BYTES, 100_000_000);
    assert_eq!(MAX_PART_BYTES, 300_000_000);

    let PartBounds {
        max_unzipped_bytes,
        max_settings_part_bytes,
        max_texts,
        max_sheet_path_bytes,
        max_part_bytes,
    } = PART_BOUNDS;
    assert_eq!(max_unzipped_bytes, 1_000_000_000);
    assert_eq!(max_settings_part_bytes, 50_000_000);
    assert_eq!(max_texts, 10_000_000);
    assert_eq!(max_sheet_path_bytes, 100_000_000);
    assert_eq!(max_part_bytes, 300_000_000);
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
// outside the root; table_io counts every one, and every uniqueCount.
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

/// The parts of [`parts_with_styles`] whose styles have an error of XML
/// right after the declaration, an end tag with no element open, followed
/// by `padding`.
fn parts_with_styles_failing_early(padding: &str) -> Vec<(String, String)> {
    let mut parts = parts_with_styles();
    for (name, xml) in &mut parts {
        if name == "xl/styles.xml" {
            let declaration_end = xml.find("?>").unwrap().checked_add(2).unwrap();
            let (declaration, rest) = xml.split_at(declaration_end);
            *xml = format!("{declaration}</x>{padding}{rest}");
        }
    }
    parts
}

// The reader of XML stops at the error, within its first read of 8 KB, and
// the part is still read to its end: its 50,000 bytes more pass the bound.
#[test]
fn a_settings_part_is_counted_to_its_end_after_an_error_of_xml() {
    let parts = parts_with_styles_failing_early(&" ".repeat(50_000));

    let read = read_within(
        &parts,
        bounds_with(|bounds| bounds.max_settings_part_bytes = 20_000),
    );

    assert_eq!(
        read,
        Err(ReadError::Unreadable(
            "a part of the file is too large".to_owned()
        ))
    );
}

// A byte changed after the error of XML, where the part is stored
// uncompressed, is found by the checksum of the part, which the zip crate
// checks at its end.
#[test]
fn a_settings_part_is_checksummed_after_an_error_of_xml() {
    let mark = "a byte after the error";
    let parts = parts_with_styles_failing_early(&format!("{}{mark}", " ".repeat(50_000)));
    let files: Vec<(&str, &str)> = parts
        .iter()
        .map(|(name, xml)| (name.as_str(), xml.as_str()))
        .collect();
    let mut bytes = stored_zip(&files);
    let mark_position = bytes
        .windows(mark.len())
        .position(|window| window == mark.as_bytes())
        .unwrap();
    bytes[mark_position] = b'A';

    let read = read_first_sheet_within(&bytes, MAX_SHEET_CELLS, PART_BOUNDS);

    assert_eq!(
        read,
        Err(ReadError::Unreadable("Invalid checksum".to_owned()))
    );
}

// The package relationships are the part named _rels/.rels, from the root:
// a/_rels/.rels is another part, which calamine does not hold whole.
#[test]
fn a_part_of_relationships_in_a_folder_is_not_held_to_the_bound_of_settings_parts() {
    let mut parts = parts_with_styles();
    let (_, package_relationships) = parts
        .iter()
        .find(|(name, _)| name == "_rels/.rels")
        .unwrap();
    let copy = padded(package_relationships, 50_000);
    parts.push(("a/_rels/.rels".to_owned(), copy));

    let read = read_within(
        &parts,
        bounds_with(|bounds| bounds.max_settings_part_bytes = 20_000),
    );

    assert_eq!(read.unwrap().cells, [SheetCell::Number(7.0)]);
}

/// The number of bytes of the largest part of `parts`.
fn largest_part_of(parts: &[(String, String)]) -> u64 {
    parts
        .iter()
        .map(|(name, _)| bytes_of_part(parts, name))
        .max()
        .unwrap()
}

/// `parts` with the part `part_name` made one byte larger than the largest
/// of them, by spaces after its declaration, or spaces alone for a part
/// that has none; and that largest size, the bound it passes.
fn with_largest_part(parts: &[(String, String)], part_name: &str) -> (Vec<(String, String)>, u64) {
    let largest = largest_part_of(parts);
    let mut padded_parts = parts.to_vec();
    for (name, xml) in &mut padded_parts {
        if name == part_name {
            let num_spaces = usize::try_from(
                largest
                    .checked_sub(bytes_of_part(parts, part_name))
                    .unwrap()
                    .checked_add(1)
                    .unwrap(),
            )
            .unwrap();
            *xml = if xml.contains("?>") {
                padded(xml, num_spaces)
            } else {
                format!("{xml}{}", " ".repeat(num_spaces))
            };
        }
    }
    (padded_parts, largest)
}

// A part calamine does not hold whole is bounded too, since one cell of the
// sheet can hold a text as large as the sheet ("What table_io reads before
// calamine", point 5); docProps/padding.xml is a part calamine never opens.
#[test]
fn a_part_other_than_a_table_of_texts_past_the_bound_is_too_large_and_one_at_it_read() {
    let mut parts = parts_with_styles();
    parts.push(("docProps/padding.xml".to_owned(), " ".repeat(100)));
    for part_name in ["xl/worksheets/sheet1.xml", "docProps/padding.xml"] {
        let (padded_parts, largest) = with_largest_part(&parts, part_name);

        let past_bound = read_within(
            &padded_parts,
            bounds_with(|bounds| bounds.max_part_bytes = largest),
        );
        let at_bound = read_within(
            &padded_parts,
            bounds_with(|bounds| bounds.max_part_bytes = largest + 1),
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

#[test]
fn a_settings_part_is_held_to_the_bound_of_every_part_too() {
    let (padded_parts, largest) = with_largest_part(&parts_with_styles(), "xl/styles.xml");

    let read = read_within(
        &padded_parts,
        bounds_with(|bounds| bounds.max_part_bytes = largest),
    );

    assert_eq!(
        read,
        Err(ReadError::Unreadable(
            "a part of the file is too large".to_owned()
        ))
    );
}

/// A second declaration of windows-1252, which quick-xml lets be after the
/// first of UTF-8 and the rule of UTF-8 does not.
const SECOND_DECLARATION: &str = r#"<?xml version="1.0" encoding="windows-1252"?>"#;

/// The same behind a decoy, an `encoding` of UTF-8 in the value of another
/// attribute.
const DECOY_DECLARATION: &str =
    r#"<?xml version="1.0" foo="encoding='utf-8'" encoding="windows-1252"?>"#;

/// The parts of a workbook whose sheet holds A1, 7, and A2, 8, with
/// `declaration` and `num_spaces` spaces between the two rows.
fn parts_with_declaration_between_rows(
    declaration: &str,
    num_spaces: usize,
) -> Vec<(String, String)> {
    parts_of_worksheet(&format!(
        r#"<sheetData><row r="1"><c r="A1"><v>7</v></c></row>{declaration}{}<row r="2"><c r="A2"><v>8</v></c></row></sheetData>"#,
        " ".repeat(num_spaces)
    ))
}

// A part other than those calamine holds whole is counted three times when
// it is found in another encoding, since decoding makes at most 3 bytes of
// UTF-8 of each byte ("What table_io reads before calamine", point 5).
#[test]
fn a_sheet_found_in_another_encoding_is_counted_three_times() {
    let version_of_1_000_000_bytes = format!(
        r#"<?xml version="{}" encoding="windows-1252"?>"#,
        "1".repeat(1_000_000)
    );
    for declaration in [
        SECOND_DECLARATION,
        DECOY_DECLARATION,
        &version_of_1_000_000_bytes,
    ] {
        let parts = parts_with_declaration_between_rows(declaration, 2_000);
        let num_bytes = bytes_of_part(&parts, "xl/worksheets/sheet1.xml");

        let at_bound = read_within(
            &parts,
            bounds_with(|bounds| bounds.max_part_bytes = 3 * num_bytes),
        );
        let past_bound = read_within(
            &parts,
            bounds_with(|bounds| bounds.max_part_bytes = 3 * num_bytes - 1),
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

// An image of an older program declared iso-8859-1 does not refuse the
// workbook it is in; it is counted three times.
#[test]
fn a_part_calamine_never_opens_found_in_another_encoding_is_counted_three_times() {
    let mut parts = parts_with_styles();
    parts.push((
        "xl/media/a.svg".to_owned(),
        format!(
            r#"<?xml version="1.0" encoding="iso-8859-1"?><svg>{}</svg>"#,
            " ".repeat(2_000)
        ),
    ));
    let num_bytes = bytes_of_part(&parts, "xl/media/a.svg");

    let at_bound = read_within(
        &parts,
        bounds_with(|bounds| bounds.max_part_bytes = 3 * num_bytes),
    );
    let past_bound = read_within(
        &parts,
        bounds_with(|bounds| bounds.max_part_bytes = 3 * num_bytes - 1),
    );

    assert_eq!(at_bound.unwrap().cells, [SheetCell::Number(7.0)]);
    assert_eq!(
        past_bound,
        Err(ReadError::Unreadable(
            "a part of the file is too large".to_owned()
        ))
    );
}

// A sheet in UTF-8 is counted once: at the bound it is read.
#[test]
fn a_sheet_in_utf_8_is_counted_once() {
    let parts = parts_with_declaration_between_rows("", 2_000);
    let num_bytes = bytes_of_part(&parts, "xl/worksheets/sheet1.xml");

    let read = read_within(
        &parts,
        bounds_with(|bounds| bounds.max_part_bytes = num_bytes),
    );

    assert_eq!(
        read.unwrap().cells,
        [SheetCell::Number(7.0), SheetCell::Number(8.0)]
    );
}
