#[path = "tests/audio.rs"]
mod audio;
#[path = "tests/multiplicity.rs"]
mod multiplicity;
#[path = "tests/nested.rs"]
mod nested;
#[path = "tests/segment.rs"]
mod segment;
#[path = "tests/support.rs"]
mod support;
#[path = "tests/video.rs"]
mod video;

#[test]
fn graph_rejects_a_reverse_filter_that_bypasses_the_ledger() {
    let plan = support::resolved(&support::fixture());
    let bindings = crate::unit_tests::emitter_tests::support::bindings(&plan);
    let deliverable = &plan.output.deliverables[0];
    let alpha = match &deliverable.kind {
        veac_plan::canonical::DeliverableKind::Video(video) => video.video.alpha,
        _ => unreachable!(),
    };
    let mut context = super::EmitContext::new_visual(&plan, &bindings, deliverable, alpha).unwrap();
    context
        .graph
        .filter(&["0:0"], "reverse@unbudgeted", "unbudgeted");
    context
        .graph
        .filter(&["0:1"], "areverse@unbudgeted", "unbudgeteda");
    context
        .graph
        .filter(&["0:0"], "reverse=future-option", "unbudgetedv");
    assert_eq!(context.graph.reverse_filter_count(), 3);
    let error = context.filter_graph().unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "BACKEND_REVERSE_BUDGET_BYPASS");
}
