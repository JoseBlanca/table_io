// SPDX-License-Identifier: MIT
//
// Copyright 2016-2025, Johann Tuffe.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to permit
// persons to whom the Software is furnished to do so, subject to the
// following conditions:
//
// The above copyright notice and this permission notice shall be included
// in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN
// NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
// DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE
// USE OR OTHER DEALINGS IN THE SOFTWARE.

//! The attributes of an element read as calamine 0.36.1 reads them, with a
//! copy of its `RawAttrIter` and the rule of its macro `get_attrs!`
//! (`src/attrs.rs` of calamine), so that table_io and
//! calamine find the same attributes in a tag ("What xlsx_rs reads before
//! calamine" of `docs/specs/read.md`). quick-xml's own reader of attributes
//! splits a tag another way: it keeps a form feed, the byte `0C`, in the
//! name of an attribute, which calamine takes for a space before the name.
//!
//! Copied on 28 September 2026 under calamine's MIT license, above, and
//! written again with no indexing and no plain arithmetic, as the lints of
//! table_io ask; what it gives is calamine's, byte for byte.

use quick_xml::events::BytesStart;
use quick_xml::events::attributes::AttrError;

/// The attributes of a tag, each its name and its value as the XML writes
/// them, before any entity is read, or an [`AttrError`] at the first that
/// cannot be read, after which it gives nothing more. calamine's
/// `RawAttrIter`.
///
/// A name is what comes before `=`, after any bytes of ASCII white space,
/// the form feed among them, and without those at its end; a value is what
/// is between the quotes after `=`, or up to the end of the tag when the
/// closing quote is missing.
pub(crate) struct RawAttrIter<'tag> {
    /// The bytes of the tag after the name of its element.
    raw: &'tag [u8],
    /// The position in `raw` where the next attribute is looked for.
    position: usize,
}

impl<'tag> RawAttrIter<'tag> {
    /// The attributes of `element`.
    pub(crate) fn of_element(element: &'tag BytesStart<'_>) -> Self {
        Self {
            raw: element.attributes_raw(),
            position: 0,
        }
    }

    /// The position of the first byte at or after `position` that is not
    /// ASCII white space, or the end of the tag.
    fn after_white_space(&self, position: usize) -> usize {
        let num_spaces = self.raw.get(position..).map_or(0, |rest| {
            rest.iter()
                .take_while(|byte| byte.is_ascii_whitespace())
                .count()
        });
        // Both are within `raw`, so their sum is at most its length.
        position.saturating_add(num_spaces)
    }
}

impl<'tag> Iterator for RawAttrIter<'tag> {
    type Item = Result<(&'tag [u8], &'tag [u8]), AttrError>;

    fn next(&mut self) -> Option<Self::Item> {
        let raw = self.raw;
        let key_start = self.after_white_space(self.position);
        let rest = raw.get(key_start..).unwrap_or_default();
        if rest.is_empty() {
            self.position = key_start;
            return None;
        }
        let Some(key_length) = rest.iter().position(|byte| *byte == b'=') else {
            // A name with no `=`: the next call gives nothing, rather than
            // the same error again.
            self.position = raw.len();
            return Some(Err(AttrError::ExpectedEq(key_start)));
        };
        let key = rest.get(..key_length).unwrap_or_default().trim_ascii_end();
        // The `=` is within `raw`, so the position after it is at most its
        // length.
        let after_equals = key_start.saturating_add(key_length).saturating_add(1);
        let quote_position = self.after_white_space(after_equals);
        let quote = match raw.get(quote_position) {
            Some(quote @ (b'"' | b'\'')) => *quote,
            Some(_) | None => {
                self.position = raw.len();
                return Some(Err(AttrError::UnquotedValue(quote_position)));
            }
        };
        let value_start = quote_position.saturating_add(1);
        let value_and_rest = raw.get(value_start..).unwrap_or_default();
        let (value, next_position) = match value_and_rest.iter().position(|byte| *byte == quote) {
            Some(value_length) => (
                value_and_rest.get(..value_length).unwrap_or_default(),
                value_start.saturating_add(value_length).saturating_add(1),
            ),
            None => (value_and_rest, raw.len()),
        };
        self.position = next_position;
        Some(Ok((key, value)))
    }
}

/// The values of the attributes of `element` named `keys`, in their order,
/// read as calamine's macro `get_attrs!` reads them: each attribute whose
/// name is one of `keys` is counted as found and takes the place of any
/// value found before for it, and the reading stops once as many have been
/// found as there are `keys`, a name written twice counted twice. So with
/// `keys` `Type` and `Target`, `Type="a" Type="b" Target="c"` gives `b` and
/// no target.
///
/// # Errors
///
/// The [`AttrError`] of the first attribute that cannot be read before the
/// reading stops.
pub(crate) fn attributes_of<'tag, const NUM_KEYS: usize>(
    element: &'tag BytesStart<'_>,
    keys: [&[u8]; NUM_KEYS],
) -> Result<[Option<&'tag [u8]>; NUM_KEYS], AttrError> {
    let mut values = [None; NUM_KEYS];
    let mut num_found: usize = 0;
    for attribute in RawAttrIter::of_element(element) {
        let (key, value) = attribute?;
        if let Some(key_index) = keys.iter().position(|wanted_key| *wanted_key == key) {
            if let Some(found_value) = values.get_mut(key_index) {
                *found_value = Some(value);
            }
            num_found = num_found.saturating_add(1);
        }
        if num_found == NUM_KEYS {
            break;
        }
    }
    Ok(values)
}

/// The value of the first attribute of `element` named `key`, as calamine's
/// `raw_attr` reads it.
///
/// # Errors
///
/// The [`AttrError`] of the first attribute that cannot be read before it.
pub(crate) fn attribute_of<'tag>(
    element: &'tag BytesStart<'_>,
    key: &[u8],
) -> Result<Option<&'tag [u8]>, AttrError> {
    let [value] = attributes_of(element, [key])?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use quick_xml::events::BytesStart;
    use quick_xml::events::attributes::AttrError;

    use crate::attrs::{RawAttrIter, attribute_of, attributes_of};

    /// An attribute read, its name and its value, or the error of one.
    type AttributeRead = Result<(Vec<u8>, Vec<u8>), AttrError>;

    /// The attributes of the tag `<element{raw}>`.
    fn attributes_of_raw(raw: &str) -> Vec<AttributeRead> {
        let element = BytesStart::from_content(format!("element{raw}"), "element".len());
        RawAttrIter::of_element(&element)
            .map(|attribute| attribute.map(|(key, value)| (key.to_vec(), value.to_vec())))
            .collect()
    }

    /// `key` and `value` as an attribute read.
    fn read(key: &str, value: &str) -> AttributeRead {
        Ok((key.as_bytes().to_vec(), value.as_bytes().to_vec()))
    }

    #[test]
    fn attributes_are_split_at_white_space_a_form_feed_among_it() {
        assert_eq!(
            attributes_of_raw(" key1=\"val1\" key2='val2'"),
            [read("key1", "val1"), read("key2", "val2")]
        );
        assert_eq!(
            attributes_of_raw(" \x0CuniqueCount=\"4\"\x0C\ts = '1:4' "),
            [read("uniqueCount", "4"), read("s", "1:4")]
        );
        assert_eq!(attributes_of_raw(" key\x0C=\"\""), [read("key", "")]);
        assert_eq!(attributes_of_raw("   "), []);
    }

    #[test]
    fn a_value_may_hold_the_other_quote_and_one_not_closed_runs_to_the_end() {
        assert_eq!(
            attributes_of_raw(" k='a\"b' j=\"c"),
            [read("k", "a\"b"), read("j", "c")]
        );
    }

    #[test]
    fn a_name_with_no_equals_or_a_value_with_no_quote_is_an_error_and_the_end() {
        assert_eq!(
            attributes_of_raw(" good=\"1\" bad=2 next=\"3\""),
            [read("good", "1"), Err(AttrError::UnquotedValue(14))]
        );
        assert_eq!(
            attributes_of_raw("  spans  "),
            [Err(AttrError::ExpectedEq(2))]
        );
        assert_eq!(
            attributes_of_raw(" key="),
            [Err(AttrError::UnquotedValue(5))]
        );
    }

    #[test]
    fn a_name_written_twice_is_found_twice_and_takes_the_place_of_the_first() {
        let element = BytesStart::from_content(
            r#"Relationship Type="a" Type="b" Target="c""#,
            "Relationship".len(),
        );

        let [type_value, target] = attributes_of(&element, [b"Type", b"Target"]).unwrap();

        assert_eq!(type_value, Some(&b"b"[..]));
        assert_eq!(target, None);
    }

    #[test]
    fn the_reading_stops_once_every_name_is_found() {
        let element = BytesStart::from_content(
            r#"Relationship Target="c" Type="a" bad=1"#,
            "Relationship".len(),
        );

        let found = attributes_of(&element, [b"Type", b"Target"]).unwrap();
        let bad = attributes_of(&element, [b"Type", b"Id"]);

        assert_eq!(found, [Some(&b"a"[..]), Some(&b"c"[..])]);
        assert_eq!(bad, Err(AttrError::UnquotedValue(25)));
    }

    #[test]
    fn the_first_attribute_of_a_name_is_its_value() {
        let element =
            BytesStart::from_content(r#"sst uniqueCount="4" uniqueCount="5""#, "sst".len());

        assert_eq!(attribute_of(&element, b"uniqueCount"), Ok(Some(&b"4"[..])));
        assert_eq!(attribute_of(&element, b"count"), Ok(None));
    }
}
