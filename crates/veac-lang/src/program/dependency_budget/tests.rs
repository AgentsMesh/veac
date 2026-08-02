use super::*;

#[test]
fn active_depth_is_bounded_before_recursion() {
    let mut budget = DependencyBudget::default();
    for _ in 0..MAX_DEPENDENCY_DEPTH {
        budget
            .enter("main.veac", "constant", Span::default())
            .unwrap();
    }
    let error = budget
        .enter("main.veac", "constant", Span::default())
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_DEPENDENCY_DEPTH");
    assert!(error.message.contains("64 active levels"));
}

#[test]
fn total_resolved_nodes_are_bounded() {
    let mut budget = DependencyBudget::default();
    for _ in 0..MAX_DEPENDENCY_NODES {
        budget
            .enter("main.veac", "preset", Span::default())
            .unwrap();
        budget.leave();
    }
    let error = budget
        .enter("main.veac", "preset", Span::default())
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_DEPENDENCY_BUDGET");
    assert!(error.message.contains("16384 resolved nodes"));
}
