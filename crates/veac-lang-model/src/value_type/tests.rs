use super::*;

#[path = "tests/domain.rs"]
mod domain;

fn primitive(value: PrimitiveType) -> ValueType {
    value.into()
}

#[test]
fn canonical_display_and_parse_round_trip_recursive_types() {
    for source in [
        "int",
        "list<time>",
        "range<int>",
        "map<text, list<(time, scalar)>>",
        "(int, list<time>, map<identifier, text>)",
        "fn() -> bool effect pure",
        "fn(list<time>, (int, scalar)) -> map<text, time> effect pure",
        "Project",
        "list<Sequence>",
        "fn(Canvas, FrameRate) -> Project effect pure",
    ] {
        let parsed = ValueType::parse(source).unwrap();
        assert_eq!(parsed.to_string(), source);
        assert_eq!(ValueType::parse(&parsed.to_string()).unwrap(), parsed);
    }
    assert_eq!(
        ValueType::parse(" fn( list<time> ,int)->bool effect pure ")
            .unwrap()
            .to_string(),
        "fn(list<time>, int) -> bool effect pure"
    );
}

#[test]
fn map_keys_and_tuple_arity_are_valid_by_construction() {
    for key in [PrimitiveType::Text, PrimitiveType::Identifier] {
        assert!(ValueType::map(primitive(key), primitive(PrimitiveType::Time)).is_ok());
    }
    let error = ValueType::map(
        primitive(PrimitiveType::Time),
        primitive(PrimitiveType::Text),
    )
    .unwrap_err();
    assert_eq!(error.code(), "VALUE_TYPE_MAP_KEY");
    for elements in [Vec::new(), vec![primitive(PrimitiveType::Time)]] {
        assert_eq!(
            ValueType::tuple(elements).unwrap_err().code(),
            "VALUE_TYPE_TUPLE_ARITY"
        );
    }
    assert_eq!(
        ValueType::function(
            Vec::new(),
            primitive(PrimitiveType::Time),
            FunctionEffect::Pure,
        )
        .unwrap()
        .to_string(),
        "fn() -> time effect pure"
    );
}

#[test]
fn range_type_only_accepts_int_elements() {
    let integer = primitive(PrimitiveType::Integer);
    assert_eq!(ValueType::range(integer).unwrap().to_string(), "range<int>");
    let error = ValueType::range(primitive(PrimitiveType::Time)).unwrap_err();
    assert_eq!(error.code(), "VALUE_TYPE_RANGE_ELEMENT");
    let parsed = ValueType::parse("range<time>").unwrap_err();
    assert_eq!(parsed.code(), "VALUE_TYPE_RANGE_ELEMENT");
    assert_eq!(&"range<time>"[parsed.span()], "time");
}

#[test]
fn structural_depth_accepts_32_and_rejects_33() {
    let mut value = primitive(PrimitiveType::Time);
    for _ in 1..MAX_VALUE_TYPE_DEPTH {
        value = ValueType::list(value).unwrap();
    }
    assert_eq!(value.depth(), MAX_VALUE_TYPE_DEPTH);
    assert_eq!(
        ValueType::list(value).unwrap_err().code(),
        "VALUE_TYPE_DEPTH"
    );

    let accepted = format!(
        "{}time{}",
        "list<".repeat(MAX_VALUE_TYPE_DEPTH - 1),
        ">".repeat(MAX_VALUE_TYPE_DEPTH - 1)
    );
    assert_eq!(ValueType::parse(&accepted).unwrap().depth(), 32);
    let rejected = format!(
        "{}time{}",
        "list<".repeat(MAX_VALUE_TYPE_DEPTH),
        ">".repeat(MAX_VALUE_TYPE_DEPTH)
    );
    assert_eq!(
        ValueType::parse(&rejected).unwrap_err().code(),
        "VALUE_TYPE_DEPTH"
    );
}

#[test]
fn structural_arity_accepts_64_and_rejects_65() {
    let values = vec![primitive(PrimitiveType::Integer); MAX_VALUE_TYPE_ARITY];
    assert_eq!(
        ValueType::tuple(values.clone())
            .unwrap()
            .to_string()
            .matches("int")
            .count(),
        64
    );
    let mut oversized = values;
    oversized.push(primitive(PrimitiveType::Integer));
    assert_eq!(
        ValueType::tuple(oversized.clone()).unwrap_err().code(),
        "VALUE_TYPE_ARITY"
    );
    assert_eq!(
        ValueType::function(
            oversized,
            primitive(PrimitiveType::Time),
            FunctionEffect::Pure,
        )
        .unwrap_err()
        .code(),
        "VALUE_TYPE_ARITY"
    );
}

#[test]
fn canonical_parser_reports_precise_invalid_fragments() {
    let map = ValueType::parse("map<time, text>").unwrap_err();
    assert_eq!(map.code(), "VALUE_TYPE_MAP_KEY");
    assert_eq!(&"map<time, text>"[map.span()], "time");
    for (source, code) in [
        ("()", "VALUE_TYPE_TUPLE_ARITY"),
        ("(time)", "VALUE_TYPE_TUPLE_ARITY"),
        ("list<time", "VALUE_TYPE_PARSE"),
        ("time trailing", "VALUE_TYPE_PARSE"),
    ] {
        assert_eq!(
            ValueType::parse(source).unwrap_err().code(),
            code,
            "{source}"
        );
    }
}

#[test]
fn parser_error_messages_cover_unknown_missing_arrow_and_arity_failures() {
    let unknown = ValueType::parse("mystery").unwrap_err();
    assert_eq!(unknown.code(), "VALUE_TYPE_PARSE");
    assert!(unknown.message().contains("unknown value type"));

    let arrow = ValueType::parse("fn(int) int").unwrap_err();
    assert!(arrow.message().contains("expected `->`"));

    let effect = ValueType::parse("fn() -> int effect unknown").unwrap_err();
    assert_eq!(effect.code(), "VALUE_TYPE_PARSE");
    assert!(effect.message().contains("function effect"));

    let parameters = std::iter::repeat_n("int", MAX_VALUE_TYPE_ARITY + 1)
        .collect::<Vec<_>>()
        .join(", ");
    let arity = ValueType::parse(&format!("fn({parameters}) -> int effect pure")).unwrap_err();
    assert_eq!(arity.code(), "VALUE_TYPE_ARITY");
}
