use veac_ir::{TemporalClock, TemporalClockOwner};
use veac_lang::program::expression::CoreTemporalInputIdentity;
use veac_lang::program::ClipTemporalProperty;

use super::support::{self, SOLID_SOURCE};

#[test]
fn each_canonical_clock_kind_gets_its_closed_owner_shape() {
    let ids = support::ids(SOLID_SOURCE);
    let cases = [
        (
            "clock / 1s",
            TemporalClock::SequenceTime,
            CoreTemporalInputIdentity::SequenceTime {
                sequence_id: ids.sequence.clone(),
            },
            TemporalClockOwner::Sequence {
                sequence_id: ids.sequence.clone(),
            },
        ),
        (
            "if clock == 0 { 0.0 } else { 1.0 }",
            TemporalClock::Frame,
            CoreTemporalInputIdentity::Frame {
                sequence_id: ids.sequence.clone(),
            },
            TemporalClockOwner::Sequence {
                sequence_id: ids.sequence.clone(),
            },
        ),
        (
            "clock / 1s",
            TemporalClock::ClipTime,
            CoreTemporalInputIdentity::ClipTime {
                item_id: ids.items[0].clone(),
            },
            TemporalClockOwner::Item {
                item_id: ids.items[0].clone(),
            },
        ),
    ];
    for (index, (source, expected_clock, identity, expected_owner)) in cases.into_iter().enumerate()
    {
        let expression = support::compile(source, [("clock", identity)]);
        let leaf = support::leaf(
            ids.items[0].clone(),
            ClipTemporalProperty::VisualOpacity,
            expression,
            &format!("clock_{index}"),
        );
        let envelope = support::execute(SOLID_SOURCE, &[leaf]);
        let binding = &envelope.temporal.bindings[0];
        assert_eq!(binding.clocks[0].clock, expected_clock);
        assert_eq!(binding.clocks[0].owner, expected_owner);
        assert!(veac_ir::validate(&envelope).is_ok());
    }
}
