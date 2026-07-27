use super::support::*;
use crate::{canonical::*, resolve, ResolvedSourceTimeMap};

#[test]
fn curve_mappings_preserve_forward_reverse_and_hold_semantics() {
    let mappings = [
        mapping(vec![
            segment(200, 0, 300, SourceTimeInterpolation::Linear),
            segment(100, 300, 300, SourceTimeInterpolation::Hold),
            segment(300, 300, 450, SourceTimeInterpolation::Linear),
        ]),
        mapping(vec![
            segment(200, 450, 300, SourceTimeInterpolation::Linear),
            segment(100, 300, 300, SourceTimeInterpolation::Hold),
            segment(300, 300, 0, SourceTimeInterpolation::Linear),
        ]),
        mapping(vec![segment(600, 300, 300, SourceTimeInterpolation::Hold)]),
    ];
    for authored in mappings {
        let mut value = project();
        value.project.sequences[0].tracks[0].clips[0].source_mapping = Some(authored.clone());
        let plan = resolve(&value, None).unwrap().remove(0);
        let mapping = plan.sequences[0].tracks[0].clips[0]
            .source_mapping
            .as_ref()
            .unwrap();
        let ResolvedSourceTimeMap::Curve { segments } = &mapping.time_map else {
            panic!("expected curve mapping")
        };
        let SourceTimeMap::Curve { segments: expected } = authored.time_map else {
            unreachable!()
        };
        assert_eq!(segments, &expected);
        assert_eq!(mapping.frame_synthesis, FrameSynthesisPolicy::Blend);
    }
}

#[test]
fn curve_and_hold_source_bounds_fail_during_resolution() {
    for authored in [
        mapping(vec![segment(
            600,
            0,
            60_000,
            SourceTimeInterpolation::Linear,
        )]),
        mapping(vec![segment(
            600,
            60_000,
            60_000,
            SourceTimeInterpolation::Hold,
        )]),
    ] {
        let mut value = project();
        value.project.sequences[0].tracks[0].clips[0].source_mapping = Some(authored);
        assert!(resolve(&value, None)
            .unwrap_err()
            .diagnostics()
            .iter()
            .any(|item| item.code == "SOURCE_RANGE_OUT_OF_BOUNDS"));
    }
}

fn mapping(segments: Vec<SourceTimeSegment>) -> SourceMapping {
    SourceMapping {
        time_map: SourceTimeMap::Curve { segments },
        frame_synthesis: FrameSynthesisPolicy::Blend,
        out_of_range: SourceOutOfRangePolicy::Strict,
    }
}

fn segment(
    duration: i64,
    start: i64,
    end: i64,
    interpolation: SourceTimeInterpolation,
) -> SourceTimeSegment {
    SourceTimeSegment {
        record_duration: time(duration),
        source_start: time(start),
        source_end: time(end),
        interpolation,
    }
}
