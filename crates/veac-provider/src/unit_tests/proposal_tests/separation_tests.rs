use veac_artifact::ArtifactKind;
use veac_ir::{EditOutcome, ItemId, MaterialId, MaterialKind, TimeRange};

use crate::*;

use super::support::*;

#[test]
fn separation_inserts_every_ordered_stem_as_material_and_audio_clip() {
    let project = project();
    let (request, mut response) = exchange(Capability::VocalSeparation);
    let music_artifact = crate::test_support::artifact(
        ArtifactKind::AudioStem,
        "music",
        &request.provider,
        request_hash(&request).unwrap(),
    );
    let ProviderOutput::VocalSeparation(result) = &mut response.output else {
        unreachable!()
    };
    result.stems.push(SeparatedStem {
        kind: StemKind::Music,
        label: "music".into(),
        audio: music_artifact.clone(),
        leakage: 0.04,
    });
    let mut context = separation_context(&result.stems[0]);
    let ApplicationContext::VocalSeparation(value) = &mut context else {
        unreachable!()
    };
    let mut music = value.bindings[0].clone();
    music.kind = StemKind::Music;
    music.label = "music".into();
    music.output.material = output_material(
        &music_artifact,
        "med_provider_music",
        MaterialKind::Audio,
        time(100),
    );
    music.output.clip_id = ItemId::new("itm_provider_music").unwrap();
    value.bindings.push(music);

    let proposal = propose_edit(&project, &request, &response, &context).unwrap();
    assert_eq!(proposal.batch.operations.len(), 4);
    assert!(matches!(
        &proposal.evidence[3],
        ProposalEvidence::SeparatedStemClip { operation, kind, .. }
            if operation.operation_index == 3 && *kind == StemKind::Music
    ));
    let EditOutcome::Applied {
        project: applied, ..
    } = veac_ir::apply_edit_batch(&project, &proposal.batch)
    else {
        unreachable!()
    };
    assert_eq!(applied.project.materials.len(), 4);
    assert_eq!(applied.project.sequences[0].tracks[2].clips.len(), 3);
}

#[test]
fn separation_rejects_label_kind_range_and_locked_track_mismatches() {
    let (request, response) = exchange(Capability::VocalSeparation);
    let ProviderOutput::VocalSeparation(result) = &response.output else {
        unreachable!()
    };
    let mut label = separation_context(&result.stems[0]);
    let ApplicationContext::VocalSeparation(value) = &mut label else {
        unreachable!()
    };
    value.bindings[0].label = "wrong".into();
    assert_invalid(propose_edit(&project(), &request, &response, &label));

    let mut kind = separation_context(&result.stems[0]);
    let ApplicationContext::VocalSeparation(value) = &mut kind else {
        unreachable!()
    };
    value.bindings[0].kind = StemKind::Music;
    assert_invalid(propose_edit(&project(), &request, &response, &kind));

    let mut range = separation_context(&result.stems[0]);
    let ApplicationContext::VocalSeparation(value) = &mut range else {
        unreachable!()
    };
    value.bindings[0].output.record_range = TimeRange::new(time(1), time(100)).unwrap();
    assert_invalid(propose_edit(&project(), &request, &response, &range));

    let mut locked = project();
    locked.project.sequences[0].tracks[2].state.locked = true;
    assert_invalid(propose_edit(
        &locked,
        &request,
        &response,
        &separation_context(&result.stems[0]),
    ));
}

#[test]
fn separation_rejects_duplicate_material_and_clip_ids() {
    let (request, mut response) = exchange(Capability::VocalSeparation);
    let second_artifact = crate::test_support::artifact(
        ArtifactKind::AudioStem,
        "music",
        &request.provider,
        request_hash(&request).unwrap(),
    );
    let ProviderOutput::VocalSeparation(result) = &mut response.output else {
        unreachable!()
    };
    result.stems.push(SeparatedStem {
        kind: StemKind::Music,
        label: "music".into(),
        audio: second_artifact,
        leakage: 0.1,
    });
    let mut context = separation_context(&result.stems[0]);
    let ApplicationContext::VocalSeparation(value) = &mut context else {
        unreachable!()
    };
    let mut duplicate = value.bindings[0].clone();
    duplicate.kind = StemKind::Music;
    duplicate.label = "music".into();
    duplicate.output.material.material.id = MaterialId::new("med_provider_stem").unwrap();
    duplicate.output.clip_id = ItemId::new("itm_provider_stem").unwrap();
    value.bindings.push(duplicate);
    assert_invalid(propose_edit(&project(), &request, &response, &context));
}

fn assert_invalid(value: ProviderResult<ProviderEditProposal>) {
    assert_eq!(value.unwrap_err().kind, ProviderErrorKind::InvalidContract);
}
