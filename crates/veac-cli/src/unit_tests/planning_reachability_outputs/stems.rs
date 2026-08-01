use tempfile::tempdir;
use veac_ir::{
    Animatable, AudioCodec, AudioMixSource, AudioOutput, AudioProperties, AudioStemFormat,
    AudioStemOutput, BusId, ClipSource, Deliverable, DeliverableId, DeliverableKind,
    DeliverableTarget, ItemId, Material, MaterialId, MaterialKind, MaterialSource, PitchPolicy,
    StreamChoice, StreamIntent, TrackId, TrackKind, TrackRouting,
};

use super::super::support::{assert_project_inputs, canonical_project, MEDIA_SOURCE};

#[test]
fn track_bus_and_master_stems_hydrate_exact_audio_routes() {
    let temp = tempdir().unwrap();
    for name in ["a.wav", "b.wav"] {
        std::fs::write(temp.path().join(name), b"fixture").unwrap();
    }
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    let base = envelope.project.materials[0].clone();
    envelope.project.materials = vec![
        audio_material(&base, "med_audio_a", "a.wav"),
        audio_material(&base, "med_audio_b", "b.wav"),
    ];
    let mut track_a = envelope.project.sequences[0].tracks[0].clone();
    track_a.id = TrackId::new("trk_audio_a").unwrap();
    track_a.kind = TrackKind::Audio;
    track_a.clips[0].source = media("med_audio_a");
    track_a.clips[0].visual = None;
    track_a.clips[0].audio = Some(audio());
    let mut track_b = track_a.clone();
    track_b.id = TrackId::new("trk_audio_b").unwrap();
    track_b.order = 1;
    track_b.routing = TrackRouting::AudioBus {
        bus_id: BusId::new("bus_music").unwrap(),
    };
    track_b.clips[0].id = ItemId::new("itm_audio_b").unwrap();
    track_b.clips[0].source = media("med_audio_b");
    envelope.project.sequences[0].tracks = vec![track_a, track_b];
    envelope.project.render_configs[0].raster = None;
    envelope.project.render_configs[0].deliverables = vec![stem(AudioMixSource::Track {
        track_id: TrackId::new("trk_audio_a").unwrap(),
    })];

    assert_project_inputs(&project, &envelope, &["med_audio_a"], &[], &["a.wav"]);
    set_source(
        &mut envelope,
        AudioMixSource::Bus {
            bus_id: BusId::new("bus_music").unwrap(),
        },
    );
    assert_project_inputs(&project, &envelope, &["med_audio_b"], &[], &["b.wav"]);
    set_source(&mut envelope, AudioMixSource::Master);
    assert_project_inputs(
        &project,
        &envelope,
        &["med_audio_a", "med_audio_b"],
        &[],
        &["a.wav", "b.wav"],
    );
}

fn audio_material(base: &Material, id: &str, uri: &str) -> Material {
    let mut material = base.clone();
    material.id = MaterialId::new(id).unwrap();
    material.kind = MaterialKind::Audio;
    material.source = MaterialSource::File { uri: uri.into() };
    material.identity = None;
    material.probe = None;
    material.stream_intent = StreamIntent {
        video: StreamChoice::Disabled,
        audio: StreamChoice::Auto,
    };
    material
}

fn media(id: &str) -> ClipSource {
    ClipSource::Media {
        material_id: MaterialId::new(id).unwrap(),
    }
}

fn audio() -> AudioProperties {
    AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: vec![],
        crossfade: None,
    }
}

fn stem(source: AudioMixSource) -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_stem").unwrap(),
        target: DeliverableTarget::File {
            name: "stem.wav".into(),
        },
        kind: DeliverableKind::AudioStem(AudioStemOutput {
            format: AudioStemFormat::Wav,
            audio: AudioOutput {
                codec: AudioCodec::PcmS16Le,
                sample_rate: 48_000,
                channels: 2,
            },
            source,
        }),
    }
}

fn set_source(envelope: &mut veac_ir::ProjectEnvelope, source: AudioMixSource) {
    let DeliverableKind::AudioStem(settings) =
        &mut envelope.project.render_configs[0].deliverables[0].kind
    else {
        unreachable!()
    };
    settings.source = source;
}
