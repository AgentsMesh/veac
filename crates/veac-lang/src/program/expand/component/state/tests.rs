use super::*;

#[test]
fn cycle_and_depth_are_distinct_fail_closed_guards() {
    let mut cycle = State::default();
    cycle.enter(&key(0), "main.veac", Span::default()).unwrap();
    let error = cycle
        .enter(&key(0), "component.veac", Span::default())
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_COMPONENT_CYCLE");

    let mut depth = State::default();
    for value in 0..MAX_COMPONENT_DEPTH {
        depth
            .enter(&key(value), "main.veac", Span::default())
            .unwrap();
    }
    let error = depth
        .enter(&key(MAX_COMPONENT_DEPTH), "main.veac", Span::default())
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_COMPONENT_DEPTH");
}

#[test]
fn total_instance_nodes_are_bounded_across_siblings() {
    let mut state = State::default();
    for value in 0..MAX_COMPONENT_NODES {
        state
            .enter(&key(value), "main.veac", Span::default())
            .unwrap();
        state.leave();
    }
    let error = state
        .enter(&key(MAX_COMPONENT_NODES), "main.veac", Span::default())
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_COMPONENT_BUDGET");
}

#[test]
fn active_component_frames_share_one_memory_budget() {
    let mut state = State::default();
    state
        .retain_frame("parent.veac", budget::MAX_EXPANDED_BYTES - 1)
        .unwrap();
    assert_eq!(state.remaining_frame_bytes(), 1);
    let error = state.retain_frame("child.veac", 2).unwrap_err();
    assert_eq!(error.code, "PROGRAM_EXPANSION_BUDGET");
    state.release_frame(budget::MAX_EXPANDED_BYTES - 1);
    assert_eq!(state.remaining_frame_bytes(), budget::MAX_EXPANDED_BYTES);
    state.retain_frame("next.veac", 2).unwrap();
}

#[test]
fn parameter_work_accepts_exact_limit_and_rejects_first_over() {
    let mut state = State::default();
    state
        .charge_parameter_work("main.veac", Span::default(), MAX_PARAMETER_EVALUATIONS)
        .unwrap();
    let error = state
        .charge_parameter_work("main.veac", Span::default(), 1)
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_PARAMETER_BUDGET");
}

#[test]
fn failed_parameter_charge_does_not_commit_and_overflow_is_diagnostic() {
    let mut state = State {
        parameter_evaluations: MAX_PARAMETER_EVALUATIONS - 1,
        ..State::default()
    };
    state
        .charge_parameter_work("main.veac", Span::default(), 2)
        .unwrap_err();
    assert_eq!(state.parameter_evaluations, MAX_PARAMETER_EVALUATIONS - 1);
    state
        .charge_parameter_work("main.veac", Span::default(), 1)
        .unwrap();

    state.parameter_evaluations = usize::MAX;
    let error = state
        .charge_parameter_work("main.veac", Span::default(), 1)
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_PARAMETER_BUDGET");
}

fn key(value: usize) -> ComponentKey {
    ComponentKey {
        path: "module.veac".to_owned(),
        name: format!("component-{value}"),
    }
}
