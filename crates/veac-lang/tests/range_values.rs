use veac_lang::program::expression::{evaluate, Environment, PrimitiveType, Value, ValueType};

fn value(source: &str) -> Value {
    evaluate(source, &Environment::new()).unwrap()
}

fn parts(source: &str) -> (i64, i64, i64, u64) {
    let Value::Range(value) = value(source) else {
        panic!("range expression must produce a range value");
    };
    (value.start(), value.end(), value.step(), value.count())
}

fn canonical(source: &str, expected: &str) {
    let original = value(source);
    let rendered = original.render();
    assert_eq!(rendered, expected);
    let round_trip = value(&rendered);
    assert_eq!(round_trip, original);
    assert_eq!(round_trip.render(), rendered);
}

#[test]
fn ranges_have_one_canonical_round_trip() {
    canonical("0..10", "0 .. 10");
    canonical("0 .. 10 by 1", "0 .. 10");
    canonical("10..0 by -2", "10 .. 0 by -2");
    canonical("0 .. 10 by -2", "0 .. 10 by -2");
    canonical(
        "-9223372036854775808..9223372036854775807 by 2",
        "-9223372036854775808 .. 9223372036854775807 by 2",
    );
    canonical("{ let value: range<int> = 0 .. 10; value }", "0 .. 10");
}

#[test]
fn half_open_count_handles_direction_remainders_and_i64_extremes() {
    for (source, expected) in [
        ("0 .. 10", (0, 10, 1, 10)),
        ("0 .. 10 by 3", (0, 10, 3, 4)),
        ("10 .. 0 by -2", (10, 0, -2, 5)),
        ("-10 .. 1 by 3", (-10, 1, 3, 4)),
        ("0 .. 10 by -2", (0, 10, -2, 0)),
        ("10 .. 0 by 2", (10, 0, 2, 0)),
        ("4 .. 4 by -1", (4, 4, -1, 0)),
        (
            "-9223372036854775808 .. 9223372036854775807",
            (i64::MIN, i64::MAX, 1, u64::MAX),
        ),
        (
            "9223372036854775807 .. -9223372036854775808 by -1",
            (i64::MAX, i64::MIN, -1, u64::MAX),
        ),
        (
            "-9223372036854775808 .. 9223372036854775807 by 2",
            (i64::MIN, i64::MAX, 2, 1u64 << 63),
        ),
        (
            "9223372036854775807 .. -9223372036854775808 by -9223372036854775808",
            (i64::MAX, i64::MIN, i64::MIN, 2),
        ),
    ] {
        assert_eq!(parts(source), expected, "{source}");
    }
}

#[test]
fn public_constructor_is_checked_and_retains_the_range_type() {
    let value = Value::range(2, 11, 3).unwrap();
    assert_eq!(value.value_type().to_string(), "range<int>");
    assert_eq!(value.render(), "2 .. 11 by 3");
    let error = Value::range(0, 10, 0).unwrap_err();
    assert_eq!(error.code(), "VALUE_RANGE_STEP");

    let error = ValueType::range(PrimitiveType::Scalar.into()).unwrap_err();
    assert_eq!(error.code(), "VALUE_TYPE_RANGE_ELEMENT");
    let error = ValueType::parse("range<time>").unwrap_err();
    assert_eq!(error.code(), "VALUE_TYPE_RANGE_ELEMENT");
    assert_eq!(&"range<time>"[error.span()], "time");
}

#[test]
fn zero_step_reports_the_complete_step_expression_span() {
    for (source, expected) in [("1 .. 9 by 0", "0"), ("1 .. 9 by 1 - 1", "1 - 1")] {
        let error = evaluate(source, &Environment::new()).unwrap_err();
        assert_eq!(error.code(), "EXPRESSION_RANGE_STEP");
        assert_eq!(&source[error.span()], expected);
    }
}

#[test]
fn bounds_and_step_must_each_be_int() {
    for source in [
        "0.0 .. 10",
        "0 .. 10.0",
        "0 .. 10 by 1.0",
        "false .. 10",
        "0 .. 10 by 1s",
    ] {
        let error = evaluate(source, &Environment::new()).unwrap_err();
        assert_eq!(error.code(), "EXPRESSION_RANGE_TYPE", "{source}");
        assert!(error.message().contains("int"));
    }
}

#[test]
fn range_operands_execute_start_then_end_then_step() {
    for (source, expected_span) in [
        (
            "9223372036854775807 + 1 .. 9223372036854775807 * 2 by 9223372036854775807 + 3",
            "9223372036854775807 + 1",
        ),
        (
            "0 .. 9223372036854775807 * 2 by 9223372036854775807 + 3",
            "9223372036854775807 * 2",
        ),
        (
            "0 .. 10 by 9223372036854775807 + 3",
            "9223372036854775807 + 3",
        ),
    ] {
        let error = evaluate(source, &Environment::new()).unwrap_err();
        assert_eq!(error.code(), "EXPRESSION_OVERFLOW");
        assert_eq!(&source[error.span()], expected_span);
    }
}

#[test]
fn range_precedence_is_below_arithmetic_above_relations_and_non_associative() {
    assert_eq!(parts("1 + 2 .. 10 - 1 by 1 + 1"), (3, 9, 2, 3));
    assert_eq!(value("0 .. 2 == 0 .. 1 + 1"), Value::Bool(true));
    assert_eq!(
        evaluate("0 < 1 .. 2", &Environment::new())
            .unwrap_err()
            .code(),
        "EXPRESSION_TYPE"
    );

    let source = "0 .. 1 .. 2";
    let error = evaluate(source, &Environment::new()).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_RANGE_CHAIN");
    assert_eq!(&source[error.span()], "..");
    assert_eq!(error.span().start, source.rfind("..").unwrap());
}

#[test]
fn incomplete_range_forms_report_the_missing_value_at_eof() {
    for source in ["1 ..", "1 .. 2 by"] {
        let error = evaluate(source, &Environment::new()).unwrap_err();
        assert_eq!(error.code(), "EXPRESSION_EXPECTED_VALUE", "{source}");
        assert_eq!(error.span(), source.len()..source.len(), "{source}");
    }
}
