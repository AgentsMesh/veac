use std::time::{Duration, Instant};

use veac_artifact::{AnalysisSpec, MediaArtifactLimits, MediaArtifactSpec, SourceClockSpec};
use veac_ir::{ProbedStreamType, RationalTime};

use super::test_support::*;
use super::*;

#[test]
fn every_ffmpeg_artifact_spec_passes_source_preflight() {
    let temp = tempfile::tempdir().unwrap();
    let bytes = b"source";
    let source = temp.path().join("source.mp4");
    std::fs::write(&source, bytes).unwrap();
    let tool = SystemFfprobe::new(ffprobe(temp.path()));
    for spec in specs() {
        let result = validate(
            &tool,
            &source,
            &request(bytes, spec),
            MediaArtifactLimits::default(),
            deadline(),
        );
        if let Err(error) = result {
            panic!("{error}");
        }
    }
    let analysis = MediaArtifactSpec::Analysis(AnalysisSpec {
        analysis_type: "scene".into(),
        configuration: serde_json::json!({}),
    });
    let error = validate(
        &tool,
        &source,
        &request(bytes, analysis),
        MediaArtifactLimits::default(),
        deadline(),
    )
    .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::InvalidContract);
}

#[test]
fn preflight_rejects_identity_tool_and_deadline_failures() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.mp4");
    std::fs::write(&source, b"source").unwrap();
    let mut wrong = request(b"wrong", specs().remove(0));
    let tool = SystemFfprobe::new(ffprobe(temp.path()));
    assert_eq!(
        validate(
            &tool,
            &source,
            &wrong,
            MediaArtifactLimits::default(),
            deadline()
        )
        .unwrap_err()
        .kind,
        WorkflowErrorKind::InvalidContract
    );
    wrong.source_identity = veac_artifact::ContentDigest::sha256(b"source");
    assert_eq!(
        validate(
            &tool,
            &source,
            &wrong,
            MediaArtifactLimits::default(),
            Instant::now(),
        )
        .unwrap_err()
        .kind,
        WorkflowErrorKind::ResourceLimit
    );
    let missing = SystemFfprobe::new(temp.path().join("missing"));
    assert_eq!(
        validate(
            &missing,
            &source,
            &wrong,
            MediaArtifactLimits::default(),
            deadline(),
        )
        .unwrap_err()
        .kind,
        WorkflowErrorKind::ToolFailure
    );
}

#[test]
fn clock_selection_and_probe_errors_fail_closed() {
    let identity = SourceClockSpec::Identity {
        duration: time(100),
    };
    assert_eq!(clock_range(identity).unwrap(), (time(0), time(100)));
    let invalid = RationalTime {
        value: i64::MAX,
        timescale: 1,
    };
    assert_eq!(
        range_end(invalid, invalid).unwrap_err().kind,
        WorkflowErrorKind::ResourceLimit
    );
    let bad_origin = SourceClockSpec::Identity {
        duration: RationalTime {
            value: 1,
            timescale: 0,
        },
    };
    assert_eq!(
        clock_range(bad_origin).unwrap_err().kind,
        WorkflowErrorKind::ResourceLimit
    );

    let empty = veac_ir::MediaProbeSnapshot {
        schema_version: 3,
        engine: "test".into(),
        selection_policy: "test".into(),
        container_format: "test".into(),
        container_brand: None,
        observed_identity: veac_ir::MediaIdentity {
            algorithm: veac_ir::HashAlgorithm::Sha256,
            digest: "ab".repeat(32),
        },
        container_duration: None,
        streams: vec![],
        selected_video_stream: None,
        selected_audio_stream: None,
    };
    let selection = veac_ir::StreamSelection {
        global_index: 0,
        type_index: 0,
    };
    assert!(selected(&empty, selection, ProbedStreamType::Data).is_err());
    assert_eq!(
        probe_error(ProbeError::IdentityChanged {
            path: "source".into()
        })
        .kind,
        WorkflowErrorKind::SourceIdentityMismatch
    );
    assert_eq!(
        probe_error(ProbeError::InvalidField {
            field: "x",
            value: "y".into()
        })
        .kind,
        WorkflowErrorKind::InvalidContract
    );
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}
