use veac_codegen::emitter::CodegenErrorKind;
use veac_plan::canonical::*;
use veac_plan::ResolvedClipSource;

use super::support::{add_transition, bindings, emit_video_command, fixture, resolved, time};

#[test]
fn a_single_reverse_curve_segment_emits_exact_video_and_audio_time_maps() {
    let mut project = fixture();
    enable_audio(&mut project);
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    clip.source_mapping = Some(SourceMapping {
        time_map: SourceTimeMap::Curve {
            segments: vec![SourceTimeSegment {
                record_duration: time(600),
                source_start: time(600),
                source_end: time(0),
                interpolation: SourceTimeInterpolation::Linear,
            }],
        },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: SourceOutOfRangePolicy::Strict,
    });
    clip.audio = Some(audio());
    let plan = resolved(&project);
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    for marker in ["rampreversev", "rampreversea", "atempo=1"] {
        assert!(graph.contains(marker), "missing {marker}: {graph}");
    }
    assert!(!graph.contains("concat=n=1"));
}

#[test]
fn a_sidechain_source_inherits_its_authored_transition_fades() {
    let project = sidechain_project();
    let plan = resolved(&project);
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    for marker in [
        "afade=t=out:st=0.9:d=0.1",
        "afade=t=in:st=0:d=0.1",
        "sidechainsource",
        "sidechaincompress",
    ] {
        assert!(graph.contains(marker), "missing {marker}: {graph}");
    }
}

#[test]
fn a_selected_sidechain_track_that_produces_no_audio_is_typed_error() {
    let mut plan = resolved(&sidechain_project());
    let source = plan.sequences[0]
        .tracks
        .iter_mut()
        .find(|track| track.id.as_str() == "trk_voice")
        .unwrap();
    for clip in &mut source.clips {
        clip.source = ResolvedClipSource::Generated {
            generator: Generator::Transparent,
        };
    }
    let error = emit_video_command(&plan, &bindings(&plan)).unwrap_err();
    assert!(error.diagnostics().iter().any(|value| {
        value.kind == CodegenErrorKind::InvalidPlan && value.code == "PLAN_STRUCTURE_INVALID"
    }));
}

fn sidechain_project() -> ProjectEnvelope {
    let mut project = fixture();
    enable_audio(&mut project);
    let sequence = &mut project.project.sequences[0];
    let mut source = sequence.tracks[0].clone();
    source.id = TrackId::new("trk_voice").unwrap();
    source.kind = TrackKind::Audio;
    source.order = 1;
    source.clips[0].id = ItemId::new("itm_voice_out").unwrap();
    source.clips[0].visual = None;
    source.clips[0].audio = Some(audio());
    let transition = Transition {
        kind: TransitionKind::Dissolve,
        duration: time(120),
        alignment: TransitionAlignment::Centered,
    };
    let mut incoming = source.clips[0].clone();
    incoming.id = ItemId::new("itm_voice_in").unwrap();
    incoming.record_range.start = time(600);
    source.clips.push(incoming);
    sequence.tracks.push(source);
    sequence.tracks[0].clips[0].audio = Some(audio());
    project.project.relations.push(Relation {
        id: RelationId::new("rel_transition_sidechain").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Sidechain {
            key: RelationEndpoint::track(TrackId::new("trk_voice").unwrap()),
            target: RelationEndpoint::item(ItemId::new("itm_video").unwrap()),
            parameters: SidechainRelationParameters {
                threshold_db: -24.0,
                ratio: 4.0,
                attack_ms: 10.0,
                release_ms: 250.0,
                active_range: None,
            },
        },
    });
    add_transition(
        &mut project,
        "seq_main",
        "itm_voice_out",
        "itm_voice_in",
        transition,
    );
    project
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

fn enable_audio(project: &mut ProjectEnvelope) {
    project.project.render_configs[0]
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
}
