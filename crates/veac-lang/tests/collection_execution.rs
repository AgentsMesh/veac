use veac_lang::program::expression::{
    evaluate, evaluate_in, Environment, ExpressionContext, FunctionDefinition, FunctionEffect,
    FunctionParameter, PrimitiveType, ValueType,
};

fn rendered(source: &str) -> String {
    evaluate(source, &Environment::new()).unwrap().render()
}

fn integer_type() -> ValueType {
    ValueType::primitive(PrimitiveType::Integer)
}

#[test]
fn list_and_range_aggregates_execute_in_canonical_order() {
    assert_eq!(
        rendered("map([1, 2, 3], fn(value: int) -> int effect pure { value * 2 })"),
        "[2, 4, 6]"
    );
    assert_eq!(
        rendered("filter(0 .. 6, fn(value: int) -> bool effect pure { value > 2 })"),
        "[3, 4, 5]"
    );
    assert_eq!(
        rendered(
            "fold(1 .. 5, 0, fn(total: int, value: int) -> int effect pure { total + value })"
        ),
        "10"
    );
}

#[test]
fn descending_ranges_and_empty_inputs_keep_collection_semantics() {
    assert_eq!(
        rendered("map(5 .. 0 by -2, fn(value: int) -> int effect pure { value })"),
        "[5, 3, 1]"
    );
    assert_eq!(
        rendered("filter([], fn(value: int) -> bool effect pure { true })"),
        "{ let value: list<int> = []; value }"
    );
    assert_eq!(
        rendered("fold([], 42, fn(total: int, value: int) -> int effect pure { total + value })"),
        "42"
    );
}

#[test]
fn map_inputs_materialize_one_entry_tuple_per_canonical_entry() {
    let source = "map(#{\"b\": 2, \"a\": 1}, \
        fn(entry: (text, int)) -> (text, int) effect pure { entry })";
    assert_eq!(rendered(source), "[(\"a\", 1), (\"b\", 2)]");
}

#[test]
fn for_is_an_expression_with_map_order_and_lexical_capture_snapshot() {
    let source = "{ let offset = 3; for value in 0 .. 3 { value + offset } }";
    assert_eq!(rendered(source), "[3, 4, 5]");
}

#[test]
fn higher_order_map_helpers_use_their_static_pure_callback_contract() {
    let list = ValueType::list(integer_type()).unwrap();
    let callback =
        ValueType::function(vec![integer_type()], integer_type(), FunctionEffect::Pure).unwrap();
    let definition = FunctionDefinition::new(
        "apply_values",
        vec![
            FunctionParameter::new("values", list.clone()),
            FunctionParameter::new("callback", callback),
        ],
        list,
        "{ map(values, callback) }",
    );
    let context = veac_lang::program::expression::compile_functions(
        &ExpressionContext::empty(),
        &[definition],
    )
    .unwrap();
    let value = evaluate_in(
        "apply_values([1, 2], fn(value: int) -> int effect pure { value + 1 })",
        &Environment::new(),
        &context,
    )
    .unwrap();
    assert_eq!(value.render(), "[2, 3]");
}

#[test]
fn higher_order_map_helpers_reject_local_and_any_callback_bounds() {
    for effect in [FunctionEffect::Local, FunctionEffect::Any] {
        let integer = integer_type();
        let callback = ValueType::function(vec![integer.clone()], integer.clone(), effect).unwrap();
        let list = ValueType::list(integer).unwrap();
        let definition = FunctionDefinition::new(
            "apply_values",
            vec![
                FunctionParameter::new("values", list.clone()),
                FunctionParameter::new("callback", callback),
            ],
            list,
            "{ map(values, callback) }",
        );
        let error = veac_lang::program::expression::compile_functions(
            &ExpressionContext::empty(),
            &[definition],
        )
        .unwrap_err();
        assert_eq!(error.code(), "EXPRESSION_COLLECTION_CALLBACK_EFFECT");
    }
}

#[test]
fn source_callbacks_cannot_hide_local_mutation_inside_collections() {
    for operation in ["map", "filter", "fold"] {
        let arguments = match operation {
            "map" | "filter" => "[1], ",
            "fold" => "[1], 0, ",
            _ => unreachable!(),
        };
        let parameters = if operation == "fold" {
            "total: int, value: int"
        } else {
            "value: int"
        };
        let result = if operation == "filter" { "bool" } else { "int" };
        let body = if operation == "filter" {
            "true"
        } else {
            "local"
        };
        let source = format!(
            "{operation}({arguments}fn({parameters}) -> {result} effect local {{ \
             var local = value; set local = local + 1; {body} }})"
        );
        let error = evaluate(&source, &Environment::new()).unwrap_err();
        assert_eq!(error.code(), "EXPRESSION_COLLECTION_CALLBACK_EFFECT");
    }
}

#[test]
fn oversized_lazy_ranges_fail_before_iteration_or_allocation() {
    let source = "map(-9223372036854775808 .. 9223372036854775807, \
        fn(value: int) -> int effect pure { value })";
    let error = evaluate(source, &Environment::new()).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert!(error.message().contains("aggregate iteration"));
}
