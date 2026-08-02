use veac_ir::{ColorStage, EditOperation, VisualProperty};

use crate::*;

use super::support::*;

#[test]
fn color_match_builds_a_pipeline_stage_and_typed_effect() {
    let project = project();
    let (request, response) = exchange(Capability::ColorMatch);
    let proposal = propose_edit(&project, &request, &response, &color_context()).unwrap();
    assert_eq!(proposal.batch.operations.len(), 2);
    assert!(matches!(
        &proposal.batch.operations[0],
        EditOperation::SetVisualProperty {
            property: VisualProperty::ColorPipeline(Some(pipeline)), ..
        } if matches!(pipeline.stages.as_slice(), [ColorStage::Matrix { .. }, ColorStage::Basic { .. }])
    ));
    assert!(matches!(
        &proposal.batch.operations[1],
        EditOperation::AddEffect { effect, .. }
            if effect.id.as_str() == "fx_provider_color"
                && effect.effect_type == "video.color_adjust"
    ));
    assert!(matches!(
        &proposal.evidence[1],
        ProposalEvidence::ColorAdjustEffect { operation }
            if operation.operation_index == 1
    ));
    assert!(matches!(
        veac_ir::apply_edit_batch(&project, &proposal.batch),
        veac_ir::EditOutcome::Applied { .. }
    ));
}

#[test]
fn color_match_applies_general_matrix_and_rejects_invalid_ranges() {
    let (request, response) = exchange(Capability::ColorMatch);
    let mut matrix = response.clone();
    let ProviderOutput::ColorMatch(result) = &mut matrix.output else {
        unreachable!()
    };
    result.adjustment.matrix[0] = 0.9;
    result.adjustment.offset[0] = 0.1;
    let proposal = propose_edit(&project(), &request, &matrix, &color_context()).unwrap();
    assert!(matches!(
        &proposal.batch.operations[0],
        EditOperation::SetVisualProperty {
            property: VisualProperty::ColorPipeline(Some(pipeline)), ..
        } if matches!(
            pipeline.stages.first(),
            Some(ColorStage::Matrix { adjustment })
                if adjustment.matrix[0] == 0.9 && adjustment.offset[0] == 0.1
        )
    ));

    let mut coefficient = response.clone();
    let ProviderOutput::ColorMatch(result) = &mut coefficient.output else {
        unreachable!()
    };
    result.adjustment.matrix[0] = 17.0;
    assert_invalid(propose_edit(
        &project(),
        &request,
        &coefficient,
        &color_context(),
    ));
    let mut offset = response.clone();
    let ProviderOutput::ColorMatch(result) = &mut offset.output else {
        unreachable!()
    };
    result.adjustment.offset[0] = 5.0;
    assert_invalid(propose_edit(
        &project(),
        &request,
        &offset,
        &color_context(),
    ));
    let mut range = response.clone();
    let ProviderOutput::ColorMatch(result) = &mut range.output else {
        unreachable!()
    };
    result.adjustment.contrast = 5.0;
    assert_invalid(propose_edit(&project(), &request, &range, &color_context()));
}

#[test]
fn color_pipeline_evidence_rejects_rehashed_matrix_tampering() {
    let (request, mut response) = exchange(Capability::ColorMatch);
    let ProviderOutput::ColorMatch(result) = &mut response.output else {
        unreachable!()
    };
    result.adjustment.matrix[1] = 0.2;
    let mut proposal = propose_edit(&project(), &request, &response, &color_context()).unwrap();
    let EditOperation::SetVisualProperty {
        property: VisualProperty::ColorPipeline(Some(pipeline)),
        ..
    } = &mut proposal.batch.operations[0]
    else {
        unreachable!()
    };
    let ColorStage::Matrix { adjustment } = &mut pipeline.stages[0] else {
        unreachable!()
    };
    adjustment.matrix[1] = 0.3;
    let ProposalEvidence::ColorPipeline { operation } = &mut proposal.evidence[0] else {
        unreachable!()
    };
    *operation = OperationBinding::new(0, &proposal.batch.operations[0]).unwrap();
    assert!(canonical_edit_proposal_bytes(&proposal).is_err());
}

#[test]
fn color_match_rejects_locked_targets_and_duplicate_effect_ids_atomically() {
    let (request, response) = exchange(Capability::ColorMatch);
    let mut locked = project();
    locked.project.sequences[0].tracks[0].state.locked = true;
    assert_invalid(propose_edit(&locked, &request, &response, &color_context()));

    let mut duplicate = project();
    let proposal = propose_edit(&duplicate, &request, &response, &color_context()).unwrap();
    let EditOperation::AddEffect { effect, .. } = &proposal.batch.operations[1] else {
        unreachable!()
    };
    duplicate.project.sequences[0].tracks[0].clips[1]
        .effects
        .push(effect.clone());
    let error = propose_edit(&duplicate, &request, &response, &color_context()).unwrap_err();
    assert_eq!(error.kind, ProviderErrorKind::InvalidContract);
    assert!(error.to_string().contains("not applicable"));
}

fn assert_invalid(value: ProviderResult<ProviderEditProposal>) {
    assert_eq!(value.unwrap_err().kind, ProviderErrorKind::InvalidContract);
}
