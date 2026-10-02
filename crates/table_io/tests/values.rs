//! The rules of a value and the conversion of a column, each case of "How
//! it is verified" of `docs/specs/values.md` a literal input and its
//! literal output.

use table_io::{
    ColumnType, ColumnValues, ConversionFailure, DecimalMark, convert_column, float_text,
    is_missing, parse_boolean, parse_float, parse_integer,
};

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
        (f64::NEG_INFINITY, "-Infinity"),
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

#[test]
fn a_float_near_a_halfway_point_that_is_not_a_tie_keeps_its_shortest_digits() {
    // Each float by the bits node printed for it: the first two round to a
    // halfway point their exact value is not, the third and the fourth have
    // lower digits that do not read back as the same float, the last is
    // 2^-24, a power of two, whose gap below is half the gap above.
    let cases = [
        (0x3e89_32dc_447b_6340, "1.8774474796234095e-7"),
        (0x3f15_012a_7a5c_6078, "0.00008012601628271273"),
        (0x4414_91df_5f3e_2460, "94861526562499990000"),
        (0x3dec_0000_0000_0000, "2.0372681319713593e-10"),
        (0x3e70_0000_0000_0000, "5.960464477539063e-8"),
    ];
    for (bits, text) in cases {
        assert_eq!(
            float_text(f64::from_bits(bits), DecimalMark::Point),
            text,
            "{bits:#x}"
        );
    }
}

/// A text column of `texts`, None for a missing value.
fn texts(texts: &[Option<&str>]) -> ColumnValues {
    ColumnValues::Text(texts.iter().map(|text| text.map(str::to_owned)).collect())
}

/// What a conversion says when `num_failed` values do not convert, the
/// first in `first_row` with the text `first_text`.
fn failure(num_failed: u64, first_row: u32, first_text: &str) -> ConversionFailure {
    ConversionFailure {
        num_failed,
        first_row,
        first_text: first_text.to_owned(),
    }
}

#[test]
fn whole_numbers_of_a_text_column_convert_to_integers_and_a_missing_value_stays_missing() {
    assert_eq!(
        convert_column(
            &texts(&[Some("1"), Some("2"), None]),
            ColumnType::Integer,
            DecimalMark::Point
        ),
        Ok(ColumnValues::Integer(vec![Some(1), Some(2), None]))
    );
}

#[test]
fn texts_that_are_not_numbers_fail_to_convert_to_float_with_their_count_and_the_first() {
    assert_eq!(
        convert_column(
            &texts(&[Some("1"), Some("n.d."), Some("2"), Some("x")]),
            ColumnType::Float,
            DecimalMark::Point
        ),
        Err(failure(2, 2, "n.d."))
    );
}

#[test]
fn numbers_of_a_text_column_with_the_comma_convert_to_floats() {
    assert_eq!(
        convert_column(
            &texts(&[Some("1,5"), Some("2")]),
            ColumnType::Float,
            DecimalMark::Comma
        ),
        Ok(ColumnValues::Float(vec![Some(1.5), Some(2.0)]))
    );
}

#[test]
fn a_text_that_is_not_true_or_false_fails_to_convert_to_boolean() {
    assert_eq!(
        convert_column(
            &texts(&[Some("TRUE"), Some("no")]),
            ColumnType::Boolean,
            DecimalMark::Point
        ),
        Err(failure(1, 2, "no"))
    );
}

#[test]
fn an_integer_column_converts_to_float_and_a_missing_value_stays_missing() {
    assert_eq!(
        convert_column(
            &ColumnValues::Integer(vec![Some(3), None]),
            ColumnType::Float,
            DecimalMark::Point
        ),
        Ok(ColumnValues::Float(vec![Some(3.0), None]))
    );
}

#[test]
fn an_integer_a_float_cannot_hold_becomes_the_nearest_float() {
    assert_eq!(
        convert_column(
            &ColumnValues::Integer(vec![Some(9_007_199_254_740_993)]),
            ColumnType::Float,
            DecimalMark::Point
        ),
        Ok(ColumnValues::Float(vec![Some(9_007_199_254_740_992.0)]))
    );
}

#[test]
fn an_integer_past_2_to_the_53_that_a_float_holds_is_that_float() {
    assert_eq!(
        convert_column(
            &ColumnValues::Integer(vec![Some(9_007_199_254_740_994)]),
            ColumnType::Float,
            DecimalMark::Point
        ),
        Ok(ColumnValues::Float(vec![Some(9_007_199_254_740_994.0)]))
    );
}

#[test]
fn a_float_that_is_not_whole_fails_to_convert_to_integer_with_its_text() {
    assert_eq!(
        convert_column(
            &ColumnValues::Float(vec![Some(2.0), Some(3.5)]),
            ColumnType::Integer,
            DecimalMark::Point
        ),
        Err(failure(1, 2, "3.5"))
    );
}

#[test]
fn the_float_2_to_the_63_fails_to_convert_to_integer() {
    assert_eq!(
        convert_column(
            &ColumnValues::Float(vec![Some(9_223_372_036_854_775_808.0)]),
            ColumnType::Integer,
            DecimalMark::Point
        ),
        Err(failure(1, 1, "9223372036854776000"))
    );
}

#[test]
fn the_float_minus_2_to_the_63_converts_to_the_smallest_integer() {
    assert_eq!(
        convert_column(
            &ColumnValues::Float(vec![Some(-9_223_372_036_854_775_808.0)]),
            ColumnType::Integer,
            DecimalMark::Point
        ),
        Ok(ColumnValues::Integer(vec![Some(
            -9_223_372_036_854_775_808
        )]))
    );
}

#[test]
fn whole_floats_convert_to_integers() {
    assert_eq!(
        convert_column(
            &ColumnValues::Float(vec![Some(2.0), Some(3.0)]),
            ColumnType::Integer,
            DecimalMark::Point
        ),
        Ok(ColumnValues::Integer(vec![Some(2), Some(3)]))
    );
}

#[test]
fn a_float_converted_to_text_with_the_comma_is_written_with_the_comma() {
    assert_eq!(
        convert_column(
            &ColumnValues::Float(vec![Some(1.5)]),
            ColumnType::Text,
            DecimalMark::Comma
        ),
        Ok(texts(&[Some("1,5")]))
    );
}

#[test]
fn booleans_converted_to_text_are_true_and_false_in_capitals() {
    assert_eq!(
        convert_column(
            &ColumnValues::Boolean(vec![Some(true), Some(false)]),
            ColumnType::Text,
            DecimalMark::Point
        ),
        Ok(texts(&[Some("TRUE"), Some("FALSE")]))
    );
}

#[test]
fn a_boolean_fails_to_convert_to_integer() {
    assert_eq!(
        convert_column(
            &ColumnValues::Boolean(vec![Some(true)]),
            ColumnType::Integer,
            DecimalMark::Point
        ),
        Err(failure(1, 1, "TRUE"))
    );
}

#[test]
fn an_integer_fails_to_convert_to_boolean() {
    assert_eq!(
        convert_column(
            &ColumnValues::Integer(vec![Some(1)]),
            ColumnType::Boolean,
            DecimalMark::Point
        ),
        Err(failure(1, 1, "1"))
    );
}

#[test]
fn an_integer_column_of_missing_values_converts_to_boolean_as_missing_values() {
    assert_eq!(
        convert_column(
            &ColumnValues::Integer(vec![None, None]),
            ColumnType::Boolean,
            DecimalMark::Point
        ),
        Ok(ColumnValues::Boolean(vec![None, None]))
    );
}

#[test]
fn integers_convert_to_their_digits_and_a_missing_value_stays_missing() {
    assert_eq!(
        convert_column(
            &ColumnValues::Integer(vec![Some(1), Some(-3), Some(i64::MIN), None]),
            ColumnType::Text,
            DecimalMark::Point
        ),
        Ok(texts(&[
            Some("1"),
            Some("-3"),
            Some("-9223372036854775808"),
            None
        ]))
    );
}

#[test]
fn a_text_column_converted_to_text_is_itself() {
    assert_eq!(
        convert_column(
            &texts(&[Some("a"), None]),
            ColumnType::Text,
            DecimalMark::Point
        ),
        Ok(texts(&[Some("a"), None]))
    );
}

#[test]
fn a_float_fails_to_convert_to_boolean() {
    assert_eq!(
        convert_column(
            &ColumnValues::Float(vec![Some(1.0)]),
            ColumnType::Boolean,
            DecimalMark::Point
        ),
        Err(failure(1, 1, "1"))
    );
}

#[test]
fn a_boolean_fails_to_convert_to_float() {
    assert_eq!(
        convert_column(
            &ColumnValues::Boolean(vec![Some(true)]),
            ColumnType::Float,
            DecimalMark::Point
        ),
        Err(failure(1, 1, "TRUE"))
    );
}

#[test]
fn a_float_that_fails_to_convert_with_the_comma_is_named_with_the_comma() {
    assert_eq!(
        convert_column(
            &ColumnValues::Float(vec![Some(3.5)]),
            ColumnType::Integer,
            DecimalMark::Comma
        ),
        Err(failure(1, 1, "3,5"))
    );
}

#[test]
fn texts_of_missing_values_a_caller_did_not_mark_missing_fail_to_convert_to_float() {
    assert_eq!(
        convert_column(
            &texts(&[Some("x"), Some("NA"), Some(""), None]),
            ColumnType::Float,
            DecimalMark::Point
        ),
        Err(failure(3, 1, "x"))
    );
}

#[test]
fn a_column_gives_its_type_and_its_number_of_rows() {
    let column = ColumnValues::Boolean(vec![Some(true), None, Some(false)]);
    assert_eq!(column.column_type(), ColumnType::Boolean);
    assert_eq!(column.len(), 3);
    assert!(!column.is_empty());
}
