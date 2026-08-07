use super::{error, value};
use crate::program::expression::{ExactNumber, PrimitiveType, Value};

#[test]
fn whole_decimal_and_unit_literals_have_distinct_exact_types() {
    assert_eq!(value("42"), Value::Integer(42));
    assert_eq!(value("42.0"), Value::Scalar(ExactNumber::integer(42)));
    assert_eq!(value("42s"), Value::Time(ExactNumber::integer(42)));
    assert_eq!(value("9223372036854775807"), Value::Integer(i64::MAX));
    assert_eq!(value("-9223372036854775808"), Value::Integer(i64::MIN));
    assert_eq!(
        value("9223372036854775808.0"),
        Value::Scalar(ExactNumber::integer(i128::from(i64::MAX) + 1))
    );
    assert_eq!(
        error("9223372036854775808").code(),
        "EXPRESSION_NUMBER_LITERAL"
    );
}

#[test]
fn integer_and_scalar_rendering_preserves_literal_type() {
    let integer = Value::Integer(2);
    let scalar = Value::Scalar(ExactNumber::integer(2));
    assert_eq!(integer.render(), "2");
    assert_eq!(scalar.render(), "2.0");
    assert_eq!(value(&integer.render()), integer);
    assert_eq!(value(&scalar.render()), scalar);
}

#[test]
fn integer_arithmetic_is_checked_and_division_returns_scalar() {
    for (source, expected) in [
        ("20 + 22", Value::Integer(42)),
        ("50 - 8", Value::Integer(42)),
        ("6 * 7", Value::Integer(42)),
        ("+42", Value::Integer(42)),
        ("-42", Value::Integer(-42)),
        ("-9223372036854775807 - 1", Value::Integer(i64::MIN)),
    ] {
        assert_eq!(value(source), expected, "{source}");
    }
    assert_eq!(
        value("1 / 2"),
        Value::Scalar(ExactNumber::new(1, 2).unwrap())
    );
    assert_eq!(value("4 / 2").render(), "2.0");
}

#[test]
fn integer_overflow_and_zero_division_have_stable_diagnostics() {
    for source in [
        "9223372036854775807 + 1",
        "(-9223372036854775807 - 1) - 1",
        "9223372036854775807 * 2",
        "-(-9223372036854775808)",
    ] {
        assert_eq!(error(source).code(), "EXPRESSION_OVERFLOW", "{source}");
    }
    assert_eq!(error("1 / 0").code(), "EXPRESSION_DIVIDE_BY_ZERO");
}

#[test]
fn integers_do_not_implicitly_widen_to_scalar_or_units() {
    for source in [
        "1 + 1.0", "1 * 1.0", "1 / 1.0", "1.0 / 1", "1 * 1s", "1 == 1.0", "1 < 1.0",
    ] {
        assert_eq!(error(source).code(), "EXPRESSION_TYPE", "{source}");
    }
    assert_eq!(error("min(1, 1.0)").code(), "EXPRESSION_CALL_ARGUMENT_TYPE");
}

#[test]
fn integer_relations_and_builtins_remain_exactly_typed() {
    for (source, expected) in [
        ("1 < 2", true),
        ("2 <= 2", true),
        ("3 > 2", true),
        ("3 >= 4", false),
        ("2 == 2", true),
        ("2 != 2", false),
    ] {
        assert_eq!(value(source), Value::Bool(expected), "{source}");
    }
    assert_eq!(value("min(2, 1)"), Value::Integer(1));
    assert_eq!(value("max(2, 1)"), Value::Integer(2));
    assert_eq!(value("clamp(4, 1, 3)"), Value::Integer(3));
}

#[test]
fn integer_kind_and_numeric_reconstruction_are_closed() {
    assert_eq!(PrimitiveType::parse("int"), Some(PrimitiveType::Integer));
    assert!(PrimitiveType::Integer.is_numeric());
    assert_eq!(
        Value::from_numeric(PrimitiveType::Integer, ExactNumber::integer(7)),
        Some(Value::Integer(7))
    );
    assert!(Value::from_numeric(PrimitiveType::Integer, ExactNumber::new(1, 2).unwrap()).is_none());
    assert!(Value::from_numeric(
        PrimitiveType::Integer,
        ExactNumber::integer(i128::from(i64::MAX) + 1)
    )
    .is_none());
}
