use veac_plan::canonical::*;
use veac_plan::ResolvedSourceTimeMap;

use super::support::{
    compile, evaluate, ffmpeg, node, numeric_expression, numeric_value, plan, program,
};

#[test]
fn every_clip_clock_matches_reference_evaluation() {
    for clock in [
        TemporalClock::SequenceTime,
        TemporalClock::ClipTime,
        TemporalClock::SourceTime,
        TemporalClock::Frame,
        TemporalClock::Progress,
    ] {
        let (plan, binding) = clock_plan(clock);
        assert_sample(&plan, &binding, 0.25);
    }
}

#[test]
fn linear_source_clock_covers_repeat_reverse_and_end_clamping() {
    for (repeat, direction) in [
        (1, PlaybackDirection::Forward),
        (2, PlaybackDirection::Forward),
        (2, PlaybackDirection::Reverse),
    ] {
        let (mut plan, binding) = clock_plan(TemporalClock::SourceTime);
        let clip = &mut plan.sequences[0].tracks[0].clips[0];
        let duration = clip.record_range.duration;
        let repeat_duration =
            RationalTime::new(duration.value / i64::from(repeat), duration.timescale).unwrap();
        let source_start = RationalTime::new(0, duration.timescale).unwrap();
        clip.source_mapping.as_mut().unwrap().time_map = ResolvedSourceTimeMap::Linear {
            source_range_per_repeat: TimeRange::new(source_start, repeat_duration).unwrap(),
            rate: Rational::new(1, 1).unwrap(),
            repeat,
            direction,
        };
        assert_sample(&plan, &binding, 0.17);
        assert_sample(&plan, &binding, seconds(duration));
    }
}

#[test]
fn piecewise_source_clock_covers_hold_linear_and_after_end() {
    let (mut plan, binding) = clock_plan(TemporalClock::SourceTime);
    let clip = &mut plan.sequences[0].tracks[0].clips[0];
    let duration = clip.record_range.duration;
    let first = RationalTime::new(duration.value / 3, duration.timescale).unwrap();
    let second = RationalTime::new(duration.value - first.value, duration.timescale).unwrap();
    clip.source_mapping.as_mut().unwrap().time_map = ResolvedSourceTimeMap::Curve {
        segments: vec![
            SourceTimeSegment {
                record_duration: first,
                source_start: time(200),
                source_end: time(200),
                interpolation: SourceTimeInterpolation::Hold,
            },
            SourceTimeSegment {
                record_duration: second,
                source_start: time(400),
                source_end: time(900),
                interpolation: SourceTimeInterpolation::Linear,
            },
        ],
    };
    for local in [0.1, seconds(first) + 0.1, seconds(duration) + 0.1] {
        assert_sample(&plan, &binding, local);
    }
}

fn clock_plan(clock: TemporalClock) -> (veac_plan::ResolvedRenderPlan, TemporalBindingId) {
    let input_id = TemporalInputId::new(0);
    plan(
        program(
            vec![TemporalInputDeclaration {
                id: input_id,
                value_type: clock.value_type(),
                source: TemporalInputSource::Clock { clock },
            }],
            vec![node(
                0,
                clock.value_type(),
                TemporalNodeKind::Input { input_id },
            )],
            0,
            clock.value_type(),
        ),
        vec![(input_id, clock)],
        Vec::new(),
    )
}

fn assert_sample(plan: &veac_plan::ResolvedRenderPlan, binding: &TemporalBindingId, local: f64) {
    let expected = numeric_value(&evaluate(plan, binding, local));
    let compiled = compile(plan, binding, &format!("{local:.12}")).unwrap();
    let actual = ffmpeg(numeric_expression(&compiled));
    assert!((actual - expected).abs() < 1e-8, "{actual} != {expected}");
}

fn time(milliseconds: i64) -> RationalTime {
    RationalTime::new(milliseconds, 1_000).unwrap()
}

fn seconds(value: RationalTime) -> f64 {
    value.value as f64 / f64::from(value.timescale)
}
