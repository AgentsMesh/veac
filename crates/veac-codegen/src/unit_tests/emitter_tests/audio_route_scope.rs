use veac_codegen::emitter::emit_all;
use veac_plan::canonical::*;
use veac_plan::{ResolvedAudioRoute, ResolvedClipSource, ResolvedSidechain};

use super::audio::audio_plan;
use super::support::{bindings, time};

#[test]
fn track_stem_ignores_unrouted_pan_rate_and_alignment_contracts() {
    let mut pan = two_track_plan(1, 48_000);
    second(&mut pan).audio.as_mut().unwrap().pan = Animatable::constant(0.5);
    assert!(emit_all(&pan, &bindings(&pan)).is_ok());
    stem_source(&mut pan, AudioStemSource::Master);
    assert_code(&pan, "PLAN_AUDIO_CHANNEL_LAYOUT_INVALID");

    let mut rate = two_track_plan(2, 48_000);
    let clip = second(&mut rate);
    clip.audio.as_mut().unwrap().pitch_policy = PitchPolicy::FollowSpeed;
    let mapping = clip.source_mapping.as_mut().unwrap();
    let veac_plan::ResolvedSourceTimeMap::Linear {
        rate: speed,
        source_range_per_repeat,
        ..
    } = &mut mapping.time_map
    else {
        unreachable!()
    };
    *speed = Rational::new(100_000, 1).unwrap();
    source_range_per_repeat.duration = time(60_000_000);
    rate.inputs[0].probe.as_mut().unwrap().container_duration = Some(time(60_000_000));
    rate.inputs[0].video.as_mut().unwrap().duration = Some(time(60_000_000));
    rate.inputs[0].audio.as_mut().unwrap().duration = Some(time(60_000_000));
    let emitted = emit_all(&rate, &bindings(&rate));
    assert!(emitted.is_ok(), "{emitted:?}");
    stem_source(&mut rate, AudioStemSource::Master);
    assert_code(&rate, "PLAN_AUDIO_RATE_INVALID");

    let mut alignment = two_track_plan(2, 44_100);
    second(&mut alignment).record_range.start = time(1);
    alignment.sequences.last_mut().unwrap().duration = time(601);
    assert!(emit_all(&alignment, &bindings(&alignment)).is_ok());
    stem_source(&mut alignment, AudioStemSource::Master);
    assert_code(&alignment, "PLAN_AUDIO_SAMPLE_ALIGNMENT_INVALID");
}

#[test]
fn bus_sidechain_and_nested_audio_expand_the_consumer_closure() {
    let mut bus = two_track_plan(1, 48_000);
    let source_id = bus.sequences.last().unwrap().tracks[1].id.clone();
    bus.sequences.last_mut().unwrap().tracks[1].routing.audio = Some(ResolvedAudioRoute::Bus {
        bus_id: "bus_dialogue".to_owned(),
    });
    second(&mut bus).audio.as_mut().unwrap().pan = Animatable::constant(0.5);
    stem_source(
        &mut bus,
        AudioStemSource::Bus {
            bus_id: BusId::new("bus_dialogue").unwrap(),
        },
    );
    assert_code(&bus, "PLAN_AUDIO_CHANNEL_LAYOUT_INVALID");

    stem_source(
        &mut bus,
        AudioStemSource::Track {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    );
    let target = &mut bus.sequences.last_mut().unwrap().tracks[0].clips[0];
    target.audio.as_mut().unwrap().sidechain = Some(ResolvedSidechain {
        relation_id: RelationId::new("rel_route_scope_sidechain").unwrap(),
        source: SidechainSource::Track {
            track_id: source_id,
        },
        threshold_db: -24.0,
        ratio: 4.0,
        attack_ms: 10.0,
        release_ms: 250.0,
        active_range: None,
    });
    assert_code(&bus, "PLAN_AUDIO_CHANNEL_LAYOUT_INVALID");

    let mut nested = nested_plan();
    assert!(emit_all(&nested, &bindings(&nested)).is_ok());
    stem_source(
        &mut nested,
        AudioStemSource::Track {
            track_id: TrackId::new("trk_nested_owner").unwrap(),
        },
    );
    assert_code(&nested, "PLAN_AUDIO_CHANNEL_LAYOUT_INVALID");
}

fn two_track_plan(channels: u8, sample_rate: u32) -> veac_plan::ResolvedRenderPlan {
    let mut plan = audio_plan(channels);
    let sequence = plan.sequences.last_mut().unwrap();
    let mut extra = sequence.tracks[0].clone();
    extra.id = TrackId::new("trk_unrelated").unwrap();
    extra.order = 1;
    extra.source_order = 1;
    extra.clips[0].id = ItemId::new("itm_unrelated").unwrap();
    sequence.tracks.push(extra);
    plan.output.deliverables[0].file_name = "isolated.wav".to_owned();
    plan.output.deliverables[0].kind = DeliverableKind::AudioStem(AudioStemOutput {
        format: AudioStemFormat::Wav,
        audio: AudioOutput {
            codec: AudioCodec::PcmS16Le,
            sample_rate,
            channels,
        },
        source: AudioStemSource::Track {
            track_id: sequence.tracks[0].id.clone(),
        },
    });
    plan
}

fn nested_plan() -> veac_plan::ResolvedRenderPlan {
    let mut plan = two_track_plan(1, 48_000);
    let mut child = plan.sequences.last().unwrap().clone();
    child.id = SequenceId::new("seq_scoped_child").unwrap();
    child.name = "Scoped child".to_owned();
    child.tracks.truncate(1);
    child.tracks[0].id = TrackId::new("trk_scoped_child").unwrap();
    child.tracks[0].clips[0].id = ItemId::new("itm_scoped_child").unwrap();
    child.tracks[0].clips[0].audio.as_mut().unwrap().pan = Animatable::constant(0.5);
    let owner = &mut plan.sequences.last_mut().unwrap().tracks[1];
    owner.id = TrackId::new("trk_nested_owner").unwrap();
    owner.clips[0].id = ItemId::new("itm_nested_owner").unwrap();
    owner.clips[0].source = ResolvedClipSource::Sequence {
        sequence_id: child.id.clone(),
    };
    plan.sequences.insert(0, child);
    plan
}

fn second(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut veac_plan::ResolvedClip {
    &mut plan.sequences.last_mut().unwrap().tracks[1].clips[0]
}

fn stem_source(plan: &mut veac_plan::ResolvedRenderPlan, source: AudioStemSource) {
    let DeliverableKind::AudioStem(stem) = &mut plan.output.deliverables[0].kind else {
        unreachable!()
    };
    stem.source = source;
}

fn assert_code(plan: &veac_plan::ResolvedRenderPlan, code: &str) {
    let error = emit_all(plan, &bindings(plan)).unwrap_err();
    assert!(error.diagnostics().iter().any(|value| value.code == code));
}
