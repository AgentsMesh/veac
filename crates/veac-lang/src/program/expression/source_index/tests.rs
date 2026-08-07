use crate::source_edit::{
    SourceExpressionPath, SourceExpressionStep as Step, SourceLocalOperation,
};
use std::collections::BTreeSet;

use super::indexed_body_sites;

const SOURCE: &str = r#"{
  let mapped = map([1, 2], fn(value: int) -> int effect pure { value + 1 });
  var current = 0;
  set current = if true {
    fold(mapped, 0, fn(total: int, value: int) -> int effect pure { total + value })
  } else {
    match Choice.A { Choice.A => 1, Choice.B => 2 }
  };
  current
}"#;

#[test]
fn nested_paths_name_locals_callbacks_control_calls_and_collections() {
    assert_path(
        "value + 1",
        vec![
            local(SourceLocalOperation::Let, "mapped", 0),
            Step::CallArgument { ordinal: 1 },
            Step::ClosureBody,
            Step::BlockResult,
        ],
    );
    assert_path(
        "2",
        vec![
            local(SourceLocalOperation::Let, "mapped", 0),
            Step::CallArgument { ordinal: 0 },
            Step::ListItem { ordinal: 1 },
        ],
    );
    assert_path(
        "true",
        vec![
            local(SourceLocalOperation::Set, "current", 2),
            Step::IfCondition,
        ],
    );
    assert_path(
        "total + value",
        vec![
            local(SourceLocalOperation::Set, "current", 2),
            Step::IfThen,
            Step::BlockResult,
            Step::CallArgument { ordinal: 2 },
            Step::ClosureBody,
            Step::BlockResult,
        ],
    );
}

#[test]
fn match_arms_and_block_results_have_stable_semantic_steps() {
    let paths = entries();
    let arm = paths
        .iter()
        .find(|(source, path)| {
            source == "2"
                && path.steps.last()
                    == Some(&Step::MatchArm {
                        pattern: "Choice.B".to_owned(),
                    })
        })
        .expect("Choice.B arm");
    assert_eq!(arm.0, "2");
    assert!(paths
        .iter()
        .any(|(source, path)| { source == "current" && path.steps == [Step::BlockResult] }));
    assert!(paths.iter().any(|(source, path)| {
        source == "map"
            && path.steps
                == [
                    local(SourceLocalOperation::Let, "mapped", 0),
                    Step::CallCallee,
                ]
    }));
}

#[test]
fn complete_statement_ranges_include_the_operation_and_semicolon() {
    let sites = indexed_body_sites(SOURCE).unwrap();
    let statements = sites
        .statements
        .into_iter()
        .map(|entry| (&SOURCE[entry.span], entry.path))
        .collect::<Vec<_>>();
    assert_eq!(
        statements[0].0,
        "let mapped = map([1, 2], fn(value: int) -> int effect pure { value + 1 });"
    );
    assert_eq!(statements[1].0, "var current = 0;");
    assert!(statements[2].0.starts_with("set current = if true"));
    assert!(statements[2].0.ends_with(";"));
    assert!(statements
        .iter()
        .all(|(_, path)| path.ends_in_local_value()));
}

#[test]
fn every_expression_and_statement_path_is_unique_within_its_owner() {
    let sites = indexed_body_sites(SOURCE).unwrap();
    let expression_count = sites.expressions.len();
    let statement_count = sites.statements.len();
    assert_eq!(
        sites
            .expressions
            .into_iter()
            .map(|entry| entry.path)
            .collect::<BTreeSet<_>>()
            .len(),
        expression_count
    );
    assert_eq!(
        sites
            .statements
            .into_iter()
            .map(|entry| entry.path)
            .collect::<BTreeSet<_>>()
            .len(),
        statement_count
    );
}

#[test]
fn valid_parser_nesting_can_exceed_the_old_path_limit_deterministically() {
    let mut expression = "1".to_owned();
    for _ in 0..24 {
        expression = format!("f(fn() -> int effect pure {{ {expression} }})");
    }
    let source = format!("{{ {expression} }}");
    let first = indexed_body_sites(&source).unwrap();
    let second = indexed_body_sites(&source).unwrap();
    assert_eq!(first, second);
    let maximum = first
        .expressions
        .iter()
        .map(|entry| entry.path.steps.len())
        .max()
        .unwrap();
    assert!(maximum > 64, "maximum semantic path depth was {maximum}");
    assert!(maximum <= crate::source_edit::MAX_SOURCE_EXPRESSION_PATH_DEPTH);
    assert!(first.expressions.iter().all(|entry| entry.path.is_valid()));
}

fn assert_path(source: &str, expected: Vec<Step>) {
    assert!(entries().iter().any(|(actual, path)| {
        actual == source && path == &SourceExpressionPath::new(expected.clone())
    }));
}

fn entries() -> Vec<(String, SourceExpressionPath)> {
    indexed_body_sites(SOURCE)
        .unwrap()
        .expressions
        .into_iter()
        .map(|entry| (SOURCE[entry.span].to_owned(), entry.path))
        .collect()
}

fn local(operation: SourceLocalOperation, binding: &str, ordinal: u32) -> Step {
    Step::LocalValue {
        operation,
        binding: binding.to_owned(),
        ordinal,
    }
}
