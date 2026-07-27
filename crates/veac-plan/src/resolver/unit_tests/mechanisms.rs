use std::collections::BTreeMap;

use super::mechanism_helpers::*;
use super::support::*;
use crate::{canonical::*, *};

#[test]
fn nested_transitions_disabled_clips_and_solo_are_explicit() {
    let mut value = project();
    let nested = sequence(
        "seq_nested",
        vec![track(
            "trk_nested",
            TrackKind::Visual,
            0,
            vec![generated_clip("itm_nested", Generator::Transparent, 0)],
        )],
    );
    value.project.sequences.push(nested);
    let main = &mut value.project.sequences[0];
    main.tracks[0].clips[0].effects = vec![
        EffectInstance {
            id: EffectId::new("fx_blur").unwrap(),
            effect_type: "video.blur".to_owned(),
            enabled: true,
            enable_range: None,
            parameters: BTreeMap::from([(
                "radius".to_owned(),
                ParameterValue::Number { value: 4.0 },
            )]),
        },
        EffectInstance {
            id: EffectId::new("fx_off").unwrap(),
            effect_type: "video.grain".to_owned(),
            enabled: false,
            enable_range: None,
            parameters: BTreeMap::from([(
                "amount".to_owned(),
                ParameterValue::Number { value: 0.2 },
            )]),
        },
    ];
    let transition = Transition {
        kind: TransitionKind::Dissolve,
        duration: time(120),
        alignment: TransitionAlignment::Centered,
    };
    main.tracks[0]
        .clips
        .push(media_clip("itm_second", "med_video", 600));
    let mut disabled = generated_clip("itm_disabled", Generator::Transparent, 0);
    disabled.enabled = false;
    main.tracks.push(track(
        "trk_precomp",
        TrackKind::Visual,
        10,
        vec![
            sequence_clip("itm_precomp_a"),
            sequence_clip("itm_precomp_b"),
            disabled,
        ],
    ));
    add_transition(
        &mut value,
        "seq_main",
        "itm_video",
        "itm_second",
        transition,
    );
    let plan = resolve(&value, None).unwrap().remove(0);
    assert_eq!(plan.sequences[0].id.as_str(), "seq_nested");
    assert_eq!(plan.sequences[1].tracks[0].transitions.len(), 1);
    assert_eq!(plan.sequences[1].tracks[0].clips[0].effects.len(), 1);
    assert_eq!(plan.sequences[1].tracks[1].clips.len(), 2);

    let mut solo = project();
    solo.project.materials.push(remote_material("med_remote"));
    solo.project.sequences[0].tracks[0].state.solo = true;
    solo.project.sequences[0].tracks.push(track(
        "trk_remote",
        TrackKind::Visual,
        20,
        vec![media_clip("itm_remote", "med_remote", 0)],
    ));
    let plan = resolve(&solo, None).unwrap().remove(0);
    assert!(!plan.sequences[0].tracks[1].state.include_in_render);
}

#[test]
fn audio_bus_image_freeze_font_and_generated_sources_resolve() {
    let mut value = project();
    value.project.render_configs[0]
        .video_deliverable_mut()
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
    value.project.materials.extend([
        audio_material("med_audio"),
        image_material("med_image"),
        font_material("med_font"),
    ]);
    let main = &mut value.project.sequences[0];
    let mut audio = track(
        "trk_audio",
        TrackKind::Audio,
        10,
        vec![media_clip("itm_audio", "med_audio", 0)],
    );
    audio.routing = TrackRouting::AudioBus {
        bus_id: BusId::new("bus_dialog").unwrap(),
    };
    main.tracks.push(audio);
    main.tracks.push(track(
        "trk_image",
        TrackKind::Visual,
        20,
        vec![freeze("itm_image", "med_image", 0)],
    ));
    main.tracks.push(track(
        "trk_text",
        TrackKind::Visual,
        30,
        vec![text_clip(false)],
    ));
    main.tracks.push(track(
        "trk_caption",
        TrackKind::Caption,
        40,
        vec![text_clip(true)],
    ));
    main.tracks.push(track(
        "trk_silence",
        TrackKind::Audio,
        50,
        vec![generated_clip("itm_silence", Generator::Silence, 0)],
    ));
    let plan = resolve(&value, None).unwrap().remove(0);
    assert_eq!(plan.inputs.len(), 4);
    assert!(matches!(
        plan.sequences[0].tracks[1].routing.audio,
        Some(ResolvedAudioRoute::Bus { .. })
    ));
    assert!(plan.sequences[0].tracks[1].clips[0].audio.is_some());

    let mut generated = project();
    generated.project.materials.clear();
    generated.project.sequences[0].tracks[0].clips = vec![generated_clip(
        "itm_solid",
        Generator::Solid { color: black() },
        0,
    )];
    let plan = resolve(&generated, None).unwrap().remove(0);
    assert!(plan.inputs.is_empty());
    assert_eq!(plan.header.resolver.stream_selection_policy, "none");
}

#[test]
fn audio_and_freeze_bounds_and_family_font_fail_early() {
    let mut audio = project();
    audio.project.render_configs[0]
        .video_deliverable_mut()
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
    let mut material = audio_material("med_audio");
    material.probe.as_mut().unwrap().container_duration = None;
    material.probe.as_mut().unwrap().streams[0].duration = None;
    audio.project.materials.push(material);
    audio.project.sequences[0].tracks.push(track(
        "trk_audio",
        TrackKind::Audio,
        10,
        vec![media_clip("itm_audio", "med_audio", 0)],
    ));
    assert_code(audio, "SOURCE_DURATION_UNAVAILABLE");

    let mut freeze_missing = freeze_project(time(100));
    let probe = freeze_missing.project.materials[0].probe.as_mut().unwrap();
    probe.container_duration = None;
    probe.streams[0].duration = None;
    assert_code(freeze_missing, "SOURCE_DURATION_UNAVAILABLE");
    assert_code(freeze_project(time(6_000)), "SOURCE_RANGE_OUT_OF_BOUNDS");

    let mut family = project();
    family.project.sequences[0].tracks[0].kind = TrackKind::Visual;
    family.project.sequences[0].tracks[0].clips = vec![family_text_clip()];
    assert_code(family, "FONT_FAMILY_UNRESOLVED");
}
