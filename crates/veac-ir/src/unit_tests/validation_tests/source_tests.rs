use std::collections::BTreeMap;

use crate::test_support::range;

use super::*;

#[test]
fn nested_sequence_cycles_are_rejected() {
    let mut project = sample_project();
    let mut nested = project.project.sequences[0].clone();
    nested.id = SequenceId::new("seq_nested").unwrap();
    nested.name = "Nested".to_owned();
    nested.tracks[0].id = TrackId::new("trk_nested_video").unwrap();
    nested.tracks[1].id = TrackId::new("trk_nested_caption").unwrap();
    nested.tracks[0].clips[0].id = ItemId::new("itm_nested_video").unwrap();
    nested.tracks[0].clips[0].effects[0].id = EffectId::new("fx_nested").unwrap();
    if let Animatable::Keyframes { keyframes } =
        &mut nested.tracks[0].clips[0].visual.as_mut().unwrap().opacity
    {
        keyframes[0].id = KeyframeId::new("kf_nested_start").unwrap();
        keyframes[1].id = KeyframeId::new("kf_nested_end").unwrap();
    }
    nested.tracks[1].clips[0].id = ItemId::new("itm_nested_caption").unwrap();
    nested.tracks[0].clips[0].source = ClipSource::Sequence {
        sequence_id: SequenceId::new("seq_main").unwrap(),
    };
    nested.tracks[0].clips[0].source_mapping = None;
    project.project.sequences[0].tracks[0].clips[0].source = ClipSource::Sequence {
        sequence_id: SequenceId::new("seq_nested").unwrap(),
    };
    project.project.sequences[0].tracks[0].clips[0].source_mapping = None;
    project.project.sequences.push(nested);
    assert_code(&validation_codes(&project), "SEQUENCE_CYCLE");
}

#[test]
fn nested_sequence_depth_is_bounded_without_recursive_validation() {
    let mut project = sample_project();
    project.project.sequences = (0..=MAX_SEQUENCE_NESTING_DEPTH)
        .map(|depth| {
            let source = if depth == MAX_SEQUENCE_NESTING_DEPTH {
                ClipSource::Generated {
                    generator: Generator::Transparent,
                }
            } else {
                ClipSource::Sequence {
                    sequence_id: SequenceId::new(format!("seq_depth_{}", depth + 1)).unwrap(),
                }
            };
            Sequence {
                id: SequenceId::new(format!("seq_depth_{depth}")).unwrap(),
                name: format!("Depth {depth}"),
                settings: project.project.sequences[0].settings.clone(),
                tracks: vec![Track {
                    id: TrackId::new(format!("trk_depth_{depth}")).unwrap(),
                    kind: TrackKind::Visual,
                    order: 0,
                    placement_mode: PlacementMode::Free,
                    state: TrackState {
                        enabled: true,
                        muted: false,
                        solo: false,
                        locked: false,
                    },
                    routing: TrackRouting::Default,
                    clips: vec![Clip {
                        id: ItemId::new(format!("itm_depth_{depth}")).unwrap(),
                        enabled: true,
                        record_range: range(0, 600),
                        source,
                        source_mapping: None,
                        visual: None,
                        audio: None,
                        effects: vec![],
                        replaceable: None,
                        template_editable_text: false,
                        metadata: BTreeMap::new(),
                    }],
                }],
                applies: vec![],
                metadata: BTreeMap::new(),
            }
        })
        .collect();
    project.project.entry_sequence_id = SequenceId::new("seq_depth_0").unwrap();
    project.project.render_configs[0].sequence_id = project.project.entry_sequence_id.clone();

    assert_code(&validation_codes(&project), "SEQUENCE_DEPTH");
}

#[test]
fn generated_and_audio_sources_validate_on_compatible_tracks() {
    let mut project = sample_project();
    project.project.materials.push(Material {
        id: MaterialId::new("med_audio").unwrap(),
        kind: MaterialKind::Audio,
        source: MaterialSource::Remote {
            uri: "https://cdn.example/audio.wav".to_owned(),
        },
        identity: None,
        stream_intent: StreamIntent {
            video: StreamChoice::Disabled,
            audio: StreamChoice::Auto,
        },
        probe: None,
        metadata: BTreeMap::new(),
    });
    let mut audio = project.project.sequences[0].tracks[0].clips[0].clone();
    audio.id = ItemId::new("itm_audio").unwrap();
    audio.source = ClipSource::Media {
        material_id: MaterialId::new("med_audio").unwrap(),
    };
    audio.visual = None;
    audio.effects.clear();
    let mut silence = project.project.sequences[0].tracks[1].clips[0].clone();
    silence.id = ItemId::new("itm_silence").unwrap();
    silence.source = ClipSource::Generated {
        generator: Generator::Silence,
    };
    silence.visual = None;
    silence.audio = Some(AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: vec![],
        crossfade: None,
    });
    silence.record_range = range(700, 60);
    project.project.sequences[0].tracks.extend([
        track("trk_audio", 20, vec![audio]),
        track("trk_silence", 30, vec![silence]),
    ]);
    validate(&project).unwrap();
}

fn track(id: &str, order: i32, clips: Vec<Clip>) -> Track {
    Track {
        id: TrackId::new(id).unwrap(),
        kind: TrackKind::Audio,
        order,
        placement_mode: PlacementMode::Free,
        state: TrackState {
            enabled: true,
            muted: false,
            solo: false,
            locked: false,
        },
        routing: TrackRouting::Default,
        clips,
    }
}
