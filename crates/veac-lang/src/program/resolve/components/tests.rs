use std::sync::Arc;

use crate::program::expand::definition::Budget;
use crate::program::model::{Scope, SurfaceFile};
use crate::program::parser;

use super::resolve;

mod retained_budget;

#[test]
fn sources_without_components_do_not_consume_definition_budget() {
    let mut scope = Scope::default();
    let mut catalog = Default::default();
    resolve_in(
        &source(""),
        &mut scope,
        &mut catalog,
        &mut Budget::with_limits(0, 0, 0),
    )
    .unwrap();
    assert!(catalog.is_empty());
}

#[test]
fn components_from_one_file_share_source_and_capture_storage() {
    let file = source(
        r#"component sequence first { body {} }
component sequence second { body {} }"#,
    );
    let mut scope = Scope::default();
    let mut catalog = Default::default();
    resolve_in(&file, &mut scope, &mut catalog, &mut Budget::default()).unwrap();

    let first = &catalog[&scope.components["first"]];
    let second = &catalog[&scope.components["second"]];
    assert!(Arc::ptr_eq(&first.source, &second.source));
    assert!(!Arc::ptr_eq(first, second));
    assert!(Arc::ptr_eq(&first.captured, &second.captured));
    assert!(Arc::ptr_eq(&first.captured.values, &scope.values));
    assert!(Arc::ptr_eq(&first.captured.presets, &scope.presets));
}

#[test]
fn definition_budget_is_shared_across_resolution_calls() {
    let mut budget = Budget::with_limits(3, usize::MAX, usize::MAX);
    let mut catalog = Default::default();
    let mut first_scope = Scope::default();
    resolve_in(
        &source("component sequence first { body {} }"),
        &mut first_scope,
        &mut catalog,
        &mut budget,
    )
    .unwrap();

    let mut second_scope = Scope::default();
    let error = resolve_in(
        &source("component sequence second { body {} }"),
        &mut second_scope,
        &mut catalog,
        &mut budget,
    )
    .unwrap_err();
    assert_eq!(error.code, "PROGRAM_DEFINITION_BUDGET");
    assert!(error.message.contains("validation units"));
}

#[test]
fn component_cycles_take_priority_over_an_exhausted_definition_budget() {
    let file = source(
        r#"component sequence recursive {
  instance sequence @again from recursive {}
  body {}
}"#,
    );
    let mut scope = Scope::default();
    let mut catalog = Default::default();
    let error = resolve_in(
        &file,
        &mut scope,
        &mut catalog,
        &mut Budget::with_limits(0, 0, 0),
    )
    .unwrap_err();
    assert_eq!(error.code, "PROGRAM_COMPONENT_CYCLE");
}

#[test]
fn nested_definition_expansion_charges_actual_component_nodes() {
    let file = source(
        r#"component sequence leaf { body {} }
component sequence parent {
  instance sequence @child from leaf {}
  body {}
}"#,
    );
    let mut scope = Scope::default();
    let mut catalog = Default::default();
    let error = resolve_in(
        &file,
        &mut scope,
        &mut catalog,
        &mut Budget::with_limits(usize::MAX, 3, usize::MAX),
    )
    .unwrap_err();
    assert_eq!(error.code, "PROGRAM_DEFINITION_BUDGET");
    assert!(error.message.contains("component nodes"));
}

#[test]
fn aggregate_work_limit_has_a_stable_diagnostic() {
    let mut scope = Scope::default();
    let mut catalog = Default::default();
    let error = resolve_in(
        &source("component sequence valid { body {} }"),
        &mut scope,
        &mut catalog,
        &mut Budget::with_limits(usize::MAX, usize::MAX, 0),
    )
    .unwrap_err();
    assert_eq!(error.code, "PROGRAM_DEFINITION_BUDGET");
    assert!(error.message.contains("work"));
}

fn resolve_in(
    file: &SurfaceFile,
    scope: &mut Scope,
    catalog: &mut crate::program::model::ComponentCatalog,
    budget: &mut Budget,
) -> Result<std::collections::BTreeSet<String>, crate::program::Diagnostic> {
    let mut retained = super::super::retained::Budget::default();
    resolve(file, scope, catalog, budget, &mut retained)
}

fn source(declarations: &str) -> SurfaceFile {
    let input = format!(
        r#"{declarations}
project validation {{
  settings {{
    timebase 1/1000; canvas 1px by 1px;
    frame-rate 1fps; sample-rate 8000hz;
  }}
  entry sequence main;
  sequence main {{}}
}}"#
    );
    parser::parse("main.veac", &input).unwrap()
}
