//! The cell of a number with a format of date, of time or of duration
//! ("Each cell" of `docs/specs/read.md`).

use calamine::{ExcelDateTime, ExcelDateTimeType};

use crate::cell::{LARGEST_EXACT_WHOLE_NUMBER, cell_of_number};
use crate::xlsx::SheetCell;

/// The date system of a workbook: the day Excel counts its numbers from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DateSystem {
    /// Excel's usual system, whose day 1 is 1 January 1900, with Excel's
    /// 29 February 1900, day 60, a day that did not exist.
    Excel1900,
    /// The system of old Excel for Mac, whose day 0 is 1 January 1904.
    Excel1904,
}

/// The cell of `date_time`, a number with a format of date, of time or of
/// duration, in the workbook's `date_system`: a text, or the number itself
/// when it has no date, as "Each cell" gives them.
///
/// The number is first rounded to a whole number of milliseconds, and the
/// rest is worked out from that with whole numbers, so that
/// 45425.9999999999, a hundredth of a millisecond before midnight, is the
/// next day at 0:00. The year, month and day come from calamine.
pub(crate) fn cell_of_date(date_time: &ExcelDateTime, date_system: DateSystem) -> SheetCell {
    let number = date_time.as_f64();
    let Some(milliseconds) = whole_milliseconds_of(number) else {
        return cell_of_number(number);
    };
    let date_text = if date_time.is_duration() {
        Some(text_of_duration(milliseconds))
    } else {
        text_of_date(milliseconds, date_system)
    };
    match date_text {
        Some(date_text) => SheetCell::Text(date_text),
        None => cell_of_number(number),
    }
}

/// The milliseconds of a day, 24 × 60 × 60 × 1000.
const MILLISECONDS_PER_DAY: u32 = 86_400_000;

/// The largest number of milliseconds taken, 2^53, up to which a float
/// holds every whole number exactly; it is 104,249,991 days, far past the
/// last day of 9999, 2,958,465.
#[expect(
    clippy::cast_precision_loss,
    reason = "2^53 is a power of 2, which a float holds exactly"
)]
const MAX_WHOLE_MILLISECONDS: f64 = LARGEST_EXACT_WHOLE_NUMBER as f64;

/// The last day calamine is asked for the parts of, 2,958,466, the first
/// day of the year 10000 in the 1900 system, and a day of 10004 in the 1904
/// one: its parts turn the days into a year of 16 bits, so a day far past
/// 9999 would give a year that wrapped around, and one of those.
const LAST_DAY_ASKED: u64 = 2_958_466;

/// `number` of days rounded to a whole number of milliseconds, or `None`
/// when it is not a number or is past [`MAX_WHOLE_MILLISECONDS`] either side
/// of 0.
fn whole_milliseconds_of(number: f64) -> Option<i64> {
    let rounded = (number * f64::from(MILLISECONDS_PER_DAY)).round();
    if !rounded.is_finite() || rounded.abs() > MAX_WHOLE_MILLISECONDS {
        return None;
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a whole float within 2^53 either side of 0 is an i64 exactly, as checked above"
    )]
    let whole_milliseconds = rounded as i64;
    Some(whole_milliseconds)
}

/// The text of a date, of a date and a time, or of a time alone, of
/// `milliseconds` from the start of `date_system`; `None` when they are
/// below 0 or after 31 December 9999.
fn text_of_date(milliseconds: i64, date_system: DateSystem) -> Option<String> {
    let milliseconds = u64::try_from(milliseconds).ok()?;
    let days = milliseconds.checked_div(u64::from(MILLISECONDS_PER_DAY))?;
    let milliseconds_of_day = milliseconds.checked_rem(u64::from(MILLISECONDS_PER_DAY))?;
    if days == 0 {
        return Some(text_of_time(milliseconds_of_day));
    }
    if days > LAST_DAY_ASKED {
        return None;
    }
    let is_1904 = match date_system {
        DateSystem::Excel1900 => false,
        DateSystem::Excel1904 => true,
    };
    let calamine_days = f64::from(u32::try_from(days).ok()?);
    let (year, month, day, _, _, _, _) =
        ExcelDateTime::new(calamine_days, ExcelDateTimeType::DateTime, is_1904).to_ymd_hms_milli();
    if year > LAST_YEAR {
        return None;
    }
    let date_text = format!("{year:04}-{month:02}-{day:02}");
    if milliseconds_of_day == 0 {
        Some(date_text)
    } else {
        Some(format!("{date_text} {}", text_of_time(milliseconds_of_day)))
    }
}

/// The last year Excel shows a date of.
const LAST_YEAR: u16 = 9999;

/// The text of a time of day, `14:30:00`, of `milliseconds_of_day` below a
/// day, with its milliseconds, `14:30:00.250`, when they are not 0.
fn text_of_time(milliseconds_of_day: u64) -> String {
    let clock = ClockParts::of(milliseconds_of_day);
    format!("{:02}{}", clock.hours, clock.text_after_hours())
}

/// The text of a duration of `milliseconds`, its hours not wrapped at 24,
/// `36:00:00`, and with a minus when it is negative, `-0:30:00`.
fn text_of_duration(milliseconds: i64) -> String {
    let sign = if milliseconds < 0 { "-" } else { "" };
    let clock = ClockParts::of(milliseconds.unsigned_abs());
    format!("{sign}{}{}", clock.hours, clock.text_after_hours())
}

/// A number of milliseconds as hours, minutes, seconds and milliseconds.
struct ClockParts {
    /// The whole hours, not wrapped at 24.
    hours: u64,
    /// The minutes past the hours, 0 to 59.
    minutes: u64,
    /// The seconds past the minutes, 0 to 59.
    seconds: u64,
    /// The milliseconds past the seconds, 0 to 999.
    milliseconds: u64,
}

impl ClockParts {
    /// The parts of `milliseconds`.
    fn of(milliseconds: u64) -> Self {
        let seconds = milliseconds.div_euclid(1000);
        let minutes = seconds.div_euclid(60);
        Self {
            hours: minutes.div_euclid(60),
            minutes: minutes.rem_euclid(60),
            seconds: seconds.rem_euclid(60),
            milliseconds: milliseconds.rem_euclid(1000),
        }
    }

    /// The minutes and the seconds, `:30:00`, and the milliseconds,
    /// `:30:00.250`, when they are not 0.
    fn text_after_hours(&self) -> String {
        if self.milliseconds == 0 {
            format!(":{:02}:{:02}", self.minutes, self.seconds)
        } else {
            format!(
                ":{:02}:{:02}.{:03}",
                self.minutes, self.seconds, self.milliseconds
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use calamine::{ExcelDateTime, ExcelDateTimeType};

    use crate::date::{DateSystem, cell_of_date};
    use crate::xlsx::SheetCell;

    /// The cell of `number` with a format of date, in the 1904 system.
    fn cell_of_1904_date(number: f64) -> SheetCell {
        let date_time = ExcelDateTime::new(number, ExcelDateTimeType::DateTime, true);
        cell_of_date(&date_time, DateSystem::Excel1904)
    }

    // rust_xlsxwriter does not write the date system of 1904, so its cases
    // are here.

    #[test]
    fn the_last_day_of_9999_in_the_1904_system_is_the_date() {
        assert_eq!(
            cell_of_1904_date(2_957_003.0),
            SheetCell::Text("9999-12-31".to_owned())
        );
    }

    #[test]
    fn the_first_day_of_10000_in_the_1904_system_is_the_number() {
        assert_eq!(
            cell_of_1904_date(2_957_004.0),
            SheetCell::Number(2_957_004.0)
        );
    }

    #[test]
    fn a_day_far_past_9999_is_the_number() {
        // calamine's year of 16 bits would give the parts of a day of 4811.
        let date_time = ExcelDateTime::new(25_000_000.0, ExcelDateTimeType::DateTime, false);
        assert_eq!(
            cell_of_date(&date_time, DateSystem::Excel1900),
            SheetCell::Number(25_000_000.0)
        );
    }

    #[test]
    fn a_day_far_past_9999_in_the_1904_system_is_the_number() {
        assert_eq!(
            cell_of_1904_date(25_000_000.0),
            SheetCell::Number(25_000_000.0)
        );
    }

    #[test]
    fn a_date_below_0_in_the_1904_system_is_the_number() {
        assert_eq!(cell_of_1904_date(-3.0), SheetCell::Number(-3.0));
    }

    #[test]
    fn half_a_day_in_the_1904_system_is_the_time_alone() {
        assert_eq!(
            cell_of_1904_date(0.5),
            SheetCell::Text("12:00:00".to_owned())
        );
    }

    #[test]
    fn a_date_and_time_of_the_1904_system_is_the_same_text_as_in_the_1900_one() {
        // 13 May 2024 at 12:00 is 45425.5 in the 1900 system, and 1462 days
        // fewer in the 1904 one.
        assert_eq!(
            cell_of_1904_date(43_963.5),
            SheetCell::Text("2024-05-13 12:00:00".to_owned())
        );
    }

    #[test]
    fn a_date_that_is_not_a_number_is_its_text() {
        let date_time = ExcelDateTime::new(f64::NAN, ExcelDateTimeType::DateTime, false);
        assert_eq!(
            cell_of_date(&date_time, DateSystem::Excel1900),
            SheetCell::Text("NaN".to_owned())
        );
    }

    #[test]
    fn a_date_past_what_a_whole_number_of_milliseconds_holds_is_the_number() {
        let date_time = ExcelDateTime::new(1e300, ExcelDateTimeType::DateTime, false);
        assert_eq!(
            cell_of_date(&date_time, DateSystem::Excel1900),
            SheetCell::Number(1e300)
        );
    }

    #[test]
    fn a_duration_past_what_a_whole_number_of_milliseconds_holds_is_the_number() {
        let date_time = ExcelDateTime::new(-1e300, ExcelDateTimeType::TimeDelta, false);
        assert_eq!(
            cell_of_date(&date_time, DateSystem::Excel1900),
            SheetCell::Number(-1e300)
        );
    }

    #[test]
    fn a_duration_with_milliseconds_gives_them() {
        // 1 hour, 2 minutes, 3.004 seconds, as a fraction of a day.
        let date_time = ExcelDateTime::new(
            3_723_004.0 / 86_400_000.0,
            ExcelDateTimeType::TimeDelta,
            false,
        );
        assert_eq!(
            cell_of_date(&date_time, DateSystem::Excel1900),
            SheetCell::Text("1:02:03.004".to_owned())
        );
    }
}
