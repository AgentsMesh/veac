use tempfile::tempdir;
use veac_ir::{
    Animatable, AudioCodec, AudioMixSource, AudioOutput, AudioProperties, AudioStemFormat,
    AudioStemOutput, ClipSource, Deliverable, DeliverableId, DeliverableKind, DeliverableTarget,
    FrameSynthesisPolicy, ItemId, Material, MaterialId, MaterialKind, MaterialSource, PitchPolicy,
    PlacementMode, PlaybackDirection, Rational, RationalTime, Relation, RelationEndpoint,
    RelationId, RelationKind, SequenceId, SidechainRelationParameters, SourceMapping,
    SourceOutOfRangePolicy, SourceTimeMap, StreamChoice, StreamIntent, TimeRange, TrackId,
    TrackKind,
};

use super::super::support::{
    assert_project_inputs, canonical_project, write_project, FakeEnvironment, MEDIA_SOURCE,
};

#[test]
fn nested_sidechain_fixpoint_hydrates_only_the_projected_child_window() {
    let temp = tempdir().unwrap();
    for name in ["clip.mp4", "child-hit.wav"] {
        std::fs::write(temp.path().join(name), b"fixture").unwrap();
    }
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    envelope.project.materials[0].stream_intent.audio = StreamChoice::Auto;
    let base_material = envelope.project.materials[0].clone();
    let base_material_id = base_material.id.to_string();
    envelope.project.materials.extend([
        audio_material(&base_material, "med_child_hit", "child-hit.wav"),
        audio_material(
            &base_material,
            "med_child_miss",
            "definitely-missing-child.wav",
        ),
    ]);
    let main_track_id = envelope.project.sequences[0].tracks[0].id.clone();
    let config = &mut envelope.project.render_configs[0];
    config.raster = None;
    config.deliverables = vec![Deliverable {
        id: DeliverableId::new("dlv_stem").unwrap(),
        target: DeliverableTarget::File {
            name: "target.wav".into(),
        },
        kind: DeliverableKind::AudioStem(AudioStemOutput {
            format: AudioStemFormat::Wav,
            audio: AudioOutput {
                codec: AudioCodec::PcmS16Le,
                sample_rate: 48_000,
                channels: 2,
            },
            source: AudioMixSource::Track {
                track_id: main_track_id,
            },
        }),
    }];

    let main = &mut envelope.project.sequences[0];
    let main_id = main.id.clone();
    let target_id = main.tracks[0].clips[0].id.clone();
    let mut base_clip = main.tracks[0].clips[0].clone();
    base_clip.record_range.duration = time(600);
    base_clip.audio = Some(audio());
    main.tracks[0].clips[0] = base_clip.clone();
    let mut nested = base_clip.clone();
    nested.id = ItemId::new("itm_nested_key").unwrap();
    nested.source = ClipSource::Sequence {
        sequence_id: SequenceId::new("seq_child").unwrap(),
    };
    nested.source_mapping = Some(reverse_repeat_mapping());
    nested.visual = None;
    let mut key_track = main.tracks[0].clone();
    key_track.id = TrackId::new("trk_key").unwrap();
    key_track.kind = TrackKind::Audio;
    key_track.order = 1;
    key_track.placement_mode = PlacementMode::Free;
    key_track.clips = vec![nested];
    main.tracks.push(key_track);

    let mut hit = child_clip(&base_clip, "itm_child_hit", "med_child_hit", 150, 100);
    let miss = child_clip(&base_clip, "itm_child_miss", "med_child_miss", 550, 50);
    hit.visual = None;
    let mut child_track = main.tracks[0].clone();
    child_track.id = TrackId::new("trk_child").unwrap();
    child_track.kind = TrackKind::Audio;
    child_track.order = 0;
    child_track.placement_mode = PlacementMode::Free;
    child_track.clips = vec![hit, miss];
    let mut child = main.clone();
    child.id = SequenceId::new("seq_child").unwrap();
    child.name = "child".into();
    child.tracks = vec![child_track];
    child.applies.clear();
    envelope.project.sequences.push(child);
    envelope.project.relations.push(Relation {
        id: RelationId::new("rel_nested_key").unwrap(),
        sequence_id: main_id,
        kind: RelationKind::Sidechain {
            key: RelationEndpoint::track(TrackId::new("trk_key").unwrap()),
            target: RelationEndpoint::item(target_id),
            parameters: SidechainRelationParameters {
                threshold_db: -18.0,
                ratio: 4.0,
                attack_ms: 10.0,
                release_ms: 200.0,
                active_range: Some(range(50, 100)),
            },
        },
    });

    assert_project_inputs(
        &project,
        &envelope,
        &["med_child_hit", base_material_id.as_str()],
        &[],
        &["child-hit.wav", "clip.mp4"],
    );
    let clips = &mut envelope.project.sequences[1].tracks[0].clips;
    clips[0].source = media("med_child_miss");
    clips[1].source = media("med_child_hit");
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

fn child_clip(
    base: &veac_ir::Clip,
    item: &str,
    material: &str,
    start: i64,
    duration: i64,
) -> veac_ir::Clip {
    let mut clip = base.clone();
    clip.id = ItemId::new(item).unwrap();
    clip.record_range = range(start, duration);
    clip.source = media(material);
    clip.visual = None;
    clip.audio = Some(audio());
    clip
}

fn media(id: &str) -> ClipSource {
    ClipSource::Media {
        material_id: MaterialId::new(id).unwrap(),
    }
}

fn reverse_repeat_mapping() -> SourceMapping {
    SourceMapping {
        time_map: SourceTimeMap::Linear {
            source_start: time(0),
            rate: Rational::new(1, 1).unwrap(),
            repeat: 2,
            direction: PlaybackDirection::Reverse,
        },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: SourceOutOfRangePolicy::Strict,
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

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 1000).unwrap()
}

fn range(start: i64, duration: i64) -> TimeRange {
    TimeRange::new(time(start), time(duration)).unwrap()
}
