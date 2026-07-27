use serde_json::json;
use veac_ir::{Rational, RationalTime, StreamSelection};

use crate::{test_support::producer, *};

#[path = "workflow_tests/resource_audio_tests.rs"]
mod resource_audio_tests;
#[path = "workflow_tests/resource_json_tests.rs"]
mod resource_json_tests;
#[path = "workflow_tests/resource_video_tests.rs"]
mod resource_video_tests;
#[path = "workflow_tests/validation_edge_tests.rs"]
mod validation_edge_tests;
#[path = "workflow_tests/validation_tests.rs"]
mod validation_tests;

#[test]
fn every_media_spec_has_a_distinct_complete_descriptor() {
    let specs = vec![
        MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
            source_stream: stream(),
            source_clock: clock(),
            width: 960,
            height: 540,
            frame_rate: Rational::new(30, 1).unwrap(),
            crf: 23,
        }),
        MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream: stream(),
            source_clock: clock(),
            sample_rate: 48_000,
            channels: 2,
        }),
        MediaArtifactSpec::Waveform(WaveformSpec {
            source_stream: stream(),
            source_clock: clock(),
            sample_rate: 48_000,
            width: 800,
            height: 160,
            color: "#40c0ff".into(),
        }),
        MediaArtifactSpec::Thumbnail(ThumbnailSpec {
            source_stream: stream(),
            at: time(90),
            width: 320,
            height: 180,
        }),
        MediaArtifactSpec::OpticalFlow(OpticalFlowSpec {
            source_stream: stream(),
            source_clock: clock(),
            width: 640,
            height: 360,
            frame_rate: Rational::new(60, 1).unwrap(),
            method: OpticalFlowMethod::MotionCompensated,
        }),
        MediaArtifactSpec::Analysis(AnalysisSpec {
            analysis_type: "scene_boundaries".into(),
            configuration: json!({"sensitivity": 0.75}),
        }),
        MediaArtifactSpec::SourceSegment(SourceSegmentSpec {
            video_stream: stream(),
            start: time(10),
            duration: time(50),
            width: 1920,
            height: 1080,
            frame_rate: Rational::new(30, 1).unwrap(),
            audio: Some(SourceSegmentAudioSpec {
                source_stream: stream(),
                sample_rate: 48_000,
                channels: 2,
            }),
            crf: 18,
        }),
    ];
    let extensions = ["mp4", "wav", "png", "png", "mp4", "json", "mp4"];
    let mut keys = Vec::new();
    for (spec, extension) in specs.into_iter().zip(extensions) {
        let request = request(spec.clone());
        let descriptor = request.descriptor().unwrap();
        assert_eq!(descriptor.kind, spec.kind());
        assert_eq!(spec.extension(), extension);
        assert_eq!(descriptor.parameters, serde_json::to_value(spec).unwrap());
        keys.push(artifact_key(&descriptor).unwrap());
    }
    keys.sort_by(|left, right| left.value.cmp(&right.value));
    keys.dedup();
    assert_eq!(keys.len(), 7);
}

#[test]
fn descriptor_key_binds_source_producer_and_parameters() {
    let base = request(MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
        source_stream: stream(),
        source_clock: clock(),
        sample_rate: 48_000,
        channels: 2,
    }));
    let mut source = base.clone();
    source.source_identity = ContentDigest::sha256(b"other-source");
    let mut producer_changed = base.clone();
    producer_changed.producer.version = "2".into();
    let mut parameters = base.clone();
    parameters.spec = MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
        source_stream: stream(),
        source_clock: clock(),
        sample_rate: 44_100,
        channels: 2,
    });
    let keys = [base, source, producer_changed, parameters]
        .map(|value| artifact_key(&value.descriptor().unwrap()).unwrap().value);
    assert!(keys
        .iter()
        .enumerate()
        .all(|(index, key)| { keys.iter().skip(index + 1).all(|other| other != key) }));
}

#[test]
fn exact_proxy_selection_hits_misses_and_rejects_wrong_binding() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let source = ContentDigest::sha256(b"source");
    let empty = ProxySelectionRequest {
        source_identity: source.clone(),
        video: None,
        audio: None,
    };
    assert_eq!(
        select_proxy(&store, &empty).unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
    let video = request_for(
        source.clone(),
        MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
            source_stream: stream(),
            source_clock: clock(),
            width: 640,
            height: 360,
            frame_rate: Rational::new(24, 1).unwrap(),
            crf: 24,
        }),
    )
    .descriptor()
    .unwrap();
    let selection = ProxySelectionRequest {
        source_identity: source.clone(),
        video: Some(video.clone()),
        audio: None,
    };
    assert!(select_proxy(&store, &selection).unwrap().video.is_none());
    let record = store.put(&video, b"proxy").unwrap();
    assert_eq!(
        select_proxy(&store, &selection)
            .unwrap()
            .video
            .unwrap()
            .record(),
        &record
    );
    let mut wrong = selection;
    wrong.source_identity = ContentDigest::sha256(b"wrong");
    assert_eq!(
        select_proxy(&store, &wrong).unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
}

fn request(spec: MediaArtifactSpec) -> MediaArtifactRequest {
    request_for(ContentDigest::sha256(b"source"), spec)
}

fn request_for(source_identity: ContentDigest, spec: MediaArtifactSpec) -> MediaArtifactRequest {
    MediaArtifactRequest {
        source_identity,
        producer: producer(),
        spec,
    }
}

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 10).unwrap()
}

fn stream() -> StreamSelection {
    StreamSelection {
        global_index: 0,
        type_index: 0,
    }
}

fn clock() -> SourceClockSpec {
    SourceClockSpec::Identity { duration: time(10) }
}
