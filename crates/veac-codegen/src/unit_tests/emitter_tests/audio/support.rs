use veac_codegen::emitter::CodegenErrorKind;

use super::super::support::{bindings, emit_video_command};

pub(super) fn silence_only(plan: &veac_plan::ResolvedRenderPlan) {
    let graph = graph(plan);
    assert!(graph.contains("silence"));
    assert!(!graph.contains("trima"));
}

pub(super) fn error(plan: &veac_plan::ResolvedRenderPlan, marker: &str) {
    let error = emit_video_command(plan, &bindings(plan)).unwrap_err();
    assert_eq!(error.diagnostics()[0].kind, CodegenErrorKind::InvalidPlan);
    assert!(error.diagnostics()[0].message.contains(marker));
}

pub(super) fn graph(plan: &veac_plan::ResolvedRenderPlan) -> String {
    emit_video_command(plan, &bindings(plan))
        .unwrap()
        .filter_graph
        .unwrap()
}
