mod support;

use veac_plan::{canonical::*, resolve, ResolvedClip, ResolvedClipSource, ResolvedSourceTimeMap};
use {std::collections::BTreeMap, support::*};

#[test]
fn resolves_fonts_freeze_generated_audio_effects_and_transitions() {
    let mut project = project();
    project.project.render_configs[0]
        .video_deliverable_mut()
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
    project.project.materials.push(font_material("med_font"));
    let main = &mut project.project.sequences[0];
    let first = &mut main.tracks[0].clips[0];
    first.effects = vec![
        effect("fx_blur", "video.blur", true),
        effect("fx_off", "video.grain", false),
    ];
    let transition = Transition {
        kind: TransitionKind::Dissolve,
        duration: time(120),
        alignment: TransitionAlignment::Centered,
    };
    let mut second = media_clip("itm_second", "med_video", 600);
    let mapping = second.source_mapping.as_mut().unwrap();
    let SourceTimeMap::Linear { rate, repeat, .. } = &mut mapping.time_map else {
        panic!("linear mapping");
    };
    *rate = Rational::new(3, 2).unwrap();
    *repeat = 2;
    main.tracks[0].clips.push(second);
    main.tracks.push(track(
        "trk_freeze",
        TrackKind::Visual,
        10,
        vec![freeze_clip()],
    ));
    main.tracks.push(track(
        "trk_text",
        TrackKind::Visual,
        20,
        vec![text_clip(false)],
    ));
    main.tracks.push(track(
        "trk_caption",
        TrackKind::Caption,
        30,
        vec![text_clip(true)],
    ));
    main.tracks.push(track(
        "trk_silence",
        TrackKind::Audio,
        40,
        vec![generated_clip("itm_silence", Generator::Silence, 0)],
    ));
    add_transition(
        &mut project,
        "seq_main",
        "itm_video",
        "itm_second",
        transition,
    );

    let plan = resolve(&project, None).unwrap().remove(0);
    assert_eq!(plan.inputs.len(), 2);
    let font = plan
        .inputs
        .iter()
        .find(|input| input.material_id.as_ref().unwrap().as_str() == "med_font")
        .unwrap();
    assert!(font.probe.is_none());
    let main = plan.sequences.last().unwrap();
    let video = &main.tracks[0];
    assert_eq!(video.transitions.len(), 1);
    assert_eq!(video.transitions[0].outgoing_handle.duration, time(60));
    assert_eq!(video.transitions[0].incoming_handle.duration, time(60));
    assert_eq!(video.clips[0].effects.len(), 1);
    let mapping = video.clips[1].source_mapping.as_ref().unwrap();
    let ResolvedSourceTimeMap::Linear {
        source_range_per_repeat,
        ..
    } = mapping.time_map
    else {
        panic!("linear mapping");
    };
    assert_eq!(source_range_per_repeat.duration, time(450));
    assert!(matches!(
        find_clip(main, "itm_freeze").source,
        ResolvedClipSource::FreezeFrame { .. }
    ));
    assert!(matches!(
        find_clip(main, "itm_text").source,
        ResolvedClipSource::Text { .. }
    ));
    assert!(matches!(
        find_clip(main, "itm_caption").source,
        ResolvedClipSource::Caption { .. }
    ));
    let silence = find_clip(main, "itm_silence");
    assert!(matches!(
        silence.source,
        ResolvedClipSource::Generated { .. }
    ));
    assert!(silence.audio.is_some());
}

#[test]
fn transition_alignment_materializes_both_zero_handle_cases() {
    let cases = [
        (TransitionAlignment::BeforeCut, 120, 0),
        (TransitionAlignment::Centered, 60, 60),
        (TransitionAlignment::AfterCut, 0, 120),
    ];
    for (alignment, outgoing, incoming) in cases {
        let mut project = project();
        let track = &mut project.project.sequences[0].tracks[0];
        let transition = Transition {
            kind: TransitionKind::Dissolve,
            duration: time(120),
            alignment,
        };
        track
            .clips
            .push(media_clip("itm_incoming", "med_video", 600));
        add_transition(
            &mut project,
            "seq_main",
            "itm_video",
            "itm_incoming",
            transition,
        );
        let plan = resolve(&project, None).unwrap().remove(0);
        let transition = &plan.sequences[0].tracks[0].transitions[0];
        assert_eq!(transition.outgoing_handle.duration, time(outgoing));
        assert_eq!(transition.incoming_handle.duration, time(incoming));
    }
}

fn freeze_clip() -> Clip {
    let mut clip = generated_clip("itm_freeze", Generator::Transparent, 0);
    clip.source = ClipSource::FreezeFrame {
        material_id: MaterialId::new("med_video").unwrap(),
        source_time: time(100),
    };
    clip.visual = Some(visual_properties());
    clip
}

fn text_clip(caption: bool) -> Clip {
    let id = if caption { "itm_caption" } else { "itm_text" };
    let mut clip = generated_clip(id, Generator::Transparent, 0);
    let style = text_style(FontRef::Material {
        material_id: MaterialId::new("med_font").unwrap(),
    });
    clip.source = if caption {
        ClipSource::Caption {
            text: "caption".to_owned(),
            speaker: None,
            style,
        }
    } else {
        ClipSource::Text {
            text: "title".to_owned(),
            style,
        }
    };
    clip.visual = Some(visual_properties());
    clip
}

fn effect(id: &str, effect_type: &str, enabled: bool) -> EffectInstance {
    let parameters = if effect_type == "video.blur" {
        BTreeMap::from([("radius".to_owned(), ParameterValue::Number { value: 4.0 })])
    } else {
        BTreeMap::from([("amount".to_owned(), ParameterValue::Number { value: 0.2 })])
    };
    EffectInstance {
        id: EffectId::new(id).unwrap(),
        effect_type: effect_type.to_owned(),
        enabled,
        enable_range: None,
        parameters,
    }
}

fn find_clip<'a>(sequence: &'a veac_plan::ResolvedSequence, id: &str) -> &'a ResolvedClip {
    sequence
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .find(|clip| clip.id.as_str() == id)
        .unwrap()
}
