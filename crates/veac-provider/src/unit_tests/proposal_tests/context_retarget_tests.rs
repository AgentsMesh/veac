use veac_ir::{
    ApplyTarget, EditOperation, ItemId, RelationEndpoint, RelationKind, SequenceId, StructureEdit,
    TrackId,
};

use crate::*;

use super::support::*;

#[test]
fn text_operations_cannot_be_retargeted_after_hash_rebinding() {
    let (request, response) = exchange(Capability::Asr);
    let mut asr = propose_edit(&project(), &request, &response, &asr_context()).unwrap();
    let EditOperation::InsertClip { sequence_id, .. } = &mut asr.batch.operations[0] else {
        unreachable!()
    };
    *sequence_id = SequenceId::new("seq_other").unwrap();
    assert_rebound_invalid(asr, 0);

    let (request, response) = exchange(Capability::Translation);
    let mut translation =
        propose_edit(&project(), &request, &response, &translation_context()).unwrap();
    let EditOperation::SetText { clip_id, .. } = &mut translation.batch.operations[0] else {
        unreachable!()
    };
    *clip_id = ItemId::new("itm_other").unwrap();
    assert_rebound_invalid(translation, 0);
}

#[test]
fn generated_and_replaced_media_cannot_be_retargeted() {
    let (request, response) = exchange(Capability::TextToSpeech);
    let ProviderOutput::TextToSpeech(result) = &response.output else {
        unreachable!()
    };
    let mut generated =
        propose_edit(&project(), &request, &response, &tts_context(&result.audio)).unwrap();
    let EditOperation::InsertClip { track_id, .. } = &mut generated.batch.operations[1] else {
        unreachable!()
    };
    *track_id = TrackId::new("trk_other").unwrap();
    assert_rebound_invalid(generated, 1);

    let (request, response) = exchange(Capability::Denoise);
    let ProviderOutput::Denoise(result) = &response.output else {
        unreachable!()
    };
    let mut replaced = propose_edit(
        &project(),
        &request,
        &response,
        &denoise_context(&result.audio),
    )
    .unwrap();
    let EditOperation::ReplaceSource { clip_id, .. } = &mut replaced.batch.operations[1] else {
        unreachable!()
    };
    *clip_id = ItemId::new("itm_other").unwrap();
    assert_rebound_invalid(replaced, 1);
}

#[test]
fn visual_operations_cannot_be_retargeted_after_hash_rebinding() {
    let (request, response) = exchange(Capability::MotionTracking);
    let mut transform = propose_edit(
        &project(),
        &request,
        &response,
        &transform_context(Capability::MotionTracking),
    )
    .unwrap();
    retarget_visual(&mut transform.batch.operations[0]);
    assert_rebound_invalid(transform, 0);

    let (request, response) = exchange(Capability::AutoReframe);
    let mut crop = propose_edit(&project(), &request, &response, &reframe_context()).unwrap();
    retarget_visual(&mut crop.batch.operations[0]);
    assert_rebound_invalid(crop, 0);

    let (request, response) = exchange(Capability::ColorMatch);
    let mut color = propose_edit(&project(), &request, &response, &color_context()).unwrap();
    retarget_visual(&mut color.batch.operations[0]);
    assert_rebound_invalid(color, 0);
}

#[test]
fn matte_target_cannot_be_retargeted_after_hash_rebinding() {
    let (request, response) = exchange(Capability::Matte);
    let ProviderOutput::Matte(result) = &response.output else {
        unreachable!()
    };
    let mut proposal = propose_edit(
        &project(),
        &request,
        &response,
        &matte_context(Capability::Matte, &result.matte),
    )
    .unwrap();
    retarget_matte(&mut proposal.batch.operations[2]);
    assert_rebound_invalid(proposal, 2);
}

#[test]
fn retouch_apply_cannot_be_retargeted_after_hash_rebinding() {
    let (request, response) = exchange(Capability::Retouch);
    let ProviderOutput::Retouch(result) = &response.output else {
        unreachable!()
    };
    let mut proposal =
        propose_edit(&project(), &request, &response, &retouch_context(result)).unwrap();
    let EditOperation::EditStructure {
        edit: StructureEdit::InsertApply { apply, .. },
    } = &mut proposal.batch.operations[2]
    else {
        unreachable!()
    };
    apply.target = ApplyTarget::ItemSet {
        item_ids: vec![ItemId::new("itm_other").unwrap()],
    };
    assert_rebound_invalid(proposal, 2);
}

fn retarget_visual(operation: &mut EditOperation) {
    let EditOperation::SetVisualProperty { clip_id, .. } = operation else {
        unreachable!()
    };
    *clip_id = ItemId::new("itm_other").unwrap();
}

fn retarget_matte(operation: &mut EditOperation) {
    let EditOperation::EditStructure {
        edit: StructureEdit::InsertRelation { relation },
    } = operation
    else {
        unreachable!()
    };
    let RelationKind::Matte { consumer, .. } = &mut relation.kind else {
        unreachable!()
    };
    *consumer = RelationEndpoint::item(ItemId::new("itm_other").unwrap());
}

fn assert_rebound_invalid(mut proposal: ProviderEditProposal, index: usize) {
    let binding = OperationBinding::new(index as u32, &proposal.batch.operations[index]).unwrap();
    binding_mut(&mut proposal.evidence[index]).clone_from(&binding);
    assert_eq!(
        canonical_edit_proposal_bytes(&proposal).unwrap_err().kind,
        ProviderErrorKind::InvalidContract
    );
}

fn binding_mut(value: &mut ProposalEvidence) -> &mut OperationBinding {
    match value {
        ProposalEvidence::AsrSegment { operation, .. }
        | ProposalEvidence::TranslationUnit { operation, .. }
        | ProposalEvidence::GeneratedAudioClip { operation, .. }
        | ProposalEvidence::ReplacedMedia { operation, .. }
        | ProposalEvidence::TransformSamples { operation, .. }
        | ProposalEvidence::CropSamples { operation, .. }
        | ProposalEvidence::ColorPipeline { operation }
        | ProposalEvidence::AppliedTrackMatte { operation, .. }
        | ProposalEvidence::RetouchApply { operation, .. } => operation,
        _ => unreachable!(),
    }
}
