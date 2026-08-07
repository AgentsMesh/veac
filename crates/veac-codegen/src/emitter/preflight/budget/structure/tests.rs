use super::*;

#[test]
fn all_mutated_plan_structure_dimensions_have_stable_codes() {
    let usage = RenderStructureUsage {
        tracks: MAX_TOTAL_TRACKS + 1,
        clips: MAX_TOTAL_CLIPS + 1,
        effects: MAX_TOTAL_EFFECTS + 1,
        masks: MAX_TOTAL_MASKS + 1,
        keyframes: MAX_TOTAL_KEYFRAMES + 1,
        source_curve_segments: MAX_TOTAL_SOURCE_CURVE_SEGMENTS + 1,
        caption_cues: MAX_TOTAL_CAPTION_CUES + 1,
        temporal_programs: MAX_TOTAL_TEMPORAL_PROGRAMS + 1,
        temporal_bindings: MAX_TOTAL_TEMPORAL_BINDINGS + 1,
        temporal_nodes: MAX_TOTAL_TEMPORAL_NODES + 1,
        temporal_provenance: MAX_TOTAL_TEMPORAL_PROVENANCE + 1,
    };
    let mut check = Check::default();

    report(&mut check, usage, "prj_budget");

    let codes: Vec<_> = check.diagnostics.iter().map(|value| value.code).collect();
    assert_eq!(
        codes,
        [
            "PLAN_BUDGET_TRACKS",
            "PLAN_BUDGET_CLIPS",
            "PLAN_BUDGET_EFFECTS",
            "PLAN_BUDGET_MASKS",
            "PLAN_BUDGET_KEYFRAMES",
            "PLAN_BUDGET_SOURCE_CURVE_SEGMENTS",
            "PLAN_BUDGET_CAPTION_CUES",
            "PLAN_BUDGET_TEMPORAL_PROGRAMS",
            "PLAN_BUDGET_TEMPORAL_BINDINGS",
            "PLAN_BUDGET_TEMPORAL_NODES",
            "PLAN_BUDGET_TEMPORAL_PROVENANCE",
        ]
    );
}

#[test]
fn resolved_fixture_usage_counts_clip_owned_work() {
    let plan = crate::unit_tests::emitter_tests::support::resolved(
        &crate::unit_tests::emitter_tests::support::fixture(),
    );
    let actual = usage(&plan);
    assert_eq!(actual.tracks, 1);
    assert_eq!(actual.clips, 1);
    assert_eq!(actual.effects, 0);
    assert_eq!(actual.masks, 0);
    assert_eq!(actual.keyframes, 0);
    assert_eq!(actual.caption_cues, 0);
    assert_eq!(actual.temporal_programs, 0);
    assert_eq!(actual.temporal_bindings, 0);
    assert_eq!(actual.temporal_nodes, 0);
    assert_eq!(actual.temporal_provenance, 0);
}
