use std::collections::BTreeMap;

use veac_plan::canonical::*;

use super::support::{add_transition, bindings, emit_video_command, fixture, resolved, visual};

#[test]
fn emits_common_visual_pipeline_and_ordered_effects() {
    let mut project = fixture();
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    clip.visual = Some(visual());
    clip.effects = vec![
        effect("fx_color", "video.color_adjust", "brightness", 0.2),
        effect("fx_blur", "video.blur", "radius", 2.0),
    ];
    let plan = resolved(&project);
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();

    let ordered = [
        "crop=w=",
        "force_original_aspect_ratio=increase",
        "eq=brightness",
        "gblur@",
        "transformscalev",
        "cornerv",
        "maskv",
        "rotate=",
        "opacityv",
    ];
    let mut previous = 0;
    for needle in ordered {
        let index = graph
            .find(needle)
            .unwrap_or_else(|| panic!("missing {needle}: {graph}"));
        assert!(index >= previous, "{needle} was out of order");
        previous = index;
    }
    assert!(graph.contains("blend=all_mode=screen"));
    assert!(graph.contains("maskedmerge=planes=7"));
    assert!(graph.contains("pow("));
}

#[test]
fn emits_real_audio_source_timing_automation_and_mix() {
    let mut project = fixture();
    project.project.render_configs[0]
        .video_deliverable_mut()
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    clip.audio = Some(AudioProperties {
        gain: Animatable::constant(0.7),
        pan: Animatable::constant(-0.25),
        muted: false,
        normalize: true,
        pitch_policy: PitchPolicy::FollowSpeed,
        processors: vec![],
        crossfade: None,
    });
    let SourceTimeMap::Linear { rate, .. } = &mut clip.source_mapping.as_mut().unwrap().time_map
    else {
        panic!("linear mapping");
    };
    *rate = Rational::new(2, 1).unwrap();
    clip.effects
        .push(effect("fx_norm", "audio.normalize", "target_lufs", -18.0));
    let plan = resolved(&project);
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();

    assert!(graph.contains("[0:5]atrim=start=0:duration=2"));
    assert!(graph.contains("asetrate=48000*2,aresample=48000"));
    assert!(graph.contains("volume='0.7':eval=frame"));
    assert!(graph.contains("channelsplit=channel_layout=stereo"));
    assert!(graph.contains("loudnorm=I=-16"));
    assert!(graph.contains("loudnorm=I=-18"));
    assert!(graph.contains("amix=inputs=2:normalize=0"));
}

#[test]
fn materializes_centered_video_transition_and_audio_edge_fades() {
    let mut project = fixture();
    project.project.render_configs[0]
        .video_deliverable_mut()
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
    let track = &mut project.project.sequences[0].tracks[0];
    track.clips[0].audio = Some(default_audio());
    let transition = Transition {
        kind: TransitionKind::Dissolve,
        duration: RationalTime::new(120, 600).unwrap(),
        alignment: TransitionAlignment::Centered,
    };
    let mut incoming = track.clips[0].clone();
    incoming.id = ItemId::new("itm_incoming").unwrap();
    incoming.record_range.start = RationalTime::new(600, 600).unwrap();
    track.clips.push(incoming);
    add_transition(
        &mut project,
        "seq_main",
        "itm_video",
        "itm_incoming",
        transition,
    );

    let plan = resolved(&project);
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    assert!(graph.contains("tpad=stop_mode=clone:stop_duration=0.1"));
    assert!(graph.contains("tpad=start_mode=clone:start_duration=0.1"));
    assert!(graph.contains("xfade=transition=fade:duration=0.2:offset=0"));
    assert!(graph.contains("afade=t=out:st=0.9:d=0.1"));
    assert!(graph.contains("afade=t=in:st=0:d=0.1"));
}

fn default_audio() -> AudioProperties {
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

fn effect(id: &str, kind: &str, parameter: &str, value: f64) -> EffectInstance {
    EffectInstance {
        id: EffectId::new(id).unwrap(),
        effect_type: kind.to_owned(),
        enabled: true,
        enable_range: None,
        parameters: BTreeMap::from([(parameter.to_owned(), ParameterValue::Number { value })]),
    }
}
