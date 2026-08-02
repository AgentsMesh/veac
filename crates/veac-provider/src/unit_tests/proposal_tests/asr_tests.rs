use veac_ir::{AppliedOperation, ClipSource, EditOperation, EditOutcome, ItemId, TrackKind};

use crate::*;

use super::support::*;

#[test]
fn asr_builds_a_deterministic_reviewable_atomic_batch() {
    let project = project();
    let (request, mut response) = exchange(Capability::Asr);
    let ProviderOutput::Asr(result) = &mut response.output else {
        unreachable!()
    };
    let mut second = result.segments[0].clone();
    second.id = "segment-2".into();
    second.range = crate::test_support::range(30, 10);
    second.text = "world".into();
    second.words.clear();
    result.segments.push(second);

    let proposal = propose_edit(&project, &request, &response, &asr_context()).unwrap();
    assert_eq!(proposal.project_revision, 7);
    assert_eq!(proposal.request_hash, request_hash(&request).unwrap());
    assert_eq!(proposal.response_hash, response_hash(&response).unwrap());
    assert_eq!(proposal.batch.operations.len(), 2);
    assert!(proposal.batch.atomic);
    assert!(matches!(
        &proposal.evidence[1],
        ProposalEvidence::AsrSegment { operation, segment_id }
            if operation.operation_index == 1 && segment_id == "segment-2"
    ));
    let EditOperation::InsertClip {
        clip, before_id, ..
    } = &proposal.batch.operations[0]
    else {
        unreachable!()
    };
    assert_eq!(clip.id.as_str(), "itm_provider_0001");
    assert!(matches!(clip.source, ClipSource::Caption { .. }));
    assert_eq!(before_id.as_ref().map(ItemId::as_str), Some("itm_existing"));
    let EditOutcome::Applied {
        project: applied, ..
    } = veac_ir::apply_edit_batch(&project, &proposal.batch)
    else {
        unreachable!()
    };
    assert_eq!(applied.project.sequences[0].tracks[1].clips.len(), 3);
    let bytes = canonical_edit_proposal_bytes(&proposal).unwrap();
    let decoded: ProviderEditProposal = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(canonical_edit_proposal_bytes(&decoded).unwrap(), bytes);
}

#[test]
fn asr_rejects_stale_or_invalid_targets_and_generated_ids() {
    let (request, response) = exchange(Capability::Asr);
    let mut context = asr_context();
    header_mut(&mut context).project_revision += 1;
    assert_invalid(propose_edit(&project(), &request, &response, &context));

    let mut value = project();
    value.project.sequences[0].tracks[1].kind = TrackKind::Visual;
    assert_invalid(propose_edit(&value, &request, &response, &asr_context()));

    let mut value = project();
    value.project.sequences[0].tracks[1].clips[0].id = ItemId::new("itm_provider_0001").unwrap();
    assert_invalid(propose_edit(&value, &request, &response, &asr_context()));

    let mut context = asr_context();
    if let ApplicationContext::AsrCaptions(value) = &mut context {
        value.item_id_prefix = "caption_".into();
    }
    assert_invalid(propose_edit(&project(), &request, &response, &context));
    if let ApplicationContext::AsrCaptions(value) = &mut context {
        value.item_id_prefix = "itm_provider_".into();
        value.track_id = veac_ir::TrackId::new("trk_missing").unwrap();
    }
    assert_invalid(propose_edit(&project(), &request, &response, &context));
}

#[test]
fn asr_rejects_empty_output_and_replayed_operation_id() {
    let (request, mut response) = exchange(Capability::Asr);
    let ProviderOutput::Asr(result) = &mut response.output else {
        unreachable!()
    };
    result.segments.clear();
    assert_invalid(propose_edit(
        &project(),
        &request,
        &response,
        &asr_context(),
    ));

    let (request, response) = exchange(Capability::Asr);
    let mut value = project();
    value.project.applied_operations.push(AppliedOperation {
        id: asr_context().header().operation_id.clone(),
        request_hash: "0".repeat(64),
    });
    let error = propose_edit(&value, &request, &response, &asr_context()).unwrap_err();
    assert!(error.to_string().contains("not applicable"));
}

fn assert_invalid(value: ProviderResult<ProviderEditProposal>) {
    assert_eq!(value.unwrap_err().kind, ProviderErrorKind::InvalidContract);
}
