use serde_json::json;

use super::request;
use crate::*;

#[test]
fn analysis_metadata_rejects_excessive_strings_depth_and_nodes() {
    assert_limit(json!({"value": "x".repeat(MAX_ARTIFACT_JSON_STRING_BYTES + 1)}));

    let mut deep = json!(null);
    for _ in 0..MAX_ARTIFACT_JSON_DEPTH {
        deep = json!([deep]);
    }
    assert_limit(json!({"value": deep}));

    let nodes = vec![serde_json::Value::Null; MAX_ARTIFACT_JSON_NODES];
    assert_limit(json!({"value": nodes}));
}

#[test]
fn canonical_descriptor_bytes_are_bounded_before_key_generation() {
    let mut descriptor = request(MediaArtifactSpec::Analysis(AnalysisSpec {
        analysis_type: "scene".into(),
        configuration: json!({}),
    }))
    .descriptor()
    .unwrap();
    descriptor.dependencies = (0..MAX_ARTIFACT_DEPENDENCIES)
        .map(|index| ArtifactDependency {
            role: format!("{index:04}-{}", "x".repeat(300)),
            identity: ContentDigest::sha256(b"dependency"),
        })
        .collect();
    assert_eq!(
        canonical_descriptor_bytes(&descriptor).unwrap_err().kind,
        ArtifactErrorKind::ResourceLimit
    );
    assert_eq!(
        artifact_key(&descriptor).unwrap_err().kind,
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

fn assert_limit(configuration: serde_json::Value) {
    let spec = MediaArtifactSpec::Analysis(AnalysisSpec {
        analysis_type: "scene".into(),
        configuration,
    });
    assert_eq!(
        request(spec).descriptor().unwrap_err().kind,
        ArtifactErrorKind::ResourceLimit
    );
}
