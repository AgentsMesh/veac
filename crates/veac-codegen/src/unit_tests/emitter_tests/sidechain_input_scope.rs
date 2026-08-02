use veac_codegen::emitter::{emit_all, BackendAction};
use veac_plan::canonical::*;
use veac_plan::{PlanInputId, ResolvedClipSource, ResolvedSidechain, ResolvedSourceTimeMap};

use super::audio::audio_plan;
use super::support::output_bindings;
use super::support::time;

#[test]
fn selected_track_stem_includes_transitive_sidechain_input() {
    let mut plan = audio_plan(2);
    let entry = plan.sequences.last_mut().unwrap();
    let target_id = entry.tracks[0].id.clone();
    let mut source = entry.tracks[0].clone();
    source.id = TrackId::new("trk_scoped_sidechain").unwrap();
    source.order = 1;
    source.source_order = 1;
    source.clips[0].id = ItemId::new("itm_scoped_sidechain").unwrap();
    let mut source_input = plan.inputs[0].clone();
    source_input.id = PlanInputId::new("pin_scoped_sidechain").unwrap();
    let ResolvedClipSource::Media { input_id, .. } = &mut source.clips[0].source else {
        unreachable!()
    };
    *input_id = source_input.id.clone();
    entry.tracks[0].clips[0].audio.as_mut().unwrap().sidechain = Some(ResolvedSidechain {
        relation_id: RelationId::new("rel_scoped_sidechain").unwrap(),
        source: SidechainSource::Track {
            track_id: source.id.clone(),
        },
        threshold_db: -24.0,
        ratio: 4.0,
        attack_ms: 10.0,
        release_ms: 250.0,
        active_range: None,
    });
    source.clips[0].audio.as_mut().unwrap().sidechain = Some(ResolvedSidechain {
        relation_id: RelationId::new("rel_reciprocal_sidechain").unwrap(),
        source: SidechainSource::Track {
            track_id: target_id.clone(),
        },
        threshold_db: -18.0,
        ratio: 3.0,
        attack_ms: 5.0,
        release_ms: 100.0,
        active_range: None,
    });
    entry.tracks.push(source);
    plan.inputs.push(source_input.clone());
    plan.inputs.sort_by(|left, right| left.id.cmp(&right.id));
    plan.output.deliverables[0].target = DeliverableTarget::File {
        name: "sidechain.wav".to_owned(),
    };
    plan.output.deliverables[0].kind = DeliverableKind::AudioStem(AudioStemOutput {
        format: AudioStemFormat::Wav,
        audio: AudioOutput {
            codec: AudioCodec::PcmS16Le,
            sample_rate: 48_000,
            channels: 2,
        },
        source: AudioMixSource::Track {
            track_id: target_id,
        },
    });
    plan.output.raster = None;
    let mut bindings = output_bindings(&plan);
    let target_input = plan
        .inputs
        .iter()
        .find(|input| input.id != source_input.id)
        .unwrap();
    bindings
        .bind_original(target_input, "/tmp/sidechain-target.mov".into())
        .unwrap();
    let error = emit_all(&plan, &bindings).unwrap_err();
    assert!(error
        .diagnostics()
        .iter()
        .any(|value| value.code == "INPUT_BINDING_MISSING"));

    bindings
        .bind_original(&source_input, "/tmp/sidechain-source.mov".into())
        .unwrap();
    let bundle = emit_all(&plan, &bindings).unwrap();
    assert_eq!(bundle.protected_resources().len(), 2);
    let BackendAction::Ffmpeg(command) = &bundle.tasks()[0].action else {
        panic!("stem must use FFmpeg")
    };
    assert_eq!(command.inputs.len(), 2);
    assert_eq!(
        command
            .filter_graph
            .as_deref()
            .unwrap()
            .matches("sidechaincompress=")
            .count(),
        1,
        "the detector must use the source's pre-sidechain signal"
    );
}

#[test]
fn nonoverlapping_sidechain_source_needs_no_binding_and_emits_no_filter() {
    let mut plan = audio_plan(2);
    let entry = plan.sequences.last_mut().unwrap();
    let target_id = entry.tracks[0].id.clone();
    let mut source = entry.tracks[0].clone();
    source.id = TrackId::new("trk_nonoverlap_key").unwrap();
    source.order = 1;
    source.source_order = 1;
    source.clips[0].id = ItemId::new("itm_nonoverlap_key").unwrap();
    source.clips[0].record_range = TimeRange::new(time(700), time(100)).unwrap();
    let ResolvedSourceTimeMap::Linear {
        source_range_per_repeat,
        ..
    } = &mut source.clips[0].source_mapping.as_mut().unwrap().time_map
    else {
        unreachable!()
    };
    source_range_per_repeat.duration = time(100);
    let mut source_input = plan.inputs[0].clone();
    source_input.id = PlanInputId::new("pin_nonoverlap_key").unwrap();
    let ResolvedClipSource::Media { input_id, .. } = &mut source.clips[0].source else {
        unreachable!()
    };
    *input_id = source_input.id.clone();
    entry.tracks[0].clips[0].audio.as_mut().unwrap().sidechain = Some(ResolvedSidechain {
        relation_id: RelationId::new("rel_nonoverlap_key").unwrap(),
        source: SidechainSource::Track {
            track_id: source.id.clone(),
        },
        threshold_db: -24.0,
        ratio: 4.0,
        attack_ms: 10.0,
        release_ms: 250.0,
        active_range: Some(TimeRange::new(time(0), time(100)).unwrap()),
    });
    entry.tracks.push(source);
    entry.duration = time(800);
    plan.inputs.push(source_input.clone());
    plan.inputs.sort_by(|left, right| left.id.cmp(&right.id));
    plan.output.raster = None;
    plan.output.deliverables[0].target = DeliverableTarget::File {
        name: "nonoverlap.wav".to_owned(),
    };
    plan.output.deliverables[0].kind = DeliverableKind::AudioStem(AudioStemOutput {
        format: AudioStemFormat::Wav,
        audio: AudioOutput {
            codec: AudioCodec::PcmS16Le,
            sample_rate: 48_000,
            channels: 2,
        },
        source: AudioMixSource::Track {
            track_id: target_id,
        },
    });
    let mut local = output_bindings(&plan);
    let target_input = plan
        .inputs
        .iter()
        .find(|input| input.id != source_input.id)
        .unwrap();
    local
        .bind_original(target_input, "/tmp/nonoverlap-target.mov".into())
        .unwrap();

    let bundle = emit_all(&plan, &local).unwrap();
    assert_eq!(bundle.protected_resources().len(), 1);
    let BackendAction::Ffmpeg(command) = &bundle.tasks()[0].action else {
        unreachable!()
    };
    assert_eq!(command.inputs.len(), 1);
    assert!(!command
        .filter_graph
        .as_deref()
        .unwrap()
        .contains("sidechaincompress"));
}
