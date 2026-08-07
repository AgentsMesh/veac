use crate::program::expression::referenced_symbols;

fn code(source: &str) -> &'static str {
    referenced_symbols(source).unwrap_err().code()
}

#[test]
fn block_requires_one_unterminated_tail_expression() {
    assert_eq!(code("{}"), "EXPRESSION_BLOCK_RESULT");
    assert_eq!(code("{ 1; }"), "EXPRESSION_BLOCK_RESULT");
    assert_eq!(code("{ let value = 1; }"), "EXPRESSION_BLOCK_RESULT");
}

#[test]
fn local_declarations_are_closed_and_unique_per_block() {
    assert_eq!(
        code("{ let value = 1; let value = 2; value }"),
        "EXPRESSION_DUPLICATE_LOCAL"
    );
    assert_eq!(code("{ let if = 1; 1 }"), "EXPRESSION_LOCAL_NAME");
    assert_eq!(code("{ let module.value = 1; 1 }"), "EXPRESSION_LOCAL_NAME");
    assert!(referenced_symbols("{ let clip = 1; let apply = clip; apply }").is_ok());
    assert!(referenced_symbols("{ let value = 1; { let value = 2; value } }").is_ok());
}

#[test]
fn conditional_requires_two_block_branches() {
    assert_eq!(code("if true { 1 }"), "EXPRESSION_EXPECTED_TOKEN");
    assert_eq!(code("if true { 1 } else 2"), "EXPRESSION_EXPECTED_TOKEN");
    assert!(referenced_symbols("if true { 1 } else { 2 }").is_ok());
    assert!(referenced_symbols("if enabled { 1 } else { 2 }").is_ok());
}

#[test]
fn comparison_families_are_non_associative() {
    assert_eq!(code("1 < 2 < 3"), "EXPRESSION_COMPARISON_CHAIN");
    assert_eq!(code("1 == 2 != 3"), "EXPRESSION_COMPARISON_CHAIN");
    assert!(referenced_symbols("1 < 2 && 2 < 3 || false").is_ok());
}

#[test]
fn boolean_operator_prefixes_are_closed_tokens() {
    assert_eq!(code("true & false"), "EXPRESSION_LEX_OPERATOR");
    assert_eq!(code("true | false"), "EXPRESSION_LEX_OPERATOR");
    assert_eq!(code("true &&"), "EXPRESSION_EXPECTED_VALUE");
}

#[test]
fn structural_surface_collects_references_in_authored_order_independently_of_storage() {
    let symbols =
        referenced_symbols(r#"[outside, (pair, "x"), #{key: value}, #{"literal": nested}]"#)
            .unwrap();
    assert_eq!(
        symbols.into_iter().collect::<Vec<_>>(),
        ["key", "nested", "outside", "pair", "value"]
    );
    assert!(referenced_symbols("#abcdef == #abcdef").is_ok());
}

#[test]
fn typed_bindings_separate_structured_syntax_from_type_resolution() {
    let source =
        "{ let values: list<map<text, (int, fn(int) -> text effect pure)>> = outside; values }";
    assert_eq!(
        referenced_symbols(source)
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>(),
        ["outside"]
    );
    let unknown = "{ let value: unknown = 1; value }";
    assert!(referenced_symbols(unknown).is_ok());
    assert_eq!(super::error(unknown).code(), "EXPRESSION_TYPE_SYNTAX");
    let invalid_map = "{ let value: map<int, text> = 1; value }";
    assert!(referenced_symbols(invalid_map).is_ok());
    assert_eq!(super::error(invalid_map).code(), "EXPRESSION_TYPE_SYNTAX");
    assert_eq!(
        code("{ let value: fn(int,) -> int effect pure = outside; value }"),
        "EXPRESSION_TYPE_SYNTAX"
    );
}

#[test]
fn tuple_and_collection_separators_are_closed() {
    assert_eq!(code("()"), "EXPRESSION_TUPLE_ARITY");
    assert_eq!(code("(1,)"), "EXPRESSION_TUPLE_ARITY");
    assert_eq!(code("[1,]"), "EXPRESSION_EXPECTED_VALUE");
    assert_eq!(code("#{\"a\": 1,}"), "EXPRESSION_EXPECTED_VALUE");
}
