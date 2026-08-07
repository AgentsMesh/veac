use super::*;
use crate::program::executable::{ExecutableTemporalSink as Sink, MaskTemporalProperty};
use veac_ir::{
    ApplyId, ApplyOperation, ColorMatrix, ColorPipeline, ColorPrimaries, ColorRange, ColorSpace,
    ColorTransfer, EffectId, EffectParameter, TemporalBindingId,
};

mod clip;

#[test]
fn apply_identity_lookup_fails_closed_at_each_owner_level() {
    let project = fixture();
    let (sequence_id, apply_id, _, _) = apply_ids(&project);

    let mut missing_sequence = project.clone();
    let sink = Sink::ApplyOpacity {
        sequence_id: veac_ir::SequenceId::new("seq_missing").unwrap(),
        apply_id: apply_id.clone(),
    };
    assert_reason(
        sink::attach(&mut missing_sequence, &sink, binding()).unwrap_err(),
        "EXECUTABLE_TEMPORAL_SINK_MISSING",
    );

    let mut missing_apply = project;
    let sink = Sink::ApplyOpacity {
        sequence_id,
        apply_id: ApplyId::new("apl_missing").unwrap(),
    };
    assert_reason(
        sink::attach(&mut missing_apply, &sink, binding()).unwrap_err(),
        "EXECUTABLE_TEMPORAL_SINK_MISSING",
    );
}

#[test]
fn apply_masks_effect_stage_kinds_and_effect_ids_are_closed() {
    let project = fixture();
    let (sequence_id, apply_id, stage_id, effect_id) = apply_ids(&project);

    let mut missing_mask = project.clone();
    let sink = Sink::ApplyMask {
        sequence_id: sequence_id.clone(),
        apply_id: apply_id.clone(),
        mask_index: u32::MAX,
        property: MaskTemporalProperty::Position,
    };
    assert_reason(
        sink::attach(&mut missing_mask, &sink, binding()).unwrap_err(),
        "EXECUTABLE_TEMPORAL_OPTIONAL_SINK",
    );

    let mut wrong_stage = project.clone();
    wrong_stage.sequences[0].applies[0].stages[0].operation = ApplyOperation::Color {
        pipeline: empty_pipeline(),
    };
    let sink = Sink::ApplyEffect {
        sequence_id: sequence_id.clone(),
        apply_id: apply_id.clone(),
        stage_id: stage_id.clone(),
        effect_id: effect_id.clone(),
        parameter: EffectParameter::Radius,
    };
    assert_reason(
        sink::attach(&mut wrong_stage, &sink, binding()).unwrap_err(),
        "EXECUTABLE_TEMPORAL_SINK_OWNER",
    );

    let mut wrong_effect = project;
    let sink = Sink::ApplyEffect {
        sequence_id,
        apply_id,
        stage_id,
        effect_id: EffectId::new("fx_missing").unwrap(),
        parameter: EffectParameter::Radius,
    };
    assert_reason(
        sink::attach(&mut wrong_effect, &sink, binding()).unwrap_err(),
        "EXECUTABLE_TEMPORAL_SINK_MISSING",
    );
}

pub(super) fn fixture() -> veac_ir::Project {
    let source = include_str!("../../../../../tests/fixtures/authored_sink_matrix.veac");
    crate::program::build_source(source)
        .unwrap()
        .envelope()
        .project
        .clone()
}

fn apply_ids(
    project: &veac_ir::Project,
) -> (
    veac_ir::SequenceId,
    ApplyId,
    veac_ir::ApplyStageId,
    EffectId,
) {
    let sequence = &project.sequences[0];
    let apply = &sequence.applies[0];
    let stage = &apply.stages[0];
    let ApplyOperation::Effect { effect } = &stage.operation else {
        unreachable!()
    };
    (
        sequence.id.clone(),
        apply.id.clone(),
        stage.id.clone(),
        effect.id.clone(),
    )
}

pub(super) fn binding() -> TemporalBindingId {
    TemporalBindingId::new("tbd_defensive_sink").unwrap()
}

pub(super) fn assert_reason(error: ExecutableLowerError, expected: &str) {
    assert_eq!(error.reason_code(), expected, "{error}");
}

fn empty_pipeline() -> ColorPipeline {
    let space = ColorSpace {
        primaries: ColorPrimaries::Bt709,
        transfer: ColorTransfer::Bt709,
        matrix: ColorMatrix::Bt709,
        range: ColorRange::Limited,
    };
    ColorPipeline {
        input: space,
        working: space,
        output: space,
        stages: Vec::new(),
    }
}
