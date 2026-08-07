use super::{value, Environment};
use crate::program::expression::{evaluate, ExactNumber, PrimitiveType, Value, ValueType};

#[test]
fn evaluates_every_literal_kind_and_renders_canonically() {
    assert_eq!(value("1.25").render(), "1.25");
    assert_eq!(value(".5").render(), "0.5");
    assert_eq!(value("1s").render(), "1s");
    assert_eq!(value("250ms").render(), "250ms");
    assert_eq!(value("3us").render(), "3us");
    assert_eq!(value("12.5px").render(), "12.5px");
    assert_eq!(value("75%").render(), "75%");
    assert_eq!(value("-90deg").render(), "-90deg");
    assert_eq!(value("#AAbbCCdd").render(), "#aabbccdd");
    assert_eq!(value("true").render(), "true");
    assert_eq!(value("false").render(), "false");
    assert_eq!(
        value(r#""line\n\t\r\"\\end""#).render(),
        r#""line\n\t\r\"\\end""#
    );
}

#[test]
fn resolves_qualified_symbols_without_coercing_values() {
    let mut environment = Environment::new();
    environment.insert(
        "brand.duration".to_owned(),
        Value::Time(ExactNumber::integer(2)),
    );
    environment.insert("asset".to_owned(), Value::Identifier("cover-art".into()));
    environment.insert("_private".to_owned(), Value::Time(ExactNumber::integer(3)));
    environment.insert(
        "brand.title-size".to_owned(),
        Value::Length(ExactNumber::integer(48)),
    );
    assert_eq!(
        evaluate("brand.duration + 500ms", &environment)
            .unwrap()
            .render(),
        "2500ms"
    );
    assert_eq!(
        evaluate("asset", &environment).unwrap().render(),
        "cover-art"
    );
    assert_eq!(evaluate("_private", &environment).unwrap().render(), "3s");
    assert_eq!(
        evaluate("brand.title-size", &environment).unwrap().render(),
        "48px"
    );
}

#[test]
fn symbols_and_comments_share_the_surface_lexical_contract() {
    let environment = [
        (
            "title-card".to_owned(),
            Value::Scalar(ExactNumber::integer(2)),
        ),
        ("_offset".to_owned(), Value::Scalar(ExactNumber::integer(1))),
    ]
    .into_iter()
    .collect();
    assert_eq!(
        evaluate(
            "title-card /* reusable */ + // caller override\n _offset",
            &environment
        )
        .unwrap()
        .render(),
        "3.0"
    );
    let oversized = "a".repeat(129);
    for (source, code) in [
        ("标题", "EXPRESSION_LEX_CHARACTER"),
        ("brand.标题", "EXPRESSION_LEX_CHARACTER"),
        (&oversized, "EXPRESSION_SYMBOL"),
    ] {
        assert_eq!(
            evaluate(source, &environment).unwrap_err().code(),
            code,
            "{source}"
        );
    }
    assert_eq!(evaluate("true", &environment).unwrap(), Value::Bool(true));
}

#[test]
fn unicode_text_and_control_escapes_round_trip() {
    assert_eq!(value(r#""你好，VEAC 🙂""#).render(), r#""你好，VEAC 🙂""#);
    assert_eq!(value(r#""a\u{0}b""#), Value::Text("a\0b".into()));
    let rendered = Value::Text("a\0b".into()).render();
    assert_eq!(value(&rendered), Value::Text("a\0b".into()));
}

#[test]
fn the_identifier_constructor_creates_safe_source_literals() {
    assert_eq!(
        value(r#"identifier("cover-art")"#),
        Value::Identifier("cover-art".into())
    );
    assert_eq!(
        value(r#"identifier("voice_over")"#),
        Value::Identifier("voice_over".into())
    );
    for source in [
        r#"identifier("bad id")"#,
        r#"identifier("-leading")"#,
        r#"identifier("true")"#,
        r#"identifier("标题")"#,
        r#"identifier("bad;item")"#,
    ] {
        assert_eq!(
            evaluate(source, &Environment::new()).unwrap_err().code(),
            "EXPRESSION_IDENTIFIER"
        );
    }
    assert_eq!(
        evaluate(r#"resource("voice_over")"#, &Environment::new())
            .unwrap_err()
            .code(),
        "EXPRESSION_UNKNOWN_FUNCTION"
    );
}

#[test]
fn reports_primitive_and_semantic_value_types() {
    let cases = [
        (value("1"), PrimitiveType::Integer, "int"),
        (value("1.0"), PrimitiveType::Scalar, "scalar"),
        (value("1s"), PrimitiveType::Time, "time"),
        (value("1px"), PrimitiveType::Length, "length"),
        (value("1%"), PrimitiveType::Percent, "percent"),
        (value("1deg"), PrimitiveType::Angle, "angle"),
        (value(r#""x""#), PrimitiveType::Text, "text"),
        (value("#abcdef"), PrimitiveType::Color, "color"),
        (value("true"), PrimitiveType::Boolean, "bool"),
    ];
    for (value, kind, name) in cases {
        assert_eq!(value.primitive_kind(), Some(kind));
        assert_eq!(value.value_type(), ValueType::primitive(kind));
        assert_eq!(kind.as_str(), name);
        assert_eq!(kind.to_string(), name);
    }
    assert_eq!(
        Value::Identifier("x".into()).primitive_kind(),
        Some(PrimitiveType::Identifier)
    );
    assert_eq!(PrimitiveType::Identifier.as_str(), "identifier");
    assert!(Value::from_numeric(PrimitiveType::Text, ExactNumber::integer(1)).is_none());
}

#[test]
fn fractional_dimensions_render_as_parseable_expressions() {
    assert_eq!(value("1px / 3.0").render(), "1px / 3.0");
    assert_eq!(value("1s / 3.0").render(), "1s / 3.0");
    let large = ExactNumber::new(i128::MAX, 2).unwrap();
    assert_eq!(Value::Time(large).render(), format!("{}s / 2.0", i128::MAX));
}
