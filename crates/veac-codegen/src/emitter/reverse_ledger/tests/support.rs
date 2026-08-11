use crate::unit_tests::emitter_tests::support::bindings;
use veac_artifact::*;
use veac_plan::canonical::*;
use veac_plan::{ResolvedRenderPlan, ResolvedSourceTimeMap};

pub(super) fn reverse(plan: &mut ResolvedRenderPlan) -> &mut veac_plan::ResolvedSourceMapping {
    let mapping = plan.sequences.last_mut().unwrap().tracks[0].clips[0]
        .source_mapping
        .as_mut()
        .unwrap();
    if let ResolvedSourceTimeMap::Linear { direction, .. } = &mut mapping.time_map {
        *direction = PlaybackDirection::Reverse;
    }
    mapping
}

pub(super) fn reverse_clip(clip: &mut veac_plan::ResolvedClip) {
    let mapping = clip.source_mapping.as_mut().unwrap();
    if let ResolvedSourceTimeMap::Linear { direction, .. } = &mut mapping.time_map {
        *direction = PlaybackDirection::Reverse;
    }
}

pub(super) fn set_reverse_span(plan: &mut ResolvedRenderPlan, value: i64) {
    let clip = &mut plan.sequences.last_mut().unwrap().tracks[0].clips[0];
    reverse_clip(clip);
    clip.record_range.duration = time(value);
    if let ResolvedSourceTimeMap::Linear {
        source_range_per_repeat,
        ..
    } = &mut clip.source_mapping.as_mut().unwrap().time_map
    {
        source_range_per_repeat.duration = time(value);
    }
    plan.sequences.last_mut().unwrap().duration = time(value);
}

pub(super) fn segment(start: i64, end: i64) -> SourceTimeSegment {
    SourceTimeSegment {
        record_duration: time(600),
        source_start: time(start),
        source_end: time(end),
        interpolation: SourceTimeInterpolation::Linear,
    }
}

pub(super) fn audio_plan(sample_rate: u32, channels: u8) -> ResolvedRenderPlan {
    let mut project = fixture();
    project.project.render_configs[0].raster = None;
    project.project.render_configs[0].deliverables = vec![Deliverable {
        id: DeliverableId::new("dlv_reverse_audio").unwrap(),
        target: DeliverableTarget::File {
            name: "reverse.wav".into(),
        },
        kind: DeliverableKind::AudioStem(AudioStemOutput {
            format: AudioStemFormat::Wav,
            audio: AudioOutput {
                codec: AudioCodec::PcmS32Le,
                sample_rate,
                channels,
            },
            source: AudioMixSource::Master,
        }),
    }];
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

pub(super) fn proxy_video_bindings(
    plan: &ResolvedRenderPlan,
    width: u32,
    height: u32,
    fps: i64,
) -> ExecutionBindings {
    let input = &plan.inputs[0];
    let duration = input
        .probe
        .as_ref()
        .and_then(|probe| probe.container_duration)
        .unwrap();
    proxy_bindings(
        plan,
        MediaRole::Video,
        ArtifactParameters::ProxyVideo(ProxyVideoSpec {
            source_stream: input.video.as_ref().unwrap().selection,
            source_clock: SourceClockSpec::Identity { duration },
            width,
            height,
            frame_rate: Rational::new(fps, 1).unwrap(),
            crf: 24,
        }),
    )
}

pub(super) fn proxy_audio_bindings(
    plan: &ResolvedRenderPlan,
    sample_rate: u32,
    channels: u8,
) -> ExecutionBindings {
    let input = &plan.inputs[0];
    let duration = input
        .probe
        .as_ref()
        .and_then(|probe| probe.container_duration)
        .unwrap();
    proxy_bindings(
        plan,
        MediaRole::Audio,
        ArtifactParameters::ProxyAudio(ProxyAudioSpec {
            source_stream: input.audio.as_ref().unwrap().selection,
            source_clock: SourceClockSpec::Identity { duration },
            sample_rate,
            channels,
        }),
    )
}

fn proxy_bindings(
    plan: &ResolvedRenderPlan,
    role: MediaRole,
    parameters: ArtifactParameters,
) -> ExecutionBindings {
    let input = &plan.inputs[0];
    let descriptor = ArtifactDescriptor::new(
        ProducerFingerprint {
            name: "reverse-proxy-test".into(),
            version: "1".into(),
            configuration: ContentDigest::sha256(b"reverse-proxy"),
        },
        vec![ArtifactDependency::new(
            ArtifactDependencyRole::Input,
            ContentDigest {
                algorithm: DigestAlgorithm::Sha256,
                value: input.observed_identity.digest.clone(),
            },
        )],
        parameters,
    );
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let record = store.put(&descriptor, b"proxy").unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();
    let mut result = bindings(plan);
    result.bind_verified_proxy(input, role, &artifact).unwrap();
    result
}

pub(super) fn codes(plan: &ResolvedRenderPlan) -> Vec<&'static str> {
    codes_with(plan, &bindings(plan))
}

pub(super) fn codes_with(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
) -> Vec<&'static str> {
    crate::emitter::emit_all(plan, bindings)
        .err()
        .map(|error| {
            error
                .diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.code)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) use crate::unit_tests::emitter_tests::support::{fixture, resolved, time};
