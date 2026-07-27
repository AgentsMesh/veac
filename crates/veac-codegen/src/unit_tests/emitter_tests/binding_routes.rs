use veac_artifact::*;
use veac_codegen::emitter::{emit_all, BackendAction, BackendBundle, BackendProduct};
use veac_plan::canonical::*;
use veac_plan::ResolvedRenderPlan;

use super::support::{bindings, fixture, resolved, time};

#[test]
fn split_video_and_audio_proxies_use_distinct_physical_inputs_and_streams() {
    let plan = av_plan();
    let input = &plan.inputs[0];
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let clock = SourceClock::bounded(range(0, 600), time(0)).unwrap();
    let video = proxy(
        &store,
        input,
        ArtifactKind::ProxyVideo,
        b"video proxy",
        clock,
    );
    let audio = proxy(
        &store,
        input,
        ArtifactKind::ProxyAudio,
        b"audio proxy",
        clock,
    );
    let mut local = typed(&plan);
    local
        .bind_verified_proxy(input, MediaRole::Video, &video)
        .unwrap();
    local
        .bind_verified_proxy(input, MediaRole::Audio, &audio)
        .unwrap();
    let bundle = emit_all(&plan, &local).unwrap();
    let command = video_command(&bundle);
    assert_eq!(command.inputs.len(), 2);
    assert_eq!(command.inputs[0].path, video.payload_path());
    assert_eq!(command.inputs[1].path, audio.payload_path());
    let graph = command.filter_graph.as_deref().unwrap();
    assert!(graph.contains("[0:0]trim="), "{graph}");
    assert!(graph.contains("[1:0]atrim="), "{graph}");
    assert!(graph.contains("trim=start=0:duration=1"), "{graph}");
    assert!(graph.contains("atrim=start=0:duration=1"), "{graph}");
    assert_eq!(bundle.protected_resources().len(), 2);
    assert_eq!(bundle.substitution_proof(), &local.substitution_proof());
}

#[test]
fn an_original_av_file_is_deduplicated_to_one_physical_input() {
    let plan = av_plan();
    let local = typed(&plan);
    let bundle = emit_all(&plan, &local).unwrap();
    let command = video_command(&bundle);
    assert_eq!(command.inputs.len(), 1);
    let graph = command.filter_graph.as_deref().unwrap();
    assert!(graph.contains("[0:2]trim="), "{graph}");
    assert!(graph.contains("[0:5]atrim="), "{graph}");
}

#[test]
fn a_missing_required_audio_role_fails_before_command_emission() {
    let plan = av_plan();
    let input = &plan.inputs[0];
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let video = proxy(
        &store,
        input,
        ArtifactKind::ProxyVideo,
        b"video proxy",
        SourceClock::identity(600).unwrap(),
    );
    let mut local = ExecutionBindings::default();
    local
        .bind_verified_proxy(input, MediaRole::Video, &video)
        .unwrap();
    let deliverable = plan.output.deliverables[0].id.clone();
    local
        .bind_output(deliverable, "/tmp/out.mp4".into())
        .unwrap();
    let error = emit_all(&plan, &local).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "INPUT_BINDING_MISSING");
    assert!(error.diagnostics()[0].message.contains("audio"));
}

pub(super) fn av_plan() -> ResolvedRenderPlan {
    let mut project = fixture();
    project.project.render_configs[0]
        .video_deliverable_mut()
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
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

pub(super) fn typed(plan: &ResolvedRenderPlan) -> ExecutionBindings {
    bindings(plan)
}

pub(super) fn proxy(
    store: &ArtifactStore,
    input: &veac_plan::ResolvedInput,
    kind: ArtifactKind,
    payload: &[u8],
    clock: SourceClock,
) -> VerifiedArtifact {
    let source_stream = match kind {
        ArtifactKind::ProxyVideo => input.video.as_ref().unwrap().selection,
        ArtifactKind::ProxyAudio => input.audio.as_ref().unwrap().selection,
        _ => unreachable!(),
    };
    let source_clock = match clock.logical_range() {
        Some(logical_range) => SourceClockSpec::Bounded { logical_range },
        None => SourceClockSpec::Identity {
            duration: input
                .probe
                .as_ref()
                .and_then(|value| value.container_duration)
                .unwrap_or_else(|| RationalTime::new(600, clock.timescale()).unwrap()),
        },
    };
    let spec = match kind {
        ArtifactKind::ProxyVideo => MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
            source_stream,
            source_clock,
            width: 640,
            height: 360,
            frame_rate: Rational::new(30, 1).unwrap(),
            crf: 24,
        }),
        ArtifactKind::ProxyAudio => MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream,
            source_clock,
            sample_rate: 48_000,
            channels: 2,
        }),
        _ => unreachable!(),
    };
    let descriptor = ArtifactDescriptor::new(
        kind,
        ProducerFingerprint {
            name: "proxy-test".into(),
            version: "1".into(),
            configuration: ContentDigest::sha256(b"proxy-test-config"),
        },
        vec![ArtifactDependency {
            role: "input".into(),
            identity: ContentDigest {
                algorithm: DigestAlgorithm::Sha256,
                value: input.observed_identity.digest.clone(),
            },
        }],
        serde_json::to_value(spec).unwrap(),
    );
    let record = store.put(&descriptor, payload).unwrap();
    store
        .open_verified(&record.key, &descriptor)
        .unwrap()
        .unwrap()
}

pub(super) fn video_command(bundle: &BackendBundle) -> &veac_codegen::emitter::BackendCommand {
    bundle
        .tasks()
        .iter()
        .find_map(|task| match (&task.action, task.product) {
            (BackendAction::Ffmpeg(command), BackendProduct::VideoMaster) => Some(command),
            _ => None,
        })
        .unwrap()
}

pub(super) fn range(start: i64, duration: i64) -> TimeRange {
    TimeRange::new(time(start), time(duration)).unwrap()
}
