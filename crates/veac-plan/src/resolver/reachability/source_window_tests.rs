use std::collections::BTreeMap;

use veac_ir::{
    Clip, ClipSource, FrameSynthesisPolicy, ItemId, PlaybackDirection, Rational, RationalTime,
    SequenceId, SourceMapping, SourceOutOfRangePolicy, SourceTimeInterpolation, SourceTimeMap,
    SourceTimeSegment, TimeRange,
};

use super::source_window::project;

#[test]
fn linear_trim_projects_the_selected_parent_window() {
    let clip = clip(
        100,
        600,
        SourceTimeMap::Linear {
            source_start: time(120),
            rate: Rational::new(1, 1).unwrap(),
            repeat: 1,
            direction: PlaybackDirection::Forward,
        },
    );
    assert_ranges(
        project(&clip, &[range(220, 100)], time(1_000)),
        &[range(240, 100)],
    );
}

#[test]
fn linear_repeat_reverse_projects_each_repeat_to_the_same_source_span() {
    let clip = clip(
        0,
        600,
        SourceTimeMap::Linear {
            source_start: time(100),
            rate: Rational::new(1, 1).unwrap(),
            repeat: 2,
            direction: PlaybackDirection::Reverse,
        },
    );
    assert_ranges(
        project(&clip, &[range(50, 100), range(350, 100)], time(1_000)),
        &[range(250, 100)],
    );
}

#[test]
fn curve_segments_project_only_intersecting_record_segments() {
    let clip = clip(
        0,
        600,
        SourceTimeMap::Curve {
            segments: vec![linear_segment(300, 0, 300), linear_segment(300, 300, 600)],
        },
    );
    assert_ranges(
        project(&clip, &[range(350, 50)], time(1_000)),
        &[range(350, 50)],
    );
}

#[test]
fn hold_points_clamp_to_one_child_timeline_tick_at_both_boundaries() {
    let first = clip(
        0,
        600,
        SourceTimeMap::Curve {
            segments: vec![hold_segment(-10, 600)],
        },
    );
    let last = clip(
        0,
        600,
        SourceTimeMap::Curve {
            segments: vec![hold_segment(600, 600)],
        },
    );
    assert_eq!(
        project(&first, &[range(100, 100)], time(600)),
        vec![range(0, 1)]
    );
    assert_eq!(
        project(&last, &[range(100, 100)], time(600)),
        vec![range(599, 1)]
    );
}

fn clip(start: i64, duration: i64, time_map: SourceTimeMap) -> Clip {
    Clip {
        id: ItemId::new("itm_nested").unwrap(),
        enabled: true,
        record_range: range(start, duration),
        source: ClipSource::Sequence {
            sequence_id: SequenceId::new("seq_child").unwrap(),
        },
        source_mapping: Some(SourceMapping {
            time_map,
            frame_synthesis: FrameSynthesisPolicy::Nearest,
            out_of_range: SourceOutOfRangePolicy::HoldBoth,
        }),
        visual: None,
        audio: None,
        effects: Vec::new(),
        replaceable: None,
        template_editable_text: false,
        metadata: BTreeMap::new(),
    }
}

fn linear_segment(duration: i64, start: i64, end: i64) -> SourceTimeSegment {
    SourceTimeSegment {
        record_duration: time(duration),
        source_start: time(start),
        source_end: time(end),
        interpolation: SourceTimeInterpolation::Linear,
    }
}

fn hold_segment(point: i64, duration: i64) -> SourceTimeSegment {
    SourceTimeSegment {
        record_duration: time(duration),
        source_start: time(point),
        source_end: time(point),
        interpolation: SourceTimeInterpolation::Hold,
    }
}

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 600).unwrap()
}

fn range(start: i64, duration: i64) -> TimeRange {
    TimeRange::new(time(start), time(duration)).unwrap()
}

fn assert_ranges(actual: Vec<TimeRange>, expected: &[TimeRange]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_eq!(
            actual.start.partial_cmp(&expected.start),
            Some(std::cmp::Ordering::Equal)
        );
        assert_eq!(
            actual.end().unwrap().partial_cmp(&expected.end().unwrap()),
            Some(std::cmp::Ordering::Equal)
        );
    }
}
