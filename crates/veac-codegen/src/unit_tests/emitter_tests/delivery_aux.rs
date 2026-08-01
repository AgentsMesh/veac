use veac_codegen::emitter::{emit_all, BackendAction, BackendProduct};
use veac_plan::canonical::*;

use super::support::{bindings, fixture, resolved};

#[test]
fn every_scope_uses_a_real_ffmpeg_filter_and_declared_size() {
    let mut plan = resolved(&fixture());
    plan.output.deliverables = [
        (
            "dlv_histogram",
            "histogram.jpg",
            VideoScope::Histogram,
            ImageFormat::Jpeg,
        ),
        (
            "dlv_vectors",
            "vectors.png",
            VideoScope::Vectorscope,
            ImageFormat::Png,
        ),
        (
            "dlv_waveform",
            "waveform.png",
            VideoScope::Waveform,
            ImageFormat::Png,
        ),
    ]
    .into_iter()
    .map(|(id, file, scope, format)| Deliverable {
        id: DeliverableId::new(id).unwrap(),
        target: DeliverableTarget::File {
            name: file.to_owned(),
        },
        kind: DeliverableKind::Scope(ScopeOutput {
            scope,
            at: RationalTime::new(300, 600).unwrap(),
            width: 320,
            height: 180,
            format,
        }),
    })
    .collect();
    let bundle = emit_all(&plan, &all_bindings(&plan)).unwrap();
    for (id, product, filter) in [
        (
            "dlv_histogram",
            BackendProduct::Histogram,
            "histogram=display_mode=overlay:level_height=256",
        ),
        (
            "dlv_vectors",
            BackendProduct::Vectorscope,
            "vectorscope=mode=color4:graticule=color",
        ),
        (
            "dlv_waveform",
            BackendProduct::VideoWaveform,
            "waveform=mode=column:components=7:display=overlay",
        ),
    ] {
        let task = bundle
            .tasks()
            .iter()
            .find(|task| task.deliverable_id.as_str() == id)
            .unwrap();
        assert_eq!(task.product, product);
        let graph = command(task).filter_graph.as_deref().unwrap();
        assert!(graph.contains(filter), "missing {filter}: {graph}");
        assert!(graph.contains("scale=320:180"));
        assert!(pair(&command(task).output_args, "-frames:v", "1"));
    }
}

#[test]
fn master_track_and_bus_stems_keep_their_typed_audio_contracts() {
    let mut project = fixture();
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    clip.audio = Some(AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: vec![],
        crossfade: None,
    });
    project.project.sequences[0].tracks[0].routing = TrackRouting::AudioBus {
        bus_id: BusId::new("bus_dialogue").unwrap(),
    };
    project.project.render_configs[0].deliverables = vec![
        stem(
            "dlv_bus",
            "bus.flac",
            AudioMixSource::Bus {
                bus_id: BusId::new("bus_dialogue").unwrap(),
            },
            true,
        ),
        stem("dlv_master", "master.wav", AudioMixSource::Master, false),
        stem(
            "dlv_track",
            "track.wav",
            AudioMixSource::Track {
                track_id: TrackId::new("trk_video").unwrap(),
            },
            false,
        ),
    ];
    project.project.render_configs[0].raster = None;
    let plan = resolved(&project);
    let bundle = emit_all(&plan, &all_bindings(&plan)).unwrap();
    assert_eq!(bundle.tasks().len(), 3);
    for task in bundle.tasks() {
        assert_eq!(task.product, BackendProduct::AudioStem);
        let command = command(task);
        let graph = command.filter_graph.as_deref().unwrap();
        assert!(
            graph.contains("[0:5]atrim="),
            "missing selected audio: {graph}"
        );
        assert!(pair(&command.output_args, "-ar", "48000"));
        assert!(pair(&command.output_args, "-ac", "2"));
        let scoped: Vec<_> = bundle
            .requirements()
            .iter()
            .filter(|value| value.deliverable_id() == &task.deliverable_id)
            .collect();
        assert!(scoped
            .iter()
            .any(|value| value.kind().as_str() == "decoder" && value.name() == "aac"));
        assert!(!scoped
            .iter()
            .any(|value| value.kind().as_str() == "decoder" && value.name() == "h264"));
    }
    let bus = bundle
        .tasks()
        .iter()
        .find(|task| task.deliverable_id.as_str() == "dlv_bus")
        .unwrap();
    assert!(pair(&command(bus).output_args, "-c:a", "flac"));
    assert!(pair(&command(bus).output_args, "-f", "flac"));
}

fn stem(id: &str, file: &str, source: AudioMixSource, flac: bool) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        target: DeliverableTarget::File {
            name: file.to_owned(),
        },
        kind: DeliverableKind::AudioStem(AudioStemOutput {
            format: if flac {
                AudioStemFormat::Flac
            } else {
                AudioStemFormat::Wav
            },
            audio: AudioOutput {
                codec: if flac {
                    AudioCodec::Flac
                } else {
                    AudioCodec::PcmS16Le
                },
                sample_rate: 48_000,
                channels: 2,
            },
            source,
        }),
    }
}

fn all_bindings(plan: &veac_plan::ResolvedRenderPlan) -> veac_artifact::ExecutionBindings {
    bindings(plan)
}

fn command(task: &veac_codegen::emitter::BackendTask) -> &veac_codegen::emitter::BackendCommand {
    let BackendAction::Ffmpeg(command) = &task.action else {
        panic!()
    };
    command
}

fn pair(arguments: &[String], name: &str, value: &str) -> bool {
    arguments.windows(2).any(|pair| pair == [name, value])
}
