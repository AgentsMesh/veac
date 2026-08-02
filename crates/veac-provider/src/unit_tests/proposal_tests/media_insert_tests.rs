use veac_ir::{ClipSource, EditOperation, EditOutcome, SourceTimeMap, StructureEdit};

use crate::*;

use super::support::*;

#[test]
fn tts_inserts_a_pinned_material_and_exact_audio_clip() {
    let project = project();
    let (request, response) = exchange(Capability::TextToSpeech);
    let ProviderOutput::TextToSpeech(result) = &response.output else {
        unreachable!()
    };
    let proposal =
        propose_edit(&project, &request, &response, &tts_context(&result.audio)).unwrap();
    assert_eq!(proposal.batch.operations.len(), 2);
    assert!(matches!(
        &proposal.batch.operations[0],
        EditOperation::EditStructure {
            edit: StructureEdit::InsertMaterial { material, .. }
        } if material.id.as_str() == "med_provider_tts"
    ));
    let EditOperation::InsertClip { clip, .. } = &proposal.batch.operations[1] else {
        unreachable!()
    };
    assert_eq!(clip.record_range, crate::test_support::range(0, 10));
    assert!(matches!(
        &clip.source,
        ClipSource::Media { material_id } if material_id.as_str() == "med_provider_tts"
    ));
    assert!(matches!(
        &clip.source_mapping.as_ref().unwrap().time_map,
        SourceTimeMap::Linear { source_start, .. } if *source_start == time(0)
    ));
    assert!(matches!(
        &proposal.evidence[1],
        ProposalEvidence::GeneratedAudioClip { operation, artifact_key, .. }
            if operation.operation_index == 1 && *artifact_key == result.audio.record.key
    ));
    let EditOutcome::Applied {
        project: applied, ..
    } = veac_ir::apply_edit_batch(&project, &proposal.batch)
    else {
        unreachable!()
    };
    assert_eq!(applied.project.materials.len(), 3);
    assert_eq!(applied.project.sequences[0].tracks[2].clips.len(), 2);
}

#[test]
fn tts_rejects_output_range_and_locked_track_mismatches() {
    let (request, mut response) = exchange(Capability::TextToSpeech);
    let ProviderOutput::TextToSpeech(result) = &response.output else {
        unreachable!()
    };
    let context = tts_context(&result.audio);
    let ProviderOutput::TextToSpeech(result) = &mut response.output else {
        unreachable!()
    };
    result.range = crate::test_support::range(1, 10);
    assert_invalid(propose_edit(&project(), &request, &response, &context));

    let (request, response) = exchange(Capability::TextToSpeech);
    let ProviderOutput::TextToSpeech(result) = &response.output else {
        unreachable!()
    };
    let mut locked = project();
    locked.project.sequences[0].tracks[2].state.locked = true;
    assert_invalid(propose_edit(
        &locked,
        &request,
        &response,
        &tts_context(&result.audio),
    ));
}

#[test]
fn dubbing_binds_requested_turns_source_and_rendered_range() {
    let project = project();
    let (request, response) = exchange(Capability::Dubbing);
    let ProviderOutput::Dubbing(result) = &response.output else {
        unreachable!()
    };
    let proposal = propose_edit(
        &project,
        &request,
        &response,
        &dubbing_context(&result.audio),
    )
    .unwrap();
    assert_eq!(proposal.batch.operations.len(), 2);
    assert!(proposal.batch.preconditions.iter().any(|value| matches!(
        value,
        veac_ir::Precondition::ClipSourceEquals { clip_id, .. }
            if clip_id.as_str() == "itm_audio"
    )));
    assert!(matches!(
        veac_ir::apply_edit_batch(&project, &proposal.batch),
        EditOutcome::Applied { .. }
    ));
}

#[test]
fn dubbing_rejects_turn_and_source_identity_mismatches() {
    let (request, mut response) = exchange(Capability::Dubbing);
    let ProviderOutput::Dubbing(result) = &response.output else {
        unreachable!()
    };
    let context = dubbing_context(&result.audio);
    let ProviderOutput::Dubbing(result) = &mut response.output else {
        unreachable!()
    };
    result.turns[0].id = "turn-other".into();
    assert_invalid(propose_edit(&project(), &request, &response, &context));

    let (request, response) = exchange(Capability::Dubbing);
    let ProviderOutput::Dubbing(result) = &response.output else {
        unreachable!()
    };
    let mut wrong = project();
    wrong.project.materials[0].identity.as_mut().unwrap().digest = "a".repeat(64);
    wrong.project.materials[0]
        .probe
        .as_mut()
        .unwrap()
        .observed_identity
        .digest = "a".repeat(64);
    assert_invalid(propose_edit(
        &wrong,
        &request,
        &response,
        &dubbing_context(&result.audio),
    ));
}

fn assert_invalid(value: ProviderResult<ProviderEditProposal>) {
    assert_eq!(value.unwrap_err().kind, ProviderErrorKind::InvalidContract);
}
