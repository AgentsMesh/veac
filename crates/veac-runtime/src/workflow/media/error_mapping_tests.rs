use veac_artifact::{
    ContentDigest, DigestAlgorithm, MediaArtifactRequest, MediaArtifactSpec, ProducerFingerprint,
};

use super::*;

#[test]
fn runtime_launch_errors_keep_resource_limits_distinct_from_tool_failures() {
    let limited = launch_error(crate::RuntimeError::resource_limit("deadline"));
    assert_eq!(limited.kind, WorkflowErrorKind::ResourceLimit);
    let failed = launch_error(crate::RuntimeError::new("spawn"));
    assert_eq!(failed.kind, WorkflowErrorKind::ToolFailure);
}

#[test]
fn invalid_artifact_descriptors_map_to_contract_errors() {
    let request = MediaArtifactRequest {
        source_identity: ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: "bad".into(),
        },
        producer: ProducerFingerprint {
            name: "test".into(),
            version: "1".into(),
            configuration: ContentDigest::sha256(b"config"),
        },
        spec: MediaArtifactSpec::Analysis(veac_artifact::AnalysisSpec {
            analysis_type: "scene".into(),
            configuration: serde_json::json!({}),
        }),
    };
    assert_eq!(
        contract(request.descriptor()).unwrap_err().kind,
        WorkflowErrorKind::InvalidContract
    );
}
