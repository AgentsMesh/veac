use veac_ir::{Animatable, EditOperation, ItemId, RationalTime, VisualProperty};

use crate::*;

use super::support::*;

#[test]
fn auto_reframe_maps_a_single_sample_without_losing_its_time_or_identity() {
    let project = project();
    let (request, response) = exchange(Capability::AutoReframe);
    let proposal = propose_edit(&project, &request, &response, &reframe_context()).unwrap();
    assert_eq!(proposal.batch.operations.len(), 1);
    assert!(matches!(
        &proposal.batch.operations[0],
        EditOperation::SetVisualProperty {
            clip_id,
            property: VisualProperty::Crop(Some(Animatable::Keyframes { keyframes })),
        } if clip_id.as_str() == "itm_video"
            && keyframes.len() == 1
            && keyframes[0].value == crate::test_support::rect()
            && keyframes[0].id.as_str() == "kf_reframe_crop_0001"
    ));
    assert!(matches!(
        &proposal.evidence[0],
        ProposalEvidence::CropSamples { operation, sample_indices, .. }
            if operation.operation_index == 0 && sample_indices == &[0]
    ));
    assert!(matches!(
        veac_ir::apply_edit_batch(&project, &proposal.batch),
        veac_ir::EditOutcome::Applied { .. }
    ));
}

#[test]
fn auto_reframe_rejects_empty_inexact_and_out_of_range_samples() {
    let (request, response) = exchange(Capability::AutoReframe);
    let mut empty = response.clone();
    let ProviderOutput::AutoReframe(result) = &mut empty.output else {
        unreachable!()
    };
    result.crops.clear();
    assert_unsupported(propose_edit(
        &project(),
        &request,
        &empty,
        &reframe_context(),
    ));

    let mut inexact = response.clone();
    let ProviderOutput::AutoReframe(result) = &mut inexact.output else {
        unreachable!()
    };
    result.crops[0].time = RationalTime::new(1, 3).unwrap();
    assert_invalid(propose_edit(
        &project(),
        &request,
        &inexact,
        &reframe_context(),
    ));

    let mut outside = response;
    let ProviderOutput::AutoReframe(result) = &mut outside.output else {
        unreachable!()
    };
    result.crops[0].time = time(101);
    assert_invalid(propose_edit(
        &project(),
        &request,
        &outside,
        &reframe_context(),
    ));
}

#[test]
fn auto_reframe_rejects_missing_and_locked_targets() {
    let (request, response) = exchange(Capability::AutoReframe);
    let mut context = reframe_context();
    let ApplicationContext::AutoReframe(value) = &mut context else {
        unreachable!()
    };
    value.clip_id = ItemId::new("itm_missing").unwrap();
    assert_invalid(propose_edit(&project(), &request, &response, &context));

    let mut locked = project();
    locked.project.sequences[0].tracks[3].state.locked = true;
    assert_invalid(propose_edit(
        &locked,
        &request,
        &response,
        &reframe_context(),
    ));
}

fn assert_invalid(value: ProviderResult<ProviderEditProposal>) {
    assert_eq!(value.unwrap_err().kind, ProviderErrorKind::InvalidContract);
}

fn assert_unsupported(value: ProviderResult<ProviderEditProposal>) {
    assert_eq!(
        value.unwrap_err().kind,
        ProviderErrorKind::UnsupportedApplication
    );
}
