mod support;

use std::error::Error;

use veac_artifact::ContentDigest;
use veac_evidence::*;
use veac_ir::{HashAlgorithm, MediaIdentity};
use veac_runtime::{
    executor::{FfmpegFingerprint, SystemFfmpeg},
    observation::{
        DecodeRequest, FramePixelFormat, FrameRequest, MediaObserver, ObservationLimits,
        ObservationSource,
    },
};

#[test]
fn json_contract_distinguishes_syntax_and_semantic_failures() {
    let suite = support::suite();
    let json = serde_json::to_string(&suite).unwrap();
    assert_eq!(
        decode_evidence_suite_json(&json).unwrap().as_suite(),
        &suite
    );

    let syntax = decode_evidence_suite_json("{").unwrap_err();
    assert!(syntax.to_string().contains("evidence JSON is invalid"));
    assert!(syntax.source().is_some());

    let mut invalid = suite;
    invalid.schema_version += 1;
    let semantic =
        decode_evidence_suite_json(&serde_json::to_string(&invalid).unwrap()).unwrap_err();
    assert!(matches!(semantic, EvidenceContractError::Validation(_)));
    assert!(!semantic.to_string().is_empty());
    assert!(semantic.source().is_some());
}

#[test]
fn every_public_evidence_schema_is_serializable() {
    for schema in [
        evidence_suite_json_schema().unwrap(),
        observation_plan_json_schema().unwrap(),
        evidence_report_json_schema().unwrap(),
        evidence_bundle_json_schema().unwrap(),
        evidence_provenance_json_schema().unwrap(),
    ] {
        assert!(schema.get("title").is_some());
    }
    let io = BundleError::from(std::io::Error::other("disk"));
    assert!(io.to_string().contains("evidence bundle I/O failed"));
}

#[test]
fn runtime_adapter_and_ffmpeg_provenance_keep_typed_failures() {
    let fingerprint = FfmpegFingerprint {
        version: "fixture".into(),
        configuration: ContentDigest::sha256(b"configuration"),
    };
    let producer = EvidenceProducerFingerprint::from(fingerprint);
    assert_eq!(producer.engine, "ffmpeg");
    assert_eq!(producer.configuration_sha256.len(), 64);

    let observer = MediaObserver::new(
        SystemFfmpeg::new("missing-ffmpeg"),
        ObservationLimits::default(),
    )
    .unwrap();
    let source = ObservationSource {
        path: "missing-video.mp4".into(),
        identity: MediaIdentity {
            algorithm: HashAlgorithm::Sha256,
            digest: "0".repeat(64),
        },
        video_stream: None,
    };
    let frame = FrameRequest {
        source: source.clone(),
        time: RationalTime::new(-1, 1).unwrap(),
        pixel_format: FramePixelFormat::Rgba8,
    };
    let error = ObservationBackend::frame(&observer, &frame).unwrap_err();
    assert!(error.contains("non-negative"));
    let error = ObservationBackend::decode(&observer, &DecodeRequest { source }).unwrap_err();
    assert!(error.contains("source changed"));
}
