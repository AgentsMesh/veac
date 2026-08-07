use veac_ir::{ItemId, SequenceId, TemporalParameterId, TemporalType, TemporalValue};
use veac_lang::program::expression::CoreTemporalInputIdentity;
use veac_lang::program::ClipTemporalProperty;

use super::support::{self, SOLID_SOURCE};

fn reason(leaves: &[veac_lang::program::ExecutableTemporalLeaf], expected: &str) {
    let diagnostic = support::failure(SOLID_SOURCE, leaves);
    assert_eq!(diagnostic.code, "PROGRAM_EXECUTABLE_LOWER");
    assert!(
        diagnostic.message.contains(expected),
        "{}",
        diagnostic.message
    );
}

fn progress(item: ItemId, name: &str) -> veac_lang::program::ExecutableTemporalLeaf {
    let expression = support::compile(
        "progress",
        [(
            "progress",
            CoreTemporalInputIdentity::Progress {
                item_id: item.clone(),
            },
        )],
    );
    support::leaf(item, ClipTemporalProperty::VisualOpacity, expression, name)
}

#[test]
fn duplicate_sink_is_rejected_before_residual_publication() {
    let ids = support::ids(SOLID_SOURCE);
    reason(
        &[
            progress(ids.items[0].clone(), "duplicate_a"),
            progress(ids.items[0].clone(), "duplicate_b"),
        ],
        "EXECUTABLE_TEMPORAL_SINK_DUPLICATE",
    );
}

#[test]
fn missing_sink_and_wrong_item_owner_fail_closed() {
    let ids = support::ids(SOLID_SOURCE);
    let missing = ItemId::new("itm_missing").unwrap();
    reason(
        &[progress(missing, "missing")],
        "EXECUTABLE_TEMPORAL_SINK_MISSING",
    );
    let expression = support::compile(
        "progress",
        [(
            "progress",
            CoreTemporalInputIdentity::Progress {
                item_id: ids.items[1].clone(),
            },
        )],
    );
    let leaf = support::leaf(
        ids.items[0].clone(),
        ClipTemporalProperty::VisualOpacity,
        expression,
        "wrong_item",
    );
    reason(&[leaf], "EXECUTABLE_TEMPORAL_CLOCK_OWNER");
}

#[test]
fn wrong_sequence_owner_is_rejected_for_sequence_and_frame_clocks() {
    let ids = support::ids(SOLID_SOURCE);
    let wrong = SequenceId::new("seq_wrong").unwrap();
    for (index, (source, identity)) in [
        (
            "clock / 1s",
            CoreTemporalInputIdentity::SequenceTime {
                sequence_id: wrong.clone(),
            },
        ),
        (
            "if clock == 0 { 0.0 } else { 1.0 }",
            CoreTemporalInputIdentity::Frame {
                sequence_id: wrong.clone(),
            },
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let expression = support::compile(source, [("clock", identity)]);
        let leaf = support::leaf(
            ids.items[0].clone(),
            ClipTemporalProperty::VisualOpacity,
            expression,
            &format!("wrong_seq_{index}"),
        );
        reason(&[leaf], "EXECUTABLE_TEMPORAL_CLOCK_OWNER");
    }
}

#[test]
fn concrete_and_wrong_typed_results_cannot_reach_a_temporal_sink() {
    let ids = support::ids(SOLID_SOURCE);
    let concrete = support::leaf(
        ids.items[0].clone(),
        ClipTemporalProperty::VisualOpacity,
        support::compile("0.5", []),
        "concrete",
    );
    reason(&[concrete], "EXECUTABLE_TEMPORAL_CONCRETE");
    let typed = progress(ids.items[0].clone(), "wrong_type");
    let expression = typed.expression().clone();
    let leaf = support::leaf(
        ids.items[0].clone(),
        ClipTemporalProperty::VisualScale,
        expression,
        "wrong_type_scale",
    );
    reason(&[leaf], "EXECUTABLE_TEMPORAL_SINK_TYPE");
}

#[test]
fn parameter_values_must_be_complete_exact_and_declared() {
    let ids = support::ids(SOLID_SOURCE);
    let parameter = TemporalParameterId::new("tpm_required").unwrap();
    let expression = support::compile(
        "gain",
        [(
            "gain",
            CoreTemporalInputIdentity::Parameter {
                parameter_id: parameter.clone(),
                value_type: TemporalType::Scalar,
            },
        )],
    );
    let missing = support::leaf(
        ids.items[0].clone(),
        ClipTemporalProperty::VisualOpacity,
        expression.clone(),
        "parameter_missing",
    );
    reason(&[missing], "EXECUTABLE_TEMPORAL_PARAMETER_MISSING");
    let wrong = support::leaf(
        ids.items[0].clone(),
        ClipTemporalProperty::VisualOpacity,
        expression,
        "parameter_wrong",
    )
    .bind_parameter(parameter, TemporalValue::Integer { value: 1 });
    reason(&[wrong], "EXECUTABLE_TEMPORAL_PARAMETER_TYPE");

    let extra_id = TemporalParameterId::new("tpm_extra").unwrap();
    let extra = progress(ids.items[0].clone(), "parameter_extra")
        .bind_parameter(extra_id, TemporalValue::Scalar { value: 0.5 });
    reason(&[extra], "EXECUTABLE_TEMPORAL_PARAMETER_UNUSED");
}

#[test]
fn a_visual_clip_does_not_implicitly_gain_an_audio_temporal_owner() {
    let ids = support::ids(SOLID_SOURCE);
    let expression = support::compile(
        "progress",
        [(
            "progress",
            CoreTemporalInputIdentity::Progress {
                item_id: ids.items[0].clone(),
            },
        )],
    );
    let leaf = support::leaf(
        ids.items[0].clone(),
        ClipTemporalProperty::AudioGain,
        expression,
        "missing_audio",
    );
    reason(&[leaf], "EXECUTABLE_TEMPORAL_AUDIO_SINK");
}
