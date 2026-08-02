use std::time::Duration;

use tempfile::tempdir;
use veac_artifact::{ContentDigest, DigestAlgorithm};
use veac_ir::{
    AudioCodec, AudioMixSource, AudioOutput, AudioStemFormat, AudioStemOutput, AudioStreamInfo,
    CaptionSidecarFormat, CaptionSidecarOutput, Deliverable, DeliverableId, DeliverableKind,
    DeliverableTarget, HashAlgorithm, StreamDisposition, StreamSelection,
};
use veac_plan::ResolvedAudioStream;

use super::*;
use crate::environment::Environment;
use crate::unit_tests::support::{canonical_project, FakeEnvironment, MEDIA_SOURCE};

#[test]
fn expired_prefer_proxy_selection_never_falls_through_to_ffmpeg() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("clip.mp4"), b"source").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let environment = FakeEnvironment::success();
    let mut prepared = crate::planning::prepare(&project, None, &environment).unwrap();
    let producer = super::super::producer(environment.ffmpeg_fingerprint().unwrap()).unwrap();
    let error = apply(
        &mut prepared,
        &veac_artifact::ArtifactStore::new(temp.path().join("store")),
        SubstitutionPolicy::Prefer,
        Some(&producer),
        &environment,
        Instant::now(),
    )
    .unwrap_err();
    assert!(error.is_resource_limit());
    assert!(environment.executed.borrow().is_empty());
}

#[test]
fn audio_only_inputs_build_an_exact_proxy_request() {
    let temp = tempdir().unwrap();
    let media = temp.path().join("clip.mp4");
    std::fs::write(&media, b"audio proxy source").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut environment = FakeEnvironment::success();
    environment.observed = veac_runtime::asset::sha256_identity(&media).unwrap();
    let mut prepared = crate::planning::prepare(&project, None, &environment).unwrap();
    prepared.plan.output.raster = None;
    prepared.plan.output.deliverables = vec![audio_stem()];
    add_audio(&mut prepared.plan);
    prepared.plan.inputs[0].video = None;
    let producer = super::super::producer(environment.ffmpeg_fingerprint().unwrap()).unwrap();

    let error = apply(
        &mut prepared,
        &veac_artifact::ArtifactStore::new(temp.path().join("store")),
        SubstitutionPolicy::Require,
        Some(&producer),
        &environment,
        Instant::now() + Duration::from_secs(2),
    )
    .unwrap_err();
    assert!(error.to_string().contains("PROXY_REQUIRED_MISSING"));
}

#[test]
fn caption_only_delivery_is_a_proxy_noop_before_deadline_and_identity_checks() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("clip.mp4"), b"source").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let environment = FakeEnvironment::success();
    let mut prepared = crate::planning::prepare(&project, None, &environment).unwrap();
    prepared.plan.output.raster = None;
    prepared.plan.output.deliverables = vec![caption_sidecar()];
    prepared.plan.inputs[0].observed_identity.algorithm = HashAlgorithm::Blake3;
    let original = prepared.bindings.inputs().len();

    apply(
        &mut prepared,
        &veac_artifact::ArtifactStore::new(temp.path().join("store")),
        SubstitutionPolicy::Require,
        None,
        &environment,
        Instant::now(),
    )
    .unwrap();
    assert_eq!(prepared.bindings.inputs().len(), original);
    assert!(environment.executed.borrow().is_empty());
}

#[test]
fn proxy_selection_rejects_non_sha_sources_and_maps_artifact_errors() {
    let temp = tempdir().unwrap();
    let media = temp.path().join("clip.mp4");
    std::fs::write(&media, b"proxy source").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut environment = FakeEnvironment::success();
    environment.observed = veac_runtime::asset::sha256_identity(&media).unwrap();
    let mut prepared = crate::planning::prepare(&project, None, &environment).unwrap();
    prepared.plan.inputs[0].observed_identity.algorithm = HashAlgorithm::Blake3;
    let producer = super::super::producer(environment.ffmpeg_fingerprint().unwrap()).unwrap();
    let error = apply(
        &mut prepared,
        &veac_artifact::ArtifactStore::new(temp.path().join("store")),
        SubstitutionPolicy::Prefer,
        Some(&producer),
        &environment,
        Instant::now() + Duration::from_secs(2),
    )
    .unwrap_err();
    assert!(error.to_string().contains("PROXY_IDENTITY_UNSUPPORTED"));

    let invalid = ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: "invalid".into(),
    }
    .validate()
    .unwrap_err();
    assert!(artifact_error(invalid)
        .to_string()
        .contains("PROXY_SELECTION_FAILED"));
}

fn add_audio(plan: &mut veac_plan::ResolvedRenderPlan) {
    let duration = plan.inputs[0]
        .probe
        .as_ref()
        .and_then(|probe| probe.container_duration);
    plan.inputs[0].audio = Some(ResolvedAudioStream {
        selection: StreamSelection {
            global_index: 1,
            type_index: 0,
        },
        codec: "aac".into(),
        start_time: None,
        duration,
        disposition: StreamDisposition {
            default: true,
            attached_picture: false,
            timed_thumbnail: false,
        },
        info: AudioStreamInfo {
            sample_rate: 48_000,
            channels: 2,
            channel_layout: "stereo".into(),
        },
    });
}

fn audio_stem() -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_stem").unwrap(),
        target: DeliverableTarget::File {
            name: "stem.wav".into(),
        },
        kind: DeliverableKind::AudioStem(AudioStemOutput {
            format: AudioStemFormat::Wav,
            audio: AudioOutput {
                codec: AudioCodec::PcmS16Le,
                sample_rate: 48_000,
                channels: 2,
            },
            source: AudioMixSource::Master,
        }),
    }
}

fn caption_sidecar() -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_caption").unwrap(),
        target: DeliverableTarget::File {
            name: "captions.srt".into(),
        },
        kind: DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
            format: CaptionSidecarFormat::Srt,
            track_ids: vec![],
        }),
    }
}
