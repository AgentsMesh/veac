use super::super::parse;
use crate::program::expression::ast::ExpressionKind;
use crate::program::expression::lexer::lex;

fn call(source: &str) -> Vec<crate::program::expression::ast::CallArgument> {
    let expression = parse(lex(source).unwrap()).unwrap();
    let ExpressionKind::Call { arguments, .. } = expression.kind else {
        panic!("expected call")
    };
    arguments
}

#[test]
fn positional_arguments_have_no_labels() {
    let arguments = call("pair(1, 2)");
    assert_eq!(arguments.len(), 2);
    assert!(arguments.iter().all(|argument| argument.label.is_none()));
}

#[test]
fn named_arguments_keep_label_value_spans_and_source_order() {
    let source = "pair(second: effect(2), first: 1)";
    let arguments = call(source);
    assert_eq!(arguments.len(), 2);
    let labels = arguments
        .iter()
        .map(|argument| argument.label.as_ref().unwrap().name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(labels, ["second", "first"]);
    assert_eq!(
        &source[arguments[0].label.as_ref().unwrap().span.clone()],
        "second"
    );
    assert_eq!(&source[arguments[0].value.span.clone()], "effect(2)");
    assert_eq!(&source[arguments[1].value.span.clone()], "1");
}

#[test]
fn parser_retains_mixed_arguments_for_typed_diagnostics() {
    let arguments = call("pair(first: 1, 2)");
    assert!(arguments[0].label.is_some());
    assert!(arguments[1].label.is_none());
}

#[test]
fn calls_accept_a_trailing_comma_for_multiline_agent_edits() {
    let arguments = call("pair(first: 1, second: 2,)");
    assert_eq!(arguments.len(), 2);
    assert_eq!(arguments[0].label.as_ref().unwrap().name, "first");
    assert_eq!(arguments[1].label.as_ref().unwrap().name, "second");
}

#[test]
fn empty_calls_remain_distinct_from_trailing_comma_calls() {
    assert!(call("pair()").is_empty());
    assert!(call("pair(1,)").len() == 1);
}
