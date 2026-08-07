use veac_artifact::{
    artifact_key, ArtifactParameters, ContentDigest, ProducedArtifactParameters,
    ProviderResultParameters,
};
use veac_ir::{ItemId, MaterialId, MaterialKind, MaterialSource};

use crate::*;

use super::support::*;

#[test]
fn material_application_rejects_wrong_artifact_kind_identity_kind_and_uri() {
    let (request, mut wrong_artifact) = exchange(Capability::TextToSpeech);
    let artifact = match &mut wrong_artifact.output {
        ProviderOutput::TextToSpeech(result) => {
            result.audio.descriptor.parameters =
                ArtifactParameters::AudioStem(ProducedArtifactParameters::Provider(
                    ProviderResultParameters::new("wrong-audio-stem").unwrap(),
                ));
            result.audio.record.key = artifact_key(&result.audio.descriptor).unwrap();
            result.audio.clone()
        }
        _ => unreachable!(),
    };
    assert_invalid(propose_edit(
        &project(),
        &request,
        &wrong_artifact,
        &tts_context(&artifact),
    ));

    let (request, response) = exchange(Capability::TextToSpeech);
    let ProviderOutput::TextToSpeech(result) = &response.output else {
        unreachable!()
    };
    let mut identity = tts_context(&result.audio);
    let output = tts_output(&mut identity);
    output.material.material.identity.as_mut().unwrap().digest = "a".repeat(64);
    output
        .material
        .material
        .probe
        .as_mut()
        .unwrap()
        .observed_identity
        .digest = "a".repeat(64);
    assert_invalid(propose_edit(&project(), &request, &response, &identity));

    let mut kind = tts_context(&result.audio);
    tts_output(&mut kind).material.material.kind = MaterialKind::Video;
    assert_invalid(propose_edit(&project(), &request, &response, &kind));

    let mut uri = tts_context(&result.audio);
    tts_output(&mut uri).material.material.source = MaterialSource::File {
        uri: "/tmp/provider.wav".into(),
    };
    assert_invalid(propose_edit(&project(), &request, &response, &uri));
}

#[test]
fn material_application_rejects_duplicate_ids_anchors_and_source_time() {
    let (request, response) = exchange(Capability::TextToSpeech);
    let ProviderOutput::TextToSpeech(result) = &response.output else {
        unreachable!()
    };
    let mut material_id = tts_context(&result.audio);
    tts_output(&mut material_id).material.material.id =
        MaterialId::new("med_source_audio").unwrap();
    assert_invalid(propose_edit(&project(), &request, &response, &material_id));

    let mut clip_id = tts_context(&result.audio);
    tts_output(&mut clip_id).clip_id = ItemId::new("itm_audio").unwrap();
    assert_invalid(propose_edit(&project(), &request, &response, &clip_id));

    let mut anchors = tts_context(&result.audio);
    let output = tts_output(&mut anchors);
    output.before_id = Some(ItemId::new("itm_audio").unwrap());
    output.after_id = Some(ItemId::new("itm_audio").unwrap());
    assert_invalid(propose_edit(&project(), &request, &response, &anchors));

    let mut source_time = tts_context(&result.audio);
    tts_output(&mut source_time).source_start = time(1);
    assert_invalid(propose_edit(&project(), &request, &response, &source_time));
}

#[test]
fn artifact_evidence_rejects_key_role_and_operation_tampering() {
    let (request, response) = exchange(Capability::TextToSpeech);
    let ProviderOutput::TextToSpeech(result) = &response.output else {
        unreachable!()
    };
    let proposal =
        propose_edit(&project(), &request, &response, &tts_context(&result.audio)).unwrap();

    let mut key = proposal.clone();
    let ProposalEvidence::ArtifactMaterial { artifact_key, .. } = &mut key.evidence[0] else {
        unreachable!()
    };
    *artifact_key = ContentDigest::sha256(b"tampered");
    assert!(canonical_edit_proposal_bytes(&key).is_err());

    let mut role = proposal.clone();
    let ProviderOutput::TextToSpeech(result) = &mut role.source_output else {
        unreachable!()
    };
    result.audio.role = ProviderArtifactSlot::new("tampered-role").unwrap();
    assert!(canonical_edit_proposal_bytes(&role).is_err());

    let mut operation = proposal;
    let veac_ir::EditOperation::InsertClip { clip, .. } = &mut operation.batch.operations[1] else {
        unreachable!()
    };
    clip.record_range.start = time(1);
    assert!(canonical_edit_proposal_bytes(&operation).is_err());
}

fn tts_output(value: &mut ApplicationContext) -> &mut AudioClipInsertion {
    let ApplicationContext::TextToSpeech(value) = value else {
        unreachable!()
    };
    &mut value.output
}

fn assert_invalid(value: ProviderResult<ProviderEditProposal>) {
    assert_eq!(value.unwrap_err().kind, ProviderErrorKind::InvalidContract);
}
