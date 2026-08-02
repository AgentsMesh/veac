use veac_ir::{apply_edit_batch, ChangedObjectId, EditOperation, EditOutcome, MulticamSyncBasis};

use crate::*;

use super::support::time;

#[path = "multicam_tests/support.rs"]
mod multicam_support;
use multicam_support::fixture;

#[test]
fn audio_and_timecode_sync_produce_reviewable_exact_group_edits() {
    for basis in [MulticamSyncBasis::Audio, MulticamSyncBasis::Timecode] {
        let (project, request, response, context) = fixture(basis);
        let proposal = propose_edit(&project, &request, &response, &context).unwrap();
        assert_eq!(proposal.batch.operations.len(), 1);
        let EditOperation::SetMulticamGroup {
            group_id,
            sync,
            angles,
        } = &proposal.batch.operations[0]
        else {
            unreachable!()
        };
        assert_eq!(group_id.as_str(), "mcg_main");
        assert_eq!(sync.basis, basis);
        assert_eq!(angles[0].source_offset, time(0));
        assert_eq!(angles[1].source_offset, time(10));
        assert!(matches!(
            &proposal.evidence[0],
            ProposalEvidence::MulticamSync { angle_indices, .. }
                if angle_indices == &[0, 1]
        ));
        let EditOutcome::Applied {
            project: updated,
            changed_objects,
            ..
        } = apply_edit_batch(&project, &proposal.batch)
        else {
            panic!("multicam sync proposal must apply")
        };
        assert_eq!(updated.project.multicam_groups[0].sync.basis, basis);
        assert!(changed_objects.iter().any(|value| matches!(
            value,
            ChangedObjectId::MulticamGroup { id } if id.as_str() == "mcg_main"
        )));
    }
}

#[test]
fn sync_rejects_request_result_project_and_limit_mismatches() {
    let (project, request, response, context) = fixture(MulticamSyncBasis::Timecode);
    let mut wrong_result = response.clone();
    let ProviderOutput::MulticamSync(result) = &mut wrong_result.output else {
        unreachable!()
    };
    result.reference_angle_id = veac_ir::MulticamAngleId::new("ang_b").unwrap();
    assert_invalid(propose_edit(&project, &request, &wrong_result, &context));

    let mut outside = response.clone();
    let ProviderOutput::MulticamSync(result) = &mut outside.output else {
        unreachable!()
    };
    result.offsets[1].source_offset = time(101);
    assert_invalid(propose_edit(&project, &request, &outside, &context));

    let mut unbound_request = request.clone();
    let ProviderRequest::MulticamSync(value) = &mut unbound_request.request else {
        unreachable!()
    };
    value.angles[0].media.content = veac_artifact::ContentDigest::sha256(b"other");
    let unbound_output = crate::test_support::output_for(&unbound_request);
    let unbound_response = ProviderResponseEnvelope::new(&unbound_request, unbound_output).unwrap();
    assert_invalid(propose_edit(
        &project,
        &unbound_request,
        &unbound_response,
        &context,
    ));

    let mut absent = context;
    let ApplicationContext::MulticamSync(value) = &mut absent else {
        unreachable!()
    };
    value.group_id = veac_ir::MulticamGroupId::new("mcg_absent").unwrap();
    assert_invalid(propose_edit(&project, &request, &response, &absent));
}

#[test]
fn sync_evidence_detects_source_operation_and_index_tampering() {
    let (project, request, response, context) = fixture(MulticamSyncBasis::Timecode);
    let proposal = propose_edit(&project, &request, &response, &context).unwrap();

    let mut source = proposal.clone();
    let ProviderOutput::MulticamSync(result) = &mut source.source_output else {
        unreachable!()
    };
    result.offsets[1].source_offset = time(11);
    assert!(canonical_edit_proposal_bytes(&source).is_err());

    let mut operation = proposal.clone();
    let EditOperation::SetMulticamGroup { angles, .. } = &mut operation.batch.operations[0] else {
        unreachable!()
    };
    angles[1].source_offset = time(11);
    assert!(canonical_edit_proposal_bytes(&operation).is_err());

    let mut evidence = proposal;
    let ProposalEvidence::MulticamSync { angle_indices, .. } = &mut evidence.evidence[0] else {
        unreachable!()
    };
    angle_indices.swap(0, 1);
    assert!(canonical_edit_proposal_bytes(&evidence).is_err());
}

#[test]
fn sync_target_cannot_be_retargeted_with_rebound_evidence() {
    let (project, request, response, context) = fixture(MulticamSyncBasis::Timecode);
    let mut proposal = propose_edit(&project, &request, &response, &context).unwrap();
    let other = veac_ir::MulticamGroupId::new("mcg_other").unwrap();
    let EditOperation::SetMulticamGroup { group_id, .. } = &mut proposal.batch.operations[0] else {
        unreachable!()
    };
    *group_id = other.clone();
    let ProposalEvidence::MulticamSync {
        operation,
        group_id,
        ..
    } = &mut proposal.evidence[0]
    else {
        unreachable!()
    };
    *group_id = other;
    *operation = OperationBinding::new(0, &proposal.batch.operations[0]).unwrap();
    assert!(canonical_edit_proposal_bytes(&proposal).is_err());
}

fn assert_invalid(value: ProviderResult<ProviderEditProposal>) {
    assert_eq!(value.unwrap_err().kind, ProviderErrorKind::InvalidContract);
}
