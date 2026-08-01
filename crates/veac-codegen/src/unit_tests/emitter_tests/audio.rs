use veac_plan::canonical::*;
use veac_plan::{ResolvedAudioRoute, ResolvedClipSource, ResolvedSourceTimeMap};

use super::support::{fixture, resolved, time};

mod support;

use self::support::{error as assert_audio_error, graph, silence_only as assert_silence_only};

#[test]
fn preserve_pitch_reverse_repeat_and_extreme_tempo_emit_exact_filters() {
    let mut slow = audio_plan(2);
    let slow_clip = clip(&mut slow);
    slow_clip.audio.as_mut().unwrap().pitch_policy = PitchPolicy::Preserve;
    let mapping = slow_clip.source_mapping.as_mut().unwrap();
    let ResolvedSourceTimeMap::Linear {
        source_range_per_repeat,
        rate,
        direction,
        repeat,
    } = &mut mapping.time_map
    else {
        panic!("linear mapping");
    };
    *rate = Rational::new(1, 4).unwrap();
    *direction = PlaybackDirection::Reverse;
    *repeat = 3;
    source_range_per_repeat.duration = time(50);
    let slow_graph = graph(&slow);
    for marker in [
        "areverse",
        "atempo=0.5,atempo=0.5",
        "asplit=3",
        "concat=n=3:v=0:a=1",
    ] {
        assert!(
            slow_graph.contains(marker),
            "missing {marker}: {slow_graph}"
        );
    }

    let mut fast = audio_plan(2);
    let fast_clip = clip(&mut fast);
    fast_clip.audio.as_mut().unwrap().pitch_policy = PitchPolicy::Preserve;
    let mapping = fast_clip.source_mapping.as_mut().unwrap();
    let ResolvedSourceTimeMap::Linear {
        source_range_per_repeat,
        rate,
        ..
    } = &mut mapping.time_map
    else {
        panic!("linear mapping");
    };
    *rate = Rational::new(200, 1).unwrap();
    source_range_per_repeat.duration = time(120_000);
    fast.inputs[0].probe.as_mut().unwrap().container_duration = Some(time(120_000));
    fast.inputs[0].video.as_mut().unwrap().duration = Some(time(120_000));
    fast.inputs[0].audio.as_mut().unwrap().duration = Some(time(120_000));
    assert!(graph(&fast).contains("atempo=100,atempo=2"));
}

#[test]
fn muted_and_absent_properties_fall_back_to_timeline_silence() {
    let mut muted = audio_plan(2);
    clip(&mut muted).audio.as_mut().unwrap().muted = true;
    assert_silence_only(&muted);

    let mut absent = audio_plan(2);
    let absent_clip = clip(&mut absent);
    absent_clip.audio = None;
    let ResolvedClipSource::Media { audio_stream, .. } = &mut absent_clip.source else {
        unreachable!()
    };
    *audio_stream = None;
    assert_silence_only(&absent);
}

#[test]
fn mono_pan_and_unaligned_record_start_are_typed_errors() {
    let mut mono = audio_plan(1);
    clip(&mut mono).audio.as_mut().unwrap().pan = Animatable::constant(0.5);
    assert_audio_error(&mono, "pan automation requires stereo output");

    let mut unaligned = audio_plan(2);
    unaligned
        .output
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap()
        .audio
        .as_mut()
        .unwrap()
        .sample_rate = 44_100;
    clip(&mut unaligned).record_range.start = time(1);
    unaligned.sequences[0].duration = time(601);
    assert_audio_error(&unaligned, "not aligned to an output audio sample");
}

#[test]
fn generated_silence_and_nested_audio_sequences_are_routed() {
    let mut generated = audio_plan(1);
    let track = &mut generated.sequences[0].tracks[0];
    track.kind = TrackKind::Audio;
    track.state.visual_enabled = false;
    track.routing.visual = None;
    track.routing.audio = Some(ResolvedAudioRoute::MainMix);
    track.clips[0].visual = None;
    track.clips[0].source = ResolvedClipSource::Generated {
        generator: Generator::Silence,
    };
    let generated_graph = graph(&generated);
    assert!(generated_graph.contains("generateda"));
    assert!(generated_graph.contains("anullsrc=r=48000:cl=mono"));

    let mut nested = audio_plan(2);
    let mut child = nested.sequences[0].clone();
    child.id = SequenceId::new("seq_audio_child").unwrap();
    child.tracks[0].id = TrackId::new("trk_audio_child").unwrap();
    child.tracks[0].clips[0].id = ItemId::new("itm_audio_child").unwrap();
    nested.sequences[0].tracks[0].clips[0].source = ResolvedClipSource::Sequence {
        sequence_id: child.id.clone(),
    };
    let mapping = nested.sequences[0].tracks[0].clips[0]
        .source_mapping
        .as_mut()
        .unwrap();
    let ResolvedSourceTimeMap::Linear { direction, .. } = &mut mapping.time_map else {
        panic!("linear mapping");
    };
    *direction = PlaybackDirection::Reverse;
    nested.sequences.insert(0, child);
    let nested_graph = graph(&nested);
    assert!(nested_graph.contains("areverse"));
    assert!(nested_graph.contains("apad=whole_dur=1"));
    assert!(nested_graph.matches("amix=inputs=2").count() >= 2);
}

#[test]
fn nonzero_record_start_uses_sample_accurate_delay() {
    let mut plan = audio_plan(2);
    clip(&mut plan).record_range.start = time(300);
    plan.sequences[0].duration = time(900);
    assert!(graph(&plan).contains("adelay=delays=24000S:all=1"));
}

#[test]
fn named_audio_buses_are_mixed_before_the_main_output() {
    let mut plan = audio_plan(2);
    let mut bus_track = plan.sequences[0].tracks[0].clone();
    bus_track.id = TrackId::new("trk_dialogue").unwrap();
    bus_track.order = 1;
    bus_track.source_order = 1;
    bus_track.clips[0].id = ItemId::new("itm_dialogue").unwrap();
    bus_track.routing.audio = Some(ResolvedAudioRoute::Bus {
        bus_id: "bus_dialogue".to_owned(),
    });
    let mut second_bus_track = bus_track.clone();
    second_bus_track.id = TrackId::new("trk_dialogue_alt").unwrap();
    second_bus_track.order = 2;
    second_bus_track.source_order = 2;
    second_bus_track.clips[0].id = ItemId::new("itm_dialogue_alt").unwrap();
    plan.sequences[0].tracks.push(bus_track);
    plan.sequences[0].tracks.push(second_bus_track);

    let graph = graph(&plan);
    assert_eq!(graph.matches("busmix").count(), 2);
    assert!(graph.contains("amix=inputs=2:normalize=0:dropout_transition=0"));
    assert!(graph.contains("amix=inputs=3:normalize=0:dropout_transition=0"));
}

pub(super) fn audio_plan(channels: u8) -> veac_plan::ResolvedRenderPlan {
    let mut project = fixture();
    project.project.render_configs[0]
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels,
    });
    project.project.sequences[0].tracks[0].clips[0].audio = Some(AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: vec![],
        crossfade: None,
    });
    resolved(&project)
}

pub(super) fn clip(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut veac_plan::ResolvedClip {
    &mut plan.sequences.last_mut().unwrap().tracks[0].clips[0]
}
