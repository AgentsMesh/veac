use tempfile::tempdir;
use veac_ir::{
    Animatable, AudioCodec, AudioMixSource, AudioOutput, AudioProperties, AudioStemFormat,
    AudioStemOutput, ClipSource, Deliverable, DeliverableId, DeliverableKind, DeliverableTarget,
    ItemId, MaterialId, MaterialKind, MaterialSource, PitchPolicy, StreamChoice, TrackId,
    TrackKind,
};

use super::support::{
    accessed_names, assert_exact_inputs, canonical_project, write_project, FakeEnvironment,
    MEDIA_SOURCE,
};

#[test]
fn unavailable_unused_material_succeeds_until_it_becomes_required() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("clip.mp4"), b"fixture").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    let base_id = envelope.project.materials[0].id.to_string();
    add_material(&mut envelope, "med_offline", "missing.mp4");
    write_project(&project, &envelope);
    let environment = FakeEnvironment::success();
    let prepared =
        crate::planning::prepare_with_material_root(&project, None, None, &environment).unwrap();
    assert_exact_inputs(&prepared, &[base_id.as_str()]);
    assert_eq!(
        accessed_names(&environment.probe_paths),
        std::collections::BTreeSet::from(["clip.mp4".to_owned()])
    );
    assert!(environment.identity_paths.borrow().is_empty());

    envelope.project.sequences[0].tracks[0].clips[0].source = media("med_offline");
    write_project(&project, &envelope);
    let error = crate::planning::prepare_with_material_root(
        &project,
        None,
        None,
        &FakeEnvironment::success(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("PATH_UNAVAILABLE"));
}

#[test]
fn disabled_clips_tracks_and_solo_suppression_are_not_hydrated() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("clip.mp4"), b"fixture").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    let base_id = envelope.project.materials[0].id.to_string();
    add_material(&mut envelope, "med_disabled", "disabled.mp4");
    add_material(&mut envelope, "med_suppressed", "suppressed.mp4");
    let mut suppressed = envelope.project.sequences[0].tracks[0].clone();
    suppressed.id = TrackId::new("trk_suppressed").unwrap();
    suppressed.order = 1;
    suppressed.clips[0].id = ItemId::new("itm_suppressed").unwrap();
    suppressed.clips[0].source = media("med_suppressed");
    let mut disabled = suppressed.clips[0].clone();
    disabled.id = ItemId::new("itm_disabled").unwrap();
    disabled.enabled = false;
    disabled.source = media("med_disabled");
    envelope.project.sequences[0].tracks[0].clips.push(disabled);
    envelope.project.sequences[0].tracks[0].state.solo = true;
    envelope.project.sequences[0].tracks.push(suppressed);
    write_project(&project, &envelope);

    let prepared = crate::planning::prepare_with_material_root(
        &project,
        None,
        None,
        &FakeEnvironment::success(),
    )
    .unwrap();
    assert_exact_inputs(&prepared, &[base_id.as_str()]);
    envelope.project.sequences[0].tracks[0].state.solo = false;
    envelope.project.sequences[0].tracks[1].state.enabled = false;
    write_project(&project, &envelope);
    let prepared = crate::planning::prepare_with_material_root(
        &project,
        None,
        None,
        &FakeEnvironment::success(),
    )
    .unwrap();
    assert_exact_inputs(&prepared, &[base_id.as_str()]);
}

#[test]
fn muted_audio_only_delivery_with_no_raster_needs_no_media() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    let material = &mut envelope.project.materials[0];
    material.kind = MaterialKind::Audio;
    material.source = MaterialSource::File {
        uri: "missing.wav".into(),
    };
    material.stream_intent.video = StreamChoice::Disabled;
    material.stream_intent.audio = StreamChoice::Auto;
    let track = &mut envelope.project.sequences[0].tracks[0];
    track.kind = TrackKind::Audio;
    track.state.muted = true;
    track.clips[0].visual = None;
    track.clips[0].audio = Some(audio_properties(true));
    envelope.project.render_configs[0].raster = None;
    envelope.project.render_configs[0].deliverables = vec![master_stem()];
    write_project(&project, &envelope);

    let prepared = crate::planning::prepare_with_material_root(
        &project,
        None,
        None,
        &FakeEnvironment::success(),
    )
    .unwrap();
    assert_exact_inputs(&prepared, &[]);
    track_state(&mut envelope, false, true);
    write_project(&project, &envelope);
    let prepared = crate::planning::prepare_with_material_root(
        &project,
        None,
        None,
        &FakeEnvironment::success(),
    )
    .unwrap();
    assert_exact_inputs(&prepared, &[]);
    track_state(&mut envelope, false, false);
    write_project(&project, &envelope);
    let error = crate::planning::prepare_with_material_root(
        &project,
        None,
        None,
        &FakeEnvironment::success(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("PATH_UNAVAILABLE"));
}

fn track_state(envelope: &mut veac_ir::ProjectEnvelope, track_muted: bool, clip_muted: bool) {
    let track = &mut envelope.project.sequences[0].tracks[0];
    track.state.muted = track_muted;
    track.clips[0].audio.as_mut().unwrap().muted = clip_muted;
}

fn add_material(envelope: &mut veac_ir::ProjectEnvelope, id: &str, uri: &str) {
    let mut material = envelope.project.materials[0].clone();
    material.id = MaterialId::new(id).unwrap();
    material.source = MaterialSource::File { uri: uri.into() };
    material.identity = None;
    material.probe = None;
    envelope.project.materials.push(material);
}

fn media(id: &str) -> ClipSource {
    ClipSource::Media {
        material_id: MaterialId::new(id).unwrap(),
    }
}

fn audio_properties(muted: bool) -> AudioProperties {
    AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: vec![],
        crossfade: None,
    }
}

fn master_stem() -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_stem").unwrap(),
        target: DeliverableTarget::File {
            name: "master.wav".into(),
        },
        kind: DeliverableKind::AudioStem(AudioStemOutput {
            format: AudioStemFormat::Wav,
            audio: AudioOutput {
                codec: AudioCodec::PcmS16Le,
                sample_rate: 48_000,
                channels: 2,
            },
            source: AudioMixSource::Master,
        }),
    }
}
