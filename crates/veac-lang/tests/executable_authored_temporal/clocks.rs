use veac_ir::{RationalTime, TemporalClock, TemporalClockOwner, TemporalValue};
use veac_lang::program::build_source;

use super::support::{evaluate, MAIN, MEDIA};

#[test]
fn authored_item_clocks_bind_evaluate_and_validate() {
    let cases = [
        (
            "pulse(progress)",
            "progress",
            TemporalClock::Progress,
            TemporalValue::Scalar { value: 0.25 },
            TemporalValue::Scalar { value: 0.25 },
        ),
        (
            "pulse(progress)",
            "clip_time / 2s",
            TemporalClock::ClipTime,
            time(1, 1),
            TemporalValue::Scalar { value: 0.5 },
        ),
    ];
    for (old, expression, clock, input, expected) in cases {
        assert_clock(
            &MAIN.replace(old, expression),
            clock,
            input,
            expected,
            false,
        );
    }
}

#[test]
fn authored_sequence_clocks_bind_evaluate_and_validate() {
    let cases = [
        (
            "sequence_time / 2s",
            TemporalClock::SequenceTime,
            time(1, 1),
            TemporalValue::Scalar { value: 0.5 },
        ),
        (
            "if frame < 30 { 0.0 } else { 1.0 }",
            TemporalClock::Frame,
            TemporalValue::Integer { value: 30 },
            TemporalValue::Scalar { value: 1.0 },
        ),
    ];
    for (expression, clock, input, expected) in cases {
        assert_clock(
            &MAIN.replace("pulse(progress)", expression),
            clock,
            input,
            expected,
            true,
        );
    }
}

#[test]
fn authored_source_clock_binds_evaluates_and_validates() {
    let source = format!(
        "animate visual-opacity on clip(@media, @main, @visual, @hero-clip) \
         using resource(@media, @hero) {{ source_time / 3s }}\n{MEDIA}"
    );
    assert_clock(
        &source,
        TemporalClock::SourceTime,
        time(3, 2),
        TemporalValue::Scalar { value: 0.5 },
        false,
    );
}

fn assert_clock(
    source: &str,
    expected_clock: TemporalClock,
    input: TemporalValue,
    expected: TemporalValue,
    sequence_owned: bool,
) {
    let built = build_source(source).unwrap();
    let envelope = built.envelope();
    let sequence = &envelope.project.sequences[0];
    let clip = &sequence.tracks[0].clips[0];
    let binding = &envelope.temporal.bindings[0];
    assert_eq!(binding.clocks.len(), 1);
    assert_eq!(binding.clocks[0].clock, expected_clock);
    let owner = if sequence_owned {
        TemporalClockOwner::Sequence {
            sequence_id: sequence.id.clone(),
        }
    } else {
        TemporalClockOwner::Item {
            item_id: clip.id.clone(),
        }
    };
    assert_eq!(binding.clocks[0].owner, owner);
    assert_eq!(evaluate(envelope, input), expected);
    assert!(veac_ir::validate(envelope).is_ok());
}

fn time(value: i64, scale: u32) -> TemporalValue {
    TemporalValue::Time {
        value: RationalTime::new(value, scale).unwrap(),
    }
}
