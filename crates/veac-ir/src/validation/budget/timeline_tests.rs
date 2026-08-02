use super::*;

#[test]
fn descending_curve_uses_source_span_for_reverse_budget() {
    let mut project = crate::test_support::sample_project();
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    clip.source_mapping.as_mut().unwrap().time_map = SourceTimeMap::Curve {
        segments: vec![SourceTimeSegment {
            record_duration: RationalTime::new(600, 600).unwrap(),
            source_start: RationalTime::new(18_001, 600).unwrap(),
            source_end: RationalTime::new(0, 600).unwrap(),
            interpolation: SourceTimeInterpolation::Linear,
        }],
    };
    assert!(reverse_exceeds(clip));
}
