use veac_lang::program::expression::{
    evaluate, Environment, MapKeyType, PrimitiveType, Value, ValueType,
};

fn canonical(source: &str, expected: &str) {
    let value = evaluate(source, &Environment::new()).unwrap();
    let rendered = value.render();
    assert_eq!(rendered, expected);
    let round_trip = evaluate(&rendered, &Environment::new()).unwrap();
    assert_eq!(round_trip, value);
    assert_eq!(round_trip.render(), rendered);
}

fn rendered(source: &str) -> String {
    evaluate(source, &Environment::new()).unwrap().render()
}

fn error_code(source: &str) -> String {
    evaluate(source, &Environment::new())
        .unwrap_err()
        .code()
        .to_owned()
}

#[test]
fn structural_literals_have_one_canonical_round_trip() {
    canonical("[1,2,3]", "[1, 2, 3]");
    canonical("(1,\"x\")", "(1, \"x\")");
    canonical("#{\"b\":2,\"a\":1}", "#{ \"a\": 1, \"b\": 2 }");
    canonical(
        "#{identifier(\"b\"):2,identifier(\"a\"):1}",
        "#{ identifier(\"a\"): 1, identifier(\"b\"): 2 }",
    );
}

#[test]
fn structural_equality_is_recursive_and_map_order_independent() {
    assert_eq!(
        rendered("[#{\"a\":(1,[2,3])}] == [#{\"a\":(1,[2,3])}]",),
        "true"
    );
    assert_eq!(
        rendered("#{\"b\":[2],\"a\":[1]} == #{\"a\":[1],\"b\":[2]}"),
        "true"
    );
    assert_eq!(rendered("(1,[2,3]) == (1,[2,4])"), "false");
}

#[test]
fn collection_literals_enforce_local_type_constraints() {
    assert_eq!(error_code("[]"), "EXPRESSION_COLLECTION_TYPE_CONTEXT");
    assert_eq!(error_code("#{}"), "EXPRESSION_COLLECTION_TYPE_CONTEXT");
    assert_eq!(error_code("[1,\"x\"]"), "EXPRESSION_LIST_ELEMENT_TYPE");
    assert_eq!(error_code("#{1:\"x\"}"), "EXPRESSION_MAP_KEY_TYPE");
    assert_eq!(error_code("()"), "EXPRESSION_TUPLE_ARITY");
    assert_eq!(error_code("(1,)"), "EXPRESSION_TUPLE_ARITY");
}

#[test]
fn map_entries_evaluate_key_then_value_from_left_to_right() {
    let key_first = "#{(if 1.0/0.0 > 0.0 {\"a\"} else {\"b\"}):2.0/0.0}";
    let key_error = evaluate(key_first, &Environment::new()).unwrap_err();
    assert_eq!(key_error.code(), "EXPRESSION_DIVIDE_BY_ZERO");
    assert_eq!(&key_first[key_error.span()], "1.0/0.0");

    let value_before_next_key = "#{\"a\":3.0/0.0,(if 4.0/0.0 > 0.0 {\"b\"} else {\"c\"}):5.0}";
    let value_error = evaluate(value_before_next_key, &Environment::new()).unwrap_err();
    assert_eq!(value_error.code(), "EXPRESSION_DIVIDE_BY_ZERO");
    assert_eq!(&value_before_next_key[value_error.span()], "3.0/0.0");
}

#[test]
fn duplicate_map_key_fails_before_its_value_at_the_key_span() {
    let source = "#{\"a\":1.0,\"a\":1.0/0.0}";
    let error = evaluate(source, &Environment::new()).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_DUPLICATE_MAP_KEY");
    assert_eq!(&source[error.span()], "\"a\"");
    assert_eq!(error.span().start, source.rfind("\"a\"").unwrap());
}

#[test]
fn detached_empty_values_render_with_enough_type_context_to_round_trip() {
    let integer = ValueType::primitive(PrimitiveType::Integer);
    let empty_list = Value::list(integer.clone(), Vec::new()).unwrap();
    let rendered_list = empty_list.render();
    assert_eq!(rendered_list, "{ let value: list<int> = []; value }");
    assert_eq!(
        evaluate(&rendered_list, &Environment::new()).unwrap(),
        empty_list
    );

    let empty_map = Value::map(MapKeyType::Text, integer, Vec::new()).unwrap();
    let rendered_map = empty_map.render();
    assert_eq!(rendered_map, "{ let value: map<text, int> = #{}; value }");
    assert_eq!(
        evaluate(&rendered_map, &Environment::new()).unwrap(),
        empty_map
    );
}
