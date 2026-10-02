//! The rules of a value and the conversion of a column, each case of "How
//! it is verified" of `docs/specs/values.md` a literal input and its
//! literal output.

use table_io::{DecimalMark, float_text, is_missing, parse_boolean, parse_float, parse_integer};

/// The bits of a float, so that two floats are compared exactly, `-0` and
/// `0` told apart.
fn bits_of(number: Option<f64>) -> Option<u64> {
    number.map(f64::to_bits)
}

#[test]
fn the_empty_text_na_and_a_dash_are_missing() {
    for text in ["", "NA", "-"] {
        assert!(is_missing(text), "{text:?} should be missing");
    }
}

#[test]
fn na_in_lower_case_n_slash_a_nan_a_spaced_na_and_two_dashes_are_not_missing() {
    for text in ["na", "N/A", "NaN", " NA", "--"] {
        assert!(!is_missing(text), "{text:?} should not be missing");
    }
}

#[test]
fn a_whole_number_with_a_sign_or_leading_zeros_is_an_integer() {
    let cases = [
        ("12", 12),
        ("-3", -3),
        ("+5", 5),
        ("007", 7),
        ("-0", 0),
        ("9223372036854775807", 9_223_372_036_854_775_807),
        ("-9223372036854775808", -9_223_372_036_854_775_808),
    ];
    for (text, whole_number) in cases {
        assert_eq!(parse_integer(text), Some(whole_number), "{text:?}");
    }
}

#[test]
fn a_decimal_an_exponent_a_space_past_the_range_no_digit_or_arabic_digits_are_not_an_integer() {
    for text in [
        "12.0",
        "1e3",
        "1 000",
        "9223372036854775808",
        "",
        "+",
        "\u{661}\u{662}",
    ] {
        assert_eq!(parse_integer(text), None, "{text:?}");
    }
}

#[test]
fn a_number_with_the_comma_is_read_with_the_comma_as_its_decimal_mark() {
    let cases = [
        ("12", 12.0),
        ("-1,75", -1.75),
        (",5", 0.5),
        ("5,", 5.0),
        ("1,2E-03", 0.0012),
        ("+,5e+2", 50.0),
        ("1e-999", 0.0),
    ];
    for (text, number) in cases {
        assert_eq!(
            bits_of(parse_float(text, DecimalMark::Comma)),
            bits_of(Some(number)),
            "{text:?}"
        );
    }
}

#[test]
fn thousands_spaces_infinities_nan_and_texts_with_no_number_are_not_a_number_with_the_comma() {
    for text in [
        "1.234,5", "1,5 ", "Inf", "inf", "NaN", "infinity", "1e999", "-", "NA", ",", "e5", "1e",
        "1.5",
    ] {
        assert_eq!(
            bits_of(parse_float(text, DecimalMark::Comma)),
            None,
            "{text:?}"
        );
    }
}

#[test]
fn a_number_with_the_point_is_read_with_the_point_and_not_the_comma() {
    assert_eq!(
        bits_of(parse_float("1.5", DecimalMark::Point)),
        bits_of(Some(1.5))
    );
    assert_eq!(bits_of(parse_float("1,5", DecimalMark::Point)), None);
    assert_eq!(
        bits_of(parse_float("0.1", DecimalMark::Point)),
        bits_of(Some(0.1))
    );
}

#[test]
fn true_and_false_in_any_case_are_booleans() {
    let cases = [
        ("TRUE", true),
        ("true", true),
        ("True", true),
        ("FALSE", false),
        ("false", false),
    ];
    for (text, is_true) in cases {
        assert_eq!(parse_boolean(text), Some(is_true), "{text:?}");
    }
}

#[test]
fn yes_one_spanish_verdadero_and_t_are_not_booleans() {
    for text in ["yes", "1", "VERDADERO", "T"] {
        assert_eq!(parse_boolean(text), None, "{text:?}");
    }
}

#[test]
fn a_float_is_written_as_javascript_writes_it_with_the_point() {
    let cases = [
        (1.0, "1"),
        (1.5, "1.5"),
        (0.1 + 0.2, "0.30000000000000004"),
        (1e20, "100000000000000000000"),
        (1e21, "1e+21"),
        (1.234_567_890_123_456_8e21, "1.2345678901234568e+21"),
        (1e-6, "0.000001"),
        (1e-7, "1e-7"),
        (1.5e-7, "1.5e-7"),
        (-0.0, "0"),
        (5e-324, "5e-324"),
        (f64::MAX, "1.7976931348623157e+308"),
        (f64::INFINITY, "Infinity"),
        (f64::NAN, "NaN"),
        (-1.5, "-1.5"),
        (-1.5e-7, "-1.5e-7"),
    ];
    for (number, text) in cases {
        assert_eq!(float_text(number, DecimalMark::Point), text, "{number:e}");
    }
}

#[test]
fn a_float_written_with_the_comma_has_the_comma_for_its_point() {
    assert_eq!(float_text(1.5, DecimalMark::Comma), "1,5");
    assert_eq!(float_text(1.5e-7, DecimalMark::Comma), "1,5e-7");
}

#[test]
#[expect(
    clippy::excessive_precision,
    reason = "each literal is the exact value of its float, which the tie is about"
)]
fn a_float_halfway_between_two_shortest_texts_takes_the_one_whose_last_digit_is_even() {
    assert_eq!(
        float_text(100_000_000_000_000.125, DecimalMark::Point),
        "100000000000000.12"
    );
    assert_eq!(
        float_text(12_345_678_901_234.062_5, DecimalMark::Point),
        "12345678901234.062"
    );
}
