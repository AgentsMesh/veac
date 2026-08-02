use veac_plan::canonical::*;

use super::support::{bindings, emit_video_command, fixture, resolved, time};

#[test]
fn typed_audio_processors_and_per_clip_crossfades_emit_in_authored_order() {
    let mut project = fixture();
    enable_audio(&mut project);
    project.project.sequences[0].tracks[0].clips[0].audio = Some(complete_audio());
    let plan = resolved(&project);
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    let filters = [
        "equalizer=f=1000:t=q:w=1.2:g=3",
        "highpass=f=80:t=q:w=0.707:p=2",
        "lowpass=f=18000:t=q:w=0.707:p=2",
        "acompressor=threshold=0.125892541179:ratio=4",
        "alimiter=limit=0.891250938134",
        "agate=threshold=0.005623413252",
        "loudnorm=I=-16:TP=-1:LRA=7:linear=true",
        "afade=t=in:st=0:d=0.05:curve=qsin",
        "afade=t=out:st=0.95:d=0.05:curve=qsin",
    ];
    let mut previous = 0;
    for filter in filters {
        let index = graph
            .find(filter)
            .unwrap_or_else(|| panic!("missing {filter}: {graph}"));
        assert!(index >= previous, "{filter} emitted out of order");
        previous = index;
    }
}

#[test]
fn sidechain_track_and_bus_sources_emit_two_input_compression() {
    for bus in [false, true] {
        let project = sidechain_project(bus, None);
        let plan = resolved(&project);
        let graph = emit_video_command(&plan, &bindings(&plan))
            .unwrap()
            .filter_graph
            .unwrap();
        assert!(graph.contains("sidechainsourcea"), "{graph}");
        assert!(
            graph.contains("sidechaincompress=threshold=0.063095734448:ratio=4"),
            "{graph}"
        );
        assert!(graph.contains("attack=10:release=250:mix=1"), "{graph}");
    }
}

#[test]
fn partial_sidechain_range_splices_only_the_active_interval() {
    let project = sidechain_project(false, Some(TimeRange::new(time(100), time(300)).unwrap()));
    let plan = resolved(&project);
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    for marker in [
        "asplit=3",
        "atrim=start=0:end=0.166666666667",
        "atrim=start=0.166666666667:end=0.666666666667",
        "sidechaincompress",
        "atrim=duration=0.5,asetpts=PTS-STARTPTS",
        "atrim=start=0.666666666667:end=1",
        "concat=n=3:v=0:a=1",
    ] {
        assert!(graph.contains(marker), "missing {marker}: {graph}");
    }
    assert_eq!(graph.matches("]apad[").count(), 2, "{graph}");
}

fn sidechain_project(bus: bool, active_range: Option<TimeRange>) -> ProjectEnvelope {
    let mut project = fixture();
    enable_audio(&mut project);
    let sequence = &mut project.project.sequences[0];
    let mut source = sequence.tracks[0].clone();
    source.id = TrackId::new("trk_voice").unwrap();
    source.order = 1;
    source.kind = TrackKind::Audio;
    source.clips[0].id = ItemId::new("itm_voice").unwrap();
    source.clips[0].visual = None;
    source.clips[0].audio = Some(base_audio());
    if bus {
        source.routing = TrackRouting::AudioBus {
            bus_id: BusId::new("bus_dialogue").unwrap(),
        };
    }
    sequence.tracks.push(source);
    sequence.tracks[0].clips[0].audio = Some(base_audio());
    let key = if bus {
        RelationEndpoint::bus(BusId::new("bus_dialogue").unwrap())
    } else {
        RelationEndpoint::track(TrackId::new("trk_voice").unwrap())
    };
    project.project.relations.push(Relation {
        id: RelationId::new("rel_audio_processing_sidechain").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Sidechain {
            key,
            target: RelationEndpoint::item(ItemId::new("itm_video").unwrap()),
            parameters: SidechainRelationParameters {
                threshold_db: -24.0,
                ratio: 4.0,
                attack_ms: 10.0,
                release_ms: 250.0,
                active_range,
            },
        },
    });
    project
}

fn complete_audio() -> AudioProperties {
    let mut audio = base_audio();
    audio.processors = vec![
        AudioProcessor::ParametricEq {
            bands: vec![ParametricEqBand {
                frequency_hz: 1_000.0,
                gain_db: 3.0,
                q: 1.2,
            }],
        },
        AudioProcessor::HighPass {
            frequency_hz: 80.0,
            q: 0.707,
            poles: 2,
        },
        AudioProcessor::LowPass {
            frequency_hz: 18_000.0,
            q: 0.707,
            poles: 2,
        },
        AudioProcessor::Compressor(Compressor {
            threshold_db: -18.0,
            ratio: 4.0,
            attack_ms: 10.0,
            release_ms: 200.0,
            knee_db: 6.0,
            makeup_gain_db: 3.0,
            mix: 1.0,
        }),
        AudioProcessor::Limiter(Limiter {
            ceiling_db: -1.0,
            attack_ms: 5.0,
            release_ms: 50.0,
        }),
        AudioProcessor::Gate(Gate {
            threshold_db: -45.0,
            ratio: 2.0,
            attack_ms: 5.0,
            release_ms: 150.0,
            range_db: -80.0,
        }),
        AudioProcessor::Loudness(LoudnessTarget {
            integrated_lufs: -16.0,
            true_peak_dbtp: -1.0,
            loudness_range_lu: 7.0,
        }),
    ];
    audio.crossfade = Some(AudioCrossfade {
        fade_in: time(30),
        fade_out: time(30),
        curve: AudioFadeCurve::EqualPower,
    });
    audio
}

fn base_audio() -> AudioProperties {
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
