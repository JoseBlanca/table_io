//! The parts of the zip xlsx_rs reads itself before calamine, as "What
//! xlsx_rs reads before calamine" of `docs/specs/read.md` gives them.
//!
//! An xlsx is a zip of parts, each an XML file. Each part carries a
//! checksum of its bytes, which the zip crate checks only once the part has
//! been read to its end; calamine stops reading a sheet at its last cell,
//! so xlsx_rs reads every part to its end first, and counts the bytes.

use std::io::{self, Cursor, Read};

use zip::ZipArchive;
use zip::result::ZipError;

use crate::ReadError;

/// Reads every part of the zip `bytes` to its end, so that the zip crate
/// checks its checksum, and discards what it reads.
///
/// # Errors
///
/// [`ReadError::Unreadable`] with the zip crate's message for any error of
/// the zip, "Invalid checksum" for a part whose bytes are not those it was
/// saved with; and "the file unzips to more than … bytes" when the parts
/// hold more than `max_unzipped_bytes` bytes together.
pub(crate) fn read_every_part(bytes: &[u8], max_unzipped_bytes: u64) -> Result<(), ReadError> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(unreadable_of_zip_error)?;
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
