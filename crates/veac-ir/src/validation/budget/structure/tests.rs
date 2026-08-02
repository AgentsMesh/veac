use super::*;

#[test]
fn every_structural_dimension_has_a_stable_diagnostic() {
    let usage = RenderStructureUsage {
        tracks: MAX_TOTAL_TRACKS + 1,
        clips: MAX_TOTAL_CLIPS + 1,
        effects: MAX_TOTAL_EFFECTS + 1,
        masks: MAX_TOTAL_MASKS + 1,
        keyframes: MAX_TOTAL_KEYFRAMES + 1,
        source_curve_segments: MAX_TOTAL_SOURCE_CURVE_SEGMENTS + 1,
        caption_cues: MAX_TOTAL_CAPTION_CUES + 1,
    };
    let mut validator = Validator::default();

    report(&mut validator, usage, "prj_budget");

    let codes: Vec<_> = validator
        .diagnostics
        .iter()
        .map(|value| value.code.as_str())
        .collect();
    assert_eq!(
        codes,
        [
            "BUDGET_TRACKS",
            "BUDGET_CLIPS",
            "BUDGET_EFFECTS",
            "BUDGET_MASKS",
            "BUDGET_KEYFRAMES",
            "BUDGET_SOURCE_CURVE_SEGMENTS",
            "BUDGET_CAPTION_CUES",
        ]
    );
}

#[test]
fn usage_counts_all_animation_owners_with_saturating_totals() {
    let project = crate::test_support::sample_project();
    let actual = usage(&project.project);
    assert_eq!(actual.tracks, 2);
    assert_eq!(actual.clips, 2);
    assert_eq!(actual.effects, 1);
    assert_eq!(actual.masks, 1);
    assert_eq!(actual.keyframes, 2);
    assert_eq!(actual.caption_cues, 1);

    let mut saturated = RenderStructureUsage {
        tracks: u64::MAX,
        ..RenderStructureUsage::default()
    };
    saturated.add(RenderStructureUsage {
        tracks: 1,
        ..RenderStructureUsage::default()
    });
    assert_eq!(saturated.tracks, u64::MAX);
}
