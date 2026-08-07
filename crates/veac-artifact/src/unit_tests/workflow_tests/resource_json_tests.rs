use serde_json::json;

use super::request;
use crate::*;

#[test]
fn typed_analysis_envelope_is_strict_canonical_and_identity_bound() {
    let source = ContentDigest::sha256(b"source");
    let result = envelope(vec![boundary(4, 900_000)]);
    let request = ingestion(source.clone(), result.clone());
    let first = request.descriptor().unwrap();
    assert_eq!(first.kind(), ArtifactKind::Analysis);
    assert_eq!(
        result.canonical_bytes(4_096).unwrap(),
        result.canonical_bytes(4_096).unwrap()
    );
    let mut changed = request.clone();
    changed.result = envelope(vec![boundary(5, 900_000)]);
    assert_ne!(
        artifact_key(&first).unwrap(),
        artifact_key(&changed.descriptor().unwrap()).unwrap()
    );
    changed = request;
    changed.source_identity = ContentDigest::sha256(b"other");
    assert_ne!(
        artifact_key(&first).unwrap(),
        artifact_key(&changed.descriptor().unwrap()).unwrap()
    );

    let mut unknown = serde_json::to_value(result).unwrap();
    unknown["unexpected"] = json!(true);
    assert!(serde_json::from_value::<AnalysisResultEnvelope>(unknown).is_err());
}

#[test]
fn typed_analysis_rejects_mismatch_order_ranges_and_budgets() {
    let mut mismatch = envelope(vec![]);
    mismatch.result = AnalysisResult::BeatMarkers(BeatMarkerAnalysisResult { markers: vec![] });
    assert_eq!(
        mismatch.validate().unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
    let mut result = envelope(vec![boundary(4, 900_000), boundary(3, 800_000)]);
    assert_eq!(
        result.validate().unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
    result = envelope(vec![boundary(4, 1_000_001)]);
    assert_eq!(
        result.validate().unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
    let result = envelope(vec![boundary(4, 900_000)]);
    assert_eq!(
        result.canonical_bytes(1).unwrap_err().kind,
        ArtifactErrorKind::ResourceLimit
    );
    let mut request = ingestion(ContentDigest::sha256(b"source"), result);
    request.producer.name = "x".repeat(MAX_ARTIFACT_JSON_STRING_BYTES + 1);
    assert_eq!(
        request.descriptor().unwrap_err().kind,
        ArtifactErrorKind::ResourceLimit
    );
}

#[test]
fn public_json_helpers_enforce_shape_and_caller_limit_caps() {
    let value = json!({"value": [1, 2, 3]});
    validate_artifact_json(&value).unwrap();
    assert_eq!(
        canonical_artifact_json_bounded(&value, 17).unwrap(),
        br#"{"value":[1,2,3]}"#
    );
    for limit in [0, MAX_ANALYSIS_PAYLOAD_BYTES + 1, u64::MAX] {
        assert_eq!(
            canonical_artifact_json_bounded(&value, limit)
                .unwrap_err()
                .kind,
            ArtifactErrorKind::ResourceLimit
        );
    }
}

#[test]
fn invalid_backend_time_precision_is_rejected_instead_of_rounded() {
    let spec = MediaArtifactSpec::Thumbnail(ThumbnailSpec {
        source_stream: veac_ir::StreamSelection {
            global_index: 0,
            type_index: 0,
        },
        at: veac_ir::RationalTime::new(1, 3).unwrap(),
        width: 1,
        height: 1,
    });
    assert_eq!(
        request(spec).descriptor().unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
}

fn ingestion(
    source_identity: ContentDigest,
    result: AnalysisResultEnvelope,
) -> AnalysisIngestionRequest {
    AnalysisIngestionRequest {
        source_identity,
        producer: crate::test_support::producer(),
        result,
    }
}

fn envelope(boundaries: Vec<SceneBoundary>) -> AnalysisResultEnvelope {
    AnalysisResultEnvelope {
        schema: ANALYSIS_RESULT_SCHEMA_ID.into(),
        schema_version: ANALYSIS_RESULT_CONTRACT_VERSION,
        descriptor: AnalysisDescriptor::SceneBoundaries(SceneBoundaryAnalysisDescriptor {
            sensitivity_millionths: 500_000,
        }),
        result: AnalysisResult::SceneBoundaries(SceneBoundaryAnalysisResult { boundaries }),
    }
}

fn boundary(value: i64, confidence_millionths: u32) -> SceneBoundary {
    SceneBoundary {
        at: veac_ir::RationalTime::new(value, 10).unwrap(),
        confidence_millionths,
    }
}
