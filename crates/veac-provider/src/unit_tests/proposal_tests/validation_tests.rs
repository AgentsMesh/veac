use veac_ir::{EditOperation, TrackId};

use crate::*;

use super::support::*;

#[test]
fn all_application_contexts_use_strict_tagged_canonical_json() {
    let (_, tts) = exchange(Capability::TextToSpeech);
    let ProviderOutput::TextToSpeech(tts) = &tts.output else {
        unreachable!()
    };
    let (_, dubbing) = exchange(Capability::Dubbing);
    let ProviderOutput::Dubbing(dubbing) = &dubbing.output else {
        unreachable!()
    };
    let (_, denoise) = exchange(Capability::Denoise);
    let ProviderOutput::Denoise(denoise) = &denoise.output else {
        unreachable!()
    };
    let (_, separation) = exchange(Capability::VocalSeparation);
    let ProviderOutput::VocalSeparation(separation) = &separation.output else {
        unreachable!()
    };
    let (_, removal) = exchange(Capability::Removal);
    let ProviderOutput::Removal(removal) = &removal.output else {
        unreachable!()
    };
    let (_, segmentation) = exchange(Capability::Segmentation);
    let ProviderOutput::Segmentation(segmentation) = &segmentation.output else {
        unreachable!()
    };
    let (_, matte) = exchange(Capability::Matte);
    let ProviderOutput::Matte(matte) = &matte.output else {
        unreachable!()
    };
    let (_, retouch) = exchange(Capability::Retouch);
    let ProviderOutput::Retouch(retouch) = &retouch.output else {
        unreachable!()
    };
    let contexts = vec![
        asr_context(),
        translation_context(),
        tts_context(&tts.audio),
        dubbing_context(&dubbing.audio),
        transform_context(Capability::MotionTracking),
        transform_context(Capability::Stabilization),
        matte_context(Capability::Segmentation, &segmentation.matte),
        matte_context(Capability::Matte, &matte.matte),
        denoise_context(&denoise.audio),
        separation_context(&separation.stems[0]),
        reframe_context(),
        removal_context(&removal.video),
        retouch_context(retouch),
        color_context(),
    ];
    for context in contexts {
        let bytes = serde_json_canonicalizer::to_vec(&context).unwrap();
        let decoded: ApplicationContext = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded, context);
        let mut value = serde_json::to_value(&context).unwrap();
        value["parameters"]["unknown"] = serde_json::json!(true);
        assert!(serde_json::from_value::<ApplicationContext>(value).is_err());
    }
    let context_schema = serde_json::to_value(schemars::schema_for!(ApplicationContext)).unwrap();
    assert!(context_schema.to_string().contains("asr_captions"));
    let proposal_schema = provider_edit_proposal_json_schema().unwrap();
    assert!(proposal_schema.to_string().contains("operation_hash"));
}

#[test]
fn proposal_validation_rejects_evidence_index_hash_and_operation_tampering() {
    let proposal = asr_proposal();
    let mut bad_index = proposal.clone();
    let ProposalEvidence::AsrSegment { operation, .. } = &mut bad_index.evidence[0] else {
        unreachable!()
    };
    operation.operation_index = 2;
    assert!(canonical_edit_proposal_bytes(&bad_index).is_err());

    let mut bad_hash = proposal.clone();
    let ProposalEvidence::AsrSegment { operation, .. } = &mut bad_hash.evidence[0] else {
        unreachable!()
    };
    operation.operation_hash.value.clear();
    assert!(canonical_edit_proposal_bytes(&bad_hash).is_err());

    let mut bad_target = proposal;
    let EditOperation::InsertClip { track_id, .. } = &mut bad_target.batch.operations[0] else {
        unreachable!()
    };
    *track_id = TrackId::new("trk_visual").unwrap();
    assert!(canonical_edit_proposal_bytes(&bad_target).is_err());
}

#[test]
fn proposal_validation_rejects_source_and_revision_tampering() {
    let proposal = asr_proposal();
    let mut source = proposal.clone();
    let ProviderOutput::Asr(result) = &mut source.source_output else {
        unreachable!()
    };
    result.segments[0].text = "tampered".into();
    assert!(canonical_edit_proposal_bytes(&source).is_err());

    let mut revision = proposal.clone();
    revision.batch.base_revision += 1;
    assert!(canonical_edit_proposal_bytes(&revision).is_err());

    let mut unsafe_revision = proposal;
    unsafe_revision.project_revision = veac_ir::MAX_SAFE_INTEGER + 1;
    unsafe_revision.batch.base_revision = veac_ir::MAX_SAFE_INTEGER + 1;
    header_mut(&mut unsafe_revision.application_context).project_revision =
        veac_ir::MAX_SAFE_INTEGER + 1;
    let error = canonical_edit_proposal_bytes(&unsafe_revision).unwrap_err();
    assert_eq!(error.kind, ProviderErrorKind::InvalidContract);
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn proposal_build_rejects_capability_revision_and_deserialized_id_mismatch() {
    let (request, response) = exchange(Capability::Translation);
    let error = propose_edit(&project(), &request, &response, &asr_context()).unwrap_err();
    assert_eq!(error.kind, ProviderErrorKind::InvalidContract);

    let (request, response) = exchange(Capability::Asr);
    let mut stale = asr_context();
    header_mut(&mut stale).project_revision = 8;
    assert_eq!(
        propose_edit(&project(), &request, &response, &stale)
            .unwrap_err()
            .kind,
        ProviderErrorKind::InvalidContract
    );

    let mut value = serde_json::to_value(asr_context()).unwrap();
    value["parameters"]["header"]["operation_id"] = serde_json::json!("invalid");
    let invalid: ApplicationContext = serde_json::from_value(value).unwrap();
    assert_eq!(
        propose_edit(&project(), &request, &response, &invalid)
            .unwrap_err()
            .kind,
        ProviderErrorKind::InvalidContract
    );
}

#[test]
fn proposal_json_rejects_unknown_fields() {
    let proposal = asr_proposal();
    let mut value = serde_json::to_value(proposal).unwrap();
    value["unknown"] = serde_json::json!(true);
    assert!(serde_json::from_value::<ProviderEditProposal>(value).is_err());
}

fn asr_proposal() -> ProviderEditProposal {
    let (request, response) = exchange(Capability::Asr);
    propose_edit(&project(), &request, &response, &asr_context()).unwrap()
}
