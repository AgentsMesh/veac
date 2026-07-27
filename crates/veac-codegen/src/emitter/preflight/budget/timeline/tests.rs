use super::*;

#[test]
fn only_descending_curve_segments_allocate_reverse_buffers() {
    let mut plan = crate::unit_tests::emitter_tests::support::resolved(
        &crate::unit_tests::emitter_tests::support::fixture(),
    );
    let clip = &mut plan.sequences[0].tracks[0].clips[0];
    let mapping = clip.source_mapping.as_mut().unwrap();
    mapping.time_map = ResolvedSourceTimeMap::Curve {
        segments: vec![SourceTimeSegment {
            record_duration: RationalTime::new(600, 600).unwrap(),
            source_start: RationalTime::new(18_001, 600).unwrap(),
            source_end: RationalTime::new(0, 600).unwrap(),
            interpolation: SourceTimeInterpolation::Linear,
        }],
    };
    assert!(reverse_exceeds(clip));
}
