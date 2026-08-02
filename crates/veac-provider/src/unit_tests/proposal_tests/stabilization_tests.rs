use veac_ir::{Animatable, EditOperation, VisualProperty};

use crate::*;

use super::support::*;

#[test]
fn stabilization_combines_transform_and_crop_curves_in_one_atomic_batch() {
    let project = project();
    let (request, mut response) = exchange(Capability::Stabilization);
    let ProviderOutput::Stabilization(result) = &mut response.output else {
        unreachable!()
    };
    let mut second = result.crops[0].clone();
    second.time = time(1);
    second.rect.x = 0.2;
    second.rect.width = 0.4;
    result.crops.push(second);
    let proposal = propose_edit(
        &project,
        &request,
        &response,
        &transform_context(Capability::Stabilization),
    )
    .unwrap();
    assert!(proposal.batch.atomic);
    assert_eq!(proposal.batch.operations.len(), 4);
    assert!(matches!(
        &proposal.batch.operations[3],
        EditOperation::SetVisualProperty {
            property: VisualProperty::Crop(Some(Animatable::Keyframes { keyframes })),
            ..
        } if keyframes.len() == 2
            && keyframes[0].value.x == 0.1
            && keyframes[1].value.x == 0.2
            && keyframes[1].value.width == 0.4
    ));
    assert!(matches!(
        &proposal.evidence[3],
        ProposalEvidence::CropSamples { operation, sample_indices, .. }
            if operation.operation_index == 3 && sample_indices == &[0, 1]
    ));
    assert!(matches!(
        veac_ir::apply_edit_batch(&project, &proposal.batch),
        veac_ir::EditOutcome::Applied { .. }
    ));
}

#[test]
fn stabilization_rejects_empty_results_and_source_identity_mismatch() {
    let (request, mut response) = exchange(Capability::Stabilization);
    let ProviderOutput::Stabilization(result) = &mut response.output else {
        unreachable!()
    };
    result.transforms.clear();
    result.crops.clear();
    assert_unsupported(propose_edit(
        &project(),
        &request,
        &response,
        &transform_context(Capability::Stabilization),
    ));

    let (mut request, response) = exchange(Capability::Stabilization);
    let ProviderRequest::Stabilization(value) = &mut request.request else {
        unreachable!()
    };
    value.video.content = veac_artifact::ContentDigest::sha256(b"wrong-video");
    let response = ProviderResponseEnvelope::new(&request, response.output).unwrap();
    assert_invalid(propose_edit(
        &project(),
        &request,
        &response,
        &transform_context(Capability::Stabilization),
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
