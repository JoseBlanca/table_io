//! The rules of a value, "What a text holds" and "The text of a value" of
//! `docs/specs/values.md`: whether a text is missing, the whole number, the
//! number or the boolean it holds, and the text of a float as JavaScript
//! writes it.

/// The mark between the whole part and the decimals of a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecimalMark {
    /// `.`, as in `1.75`.
    Point,
    /// `,`, as in `1,75`, which a Spanish Excel writes.
    Comma,
}

impl DecimalMark {
    /// The character of the mark.
    fn character(self) -> char {
        match self {
            Self::Point => '.',
            Self::Comma => ',',
        }
    }
}

/// Whether a text is a missing value: `""`, `"NA"` or `"-"`, exactly.
///
/// The text is taken as it is: `" NA"`, `"na"`, `"N/A"` and `"NaN"` are
/// not missing. The import removes the spaces at the ends of a cell before
/// it asks.
pub fn is_missing(text: &str) -> bool {
    matches!(text, "" | "NA" | "-")
}

/// The whole number a text holds, or None.
///
/// A whole number is an optional sign, `+` or `-`, then one ASCII digit or
/// more and nothing else, from −2^63 to 2^63 − 1: `"+5"` is 5, `"007"` is
/// 7, `"-0"` is 0; `"12.0"`, `"1e3"`, `"1 000"` and
/// `"9223372036854775808"` are None.
pub fn parse_integer(text: &str) -> Option<i64> {
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    if is_digits(digits) {
        text.parse::<i64>().ok()
    } else {
        None
    }
}

/// The finite number a text holds with the decimal mark, or None.
///
/// A number is an optional sign, then ASCII digits with at most one
/// decimal mark among or around them, at least one digit in all, then
/// optionally an exponent, `e` or `E`, an optional sign and one digit or
/// more: with the comma, `"-1,75"`, `",5"`, `"5,"` and `"1,2E-03"`. No
/// thousands separator, space, `%`, `Inf` or `NaN`, and the other mark is
/// not taken: with the comma `"1.5"` is None. The value is the float
/// nearest to the decimal number written, and a number too large for a
/// float, `"1e999"`, is None; one too small, `"1e-999"`, is 0.
pub fn parse_float(text: &str, decimal: DecimalMark) -> Option<f64> {
    let mark = decimal.character();
    if !is_number_text(text, mark) {
        return None;
    }
    let number = match decimal {
        DecimalMark::Point => text.parse::<f64>().ok()?,
        DecimalMark::Comma => text.replace(',', ".").parse::<f64>().ok()?,
    };
    number.is_finite().then_some(number)
}

/// The boolean a text holds, TRUE or FALSE in any case, or None.
///
/// `"True"` and `"false"` are booleans; `"yes"`, `"1"`, `"T"`, and the
/// `VERDADERO` and `FALSO` of a Spanish Excel, are not.
pub fn parse_boolean(text: &str) -> Option<bool> {
    if text.eq_ignore_ascii_case("true") {
        Some(true)
    } else if text.eq_ignore_ascii_case("false") {
        Some(false)
    } else {
        None
    }
}

/// A float as JavaScript writes it, with the decimal mark.
///
/// The text is that of JavaScript's `String(number)`: the shortest digits
/// that read back as the same float, the even one of two that are equally
/// near, laid out plainly from 10^−6 to below 10^21, `0.000001`,
/// `100000000000000000000`, and with an exponent outside, `1e-7`,
/// `1.5e+21`; `-0` is `"0"`, and a float that is not finite, which no
/// import gives, is `"Infinity"`, `"-Infinity"` or `"NaN"`. The point of
/// JavaScript's text is then made the decimal mark: `1.5` with the comma
/// is `"1,5"`.
pub fn float_text(number: f64, decimal: DecimalMark) -> String {
    if number.is_nan() {
        return "NaN".to_owned();
    }
    if number.is_infinite() {
        return if number.is_sign_positive() {
            "Infinity".to_owned()
        } else {
            "-Infinity".to_owned()
        };
    }
    if number == 0.0 {
        return "0".to_owned();
    }
    let absolute_text = text_of_positive(number.abs(), decimal.character());
    if number.is_sign_negative() {
        format!("-{absolute_text}")
    } else {
        absolute_text
    }
}

/// Whether a text is one ASCII digit or more and nothing else.
fn is_digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

/// Whether a text is a number by the rule of [`parse_float`], with `mark`
/// its decimal mark.
fn is_number_text(text: &str, mark: char) -> bool {
    let unsigned = text.strip_prefix(['+', '-']).unwrap_or(text);
    let (mantissa, exponent) = match unsigned.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, Some(exponent)),
        None => (unsigned, None),
    };
    let is_mantissa = match mantissa.split_once(mark) {
        Some((whole_part, decimals)) => {
            (whole_part.is_empty() || is_digits(whole_part))
                && (decimals.is_empty() || is_digits(decimals))
                && !(whole_part.is_empty() && decimals.is_empty())
        }
        None => is_digits(mantissa),
    };
    let is_exponent = match exponent {
        Some(exponent) => is_digits(exponent.strip_prefix(['+', '-']).unwrap_or(exponent)),
        None => true,
    };
    is_mantissa && is_exponent
}

/// The shortest digits of a float and its exponent: `1.25e-3` is `125`
/// and −3, the number being d.ddd × 10^exponent.
struct ShortestDigits {
    /// The digits, ASCII, the first not 0, the last not 0, 1 to 17 of
    /// them.
    digits: String,
    /// The power of 10 of the first digit, −324 to 308.
    exponent: i32,
}

/// The text of a finite float greater than 0, as JavaScript writes it,
/// with `mark` for its point.
fn text_of_positive(number: f64, mark: char) -> String {
    let rust_text = format!("{number:e}");
    let Some(rust_digits) = digits_of_scientific(&rust_text) else {
        // Rust's `{:e}` always writes a mantissa, `e` and an exponent of a
        // few digits; were it not so, its own text is given.
        return rust_text;
    };
    let shortest = even_of_tie(number, rust_digits);
    javascript_layout(&shortest, mark)
}

/// The digits and the exponent of a text written by Rust's `{:e}`,
/// `1.25e-3`, or None for any other text.
fn digits_of_scientific(scientific: &str) -> Option<ShortestDigits> {
    let (mantissa, exponent) = scientific.split_once('e')?;
    let exponent = exponent.parse::<i32>().ok()?;
    let digits: String = mantissa
        .chars()
        .filter(|&character| character != '.')
        .collect();
    is_digits(&digits).then_some(ShortestDigits { digits, exponent })
}

/// The digits JavaScript takes where Rust's `{:e}` gave `rust_digits`.
///
/// They differ only when the float lies exactly halfway between two texts
/// of the shortest digits: Rust's `{:e}` takes the one above and
/// JavaScript the one whose last digit is even (`docs/specs/values.md`,
/// "The text of a value"). So when the last digit of `rust_digits` is odd
/// and the exact value of the float is the halfway point between them and
/// the same digits less one in the last place, the digits less one are
/// taken, if they read back as the same float.
fn even_of_tie(number: f64, rust_digits: ShortestDigits) -> ShortestDigits {
    let Some(below) = digits_less_one_if_odd(&rust_digits.digits) else {
        return rust_digits;
    };
    let candidate = ShortestDigits {
        digits: below.trim_end_matches('0').to_owned(),
        exponent: rust_digits.exponent,
    };
    // The checks go from the quickest to the slowest, each needed for a
    // tie: the digits less one read back as the float too; the digits of
    // the float to one place more, rounded, are the halfway point; and its
    // exact value is.
    if !is_digits(&candidate.digits) || !reads_back_as(&candidate, number) {
        return rust_digits;
    }
    let halfway = format!("{below}5");
    let precision = rust_digits.digits.len();
    let is_rounded_halfway =
        digits_of_scientific(&format!("{number:.precision$e}")).is_some_and(|rounded| {
            rounded.exponent == rust_digits.exponent && rounded.digits == halfway
        });
    if !is_rounded_halfway {
        return rust_digits;
    }
    // A float holds at most 767 significant digits, so that `{:.767e}`
    // writes its exact value, here with the zeros at its end removed.
    let is_halfway = digits_of_scientific(&format!("{number:.767e}")).is_some_and(|exact| {
        exact.exponent == rust_digits.exponent && exact.digits.trim_end_matches('0') == halfway
    });
    if is_halfway { candidate } else { rust_digits }
}

/// The digits less one in the last place when the last is odd, `126` for
/// `127`, or None when the last digit is even.
fn digits_less_one_if_odd(digits: &str) -> Option<String> {
    let (head, last) = digits.split_at_checked(digits.len().checked_sub(1)?)?;
    let last_below = match last {
        "1" => '0',
        "3" => '2',
        "5" => '4',
        "7" => '6',
        "9" => '8',
        _ => return None,
    };
    Some(format!("{head}{last_below}"))
}

/// Whether `shortest` reads back as `number`.
fn reads_back_as(shortest: &ShortestDigits, number: f64) -> bool {
    let mut characters = shortest.digits.chars();
    let first = characters.next().unwrap_or('0');
    let rest: String = characters.collect();
    format!("{first}.{rest}0e{}", shortest.exponent)
        .parse::<f64>()
        .is_ok_and(|read_back| read_back.to_bits() == number.to_bits())
}

/// The layout of ECMAScript's `Number::toString`, from the shortest
/// digits, k of them, and n, their exponent plus 1, so that the number is
/// 0.d × 10^n.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "k is 1 to 17 and n −323 to 309, the digits and the exponent of a finite float"
)]
fn javascript_layout(shortest: &ShortestDigits, mark: char) -> String {
    let digits = &shortest.digits;
    let num_digits = digits.chars().fold(0_i32, |count, _| count + 1);
    let point_place = shortest.exponent + 1;
    let mut text = String::new();
    if num_digits <= point_place && point_place <= 21 {
        text.push_str(digits);
        push_zeros(&mut text, point_place - num_digits);
    } else if 0 < point_place && point_place <= 21 {
        for (place, digit) in (0_i32..).zip(digits.chars()) {
            if place == point_place {
                text.push(mark);
            }
            text.push(digit);
        }
    } else if -6 < point_place && point_place <= 0 {
        text.push('0');
        text.push(mark);
        push_zeros(&mut text, -point_place);
        text.push_str(digits);
    } else {
        let mut characters = digits.chars();
        if let Some(first) = characters.next() {
            text.push(first);
        }
        let rest = characters.as_str();
        if !rest.is_empty() {
            text.push(mark);
            text.push_str(rest);
        }
        text.push('e');
        text.push(if point_place > 0 { '+' } else { '-' });
        text.push_str(&(point_place - 1).unsigned_abs().to_string());
    }
    text
}

/// Pushes `count` zeros, none when `count` is 0 or less.
fn push_zeros(text: &mut String, count: i32) {
    for _ in 0..count {
        text.push('0');
    }
}
