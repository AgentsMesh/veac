use crate::authoring::Span;

use super::Budget;

#[test]
fn exact_node_and_work_limits_are_accepted() {
    let mut budget = Budget::with_limits(2, 3, 10);
    budget
        .component("main.veac", Span::default(), 2, 4, 6)
        .unwrap();
    budget
        .component("main.veac", Span::default(), 1, 0, 0)
        .unwrap();

    let error = budget
        .component("main.veac", Span::default(), 0, 0, 0)
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_DEFINITION_BUDGET");
    assert!(error.message.contains("validation units"));
}

#[test]
fn work_limit_is_aggregate_and_does_not_commit_failed_work() {
    let mut budget = Budget::with_limits(3, usize::MAX, 10);
    budget
        .component("main.veac", Span::default(), 1, 2, 3)
        .unwrap();

    let error = budget
        .component("main.veac", Span::default(), 1, 3, 3)
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_DEFINITION_BUDGET");
    assert!(error.message.contains("work"));

    budget
        .component("main.veac", Span::default(), 1, 2, 3)
        .unwrap();
}

#[test]
fn arithmetic_overflow_is_a_budget_diagnostic() {
    let mut budget = Budget::with_limits(usize::MAX, usize::MAX, usize::MAX);
    budget.work_bytes = usize::MAX;
    let error = budget
        .component("main.veac", Span::default(), 0, 1, 0)
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_DEFINITION_BUDGET");
    assert!(error.message.contains("overflowed"));
}

#[test]
fn expanded_component_nodes_are_aggregate() {
    let mut budget = Budget::with_limits(usize::MAX, 2, usize::MAX);
    budget
        .component("main.veac", Span::default(), 2, 0, 0)
        .unwrap();
    let error = budget
        .component("main.veac", Span::default(), 1, 0, 0)
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_DEFINITION_BUDGET");
    assert!(error.message.contains("component nodes"));
}

#[test]
fn presets_and_components_share_validation_and_work_totals() {
    let mut budget = Budget::with_limits(2, usize::MAX, 10);
    budget.preset("style.veac", Span::default(), 2, 3).unwrap();
    budget
        .component("card.veac", Span::default(), 1, 2, 3)
        .unwrap();

    let error = budget
        .preset("other.veac", Span::default(), 0, 0)
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_DEFINITION_BUDGET");
    assert!(error.message.contains("validation units"));
}
