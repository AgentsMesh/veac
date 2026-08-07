use std::error::Error;

use serde_json::json;

use crate::{test_support::*, *};

#[test]
fn digest_and_descriptor_are_deterministic() {
    let digest = ContentDigest::sha256(b"abc");
    assert_eq!(
        digest.value,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    digest.validate().unwrap();
    let value = descriptor();
    let first = canonical_descriptor_bytes(&value).unwrap();
    let second = canonical_descriptor_bytes(&value).unwrap();
    assert_eq!(first, second);
    assert_eq!(artifact_key(&value).unwrap(), ContentDigest::sha256(first));

    let decoded: ArtifactDescriptor = serde_json::from_slice(&second).unwrap();
    assert_eq!(decoded, value);
    let mut unknown: serde_json::Value = serde_json::from_slice(&second).unwrap();
    unknown["unknown"] = json!(true);
    assert!(serde_json::from_value::<ArtifactDescriptor>(unknown).is_err());
    let mut unknown_role: serde_json::Value = serde_json::from_slice(&second).unwrap();
    unknown_role["dependencies"] = json!([{
        "role": "provider_magic",
        "identity": ContentDigest::sha256(b"dependency")
    }]);
    assert!(serde_json::from_value::<ArtifactDescriptor>(unknown_role).is_err());
    let mut unknown_type: serde_json::Value = serde_json::from_slice(&second).unwrap();
    unknown_type["parameters"]["type"] = json!("extension_property_bag");
    assert!(serde_json::from_value::<ArtifactDescriptor>(unknown_type).is_err());
}

#[test]
fn malformed_digests_and_descriptor_headers_are_rejected() {
    for value in ["abc", &"A".repeat(64), &"g".repeat(64)] {
        let digest = ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: value.to_owned(),
        };
        assert_eq!(
            digest.validate().unwrap_err().kind,
            ArtifactErrorKind::InvalidContract
        );
    }
    let mut value = descriptor();
    value.schema = "other".to_owned();
    assert_invalid(&value);
    value = descriptor();
    value.schema_version = 1;
    assert_invalid(&value);
}

#[test]
fn descriptor_requires_valid_producer_parameters_and_dependencies() {
    let mut cases = Vec::new();
    let mut value = descriptor();
    value.producer.name.clear();
    cases.push(value);
    let mut value = descriptor();
    value.producer.version.clear();
    cases.push(value);
    let mut value = descriptor();
    value.producer.configuration.value = "bad".to_owned();
    cases.push(value);
    let mut value = descriptor();
    let ArtifactParameters::ProxyVideo(parameters) = &mut value.parameters else {
        unreachable!()
    };
    parameters.width = 0;
    cases.push(value);
    let mut value = descriptor();
    value.dependencies[0].identity.value = "bad".to_owned();
    cases.push(value);
    for value in cases {
        assert_invalid(&value);
    }
}

#[test]
fn dependencies_must_be_strictly_sorted() {
    let mut duplicate = descriptor();
    duplicate
        .dependencies
        .push(duplicate.dependencies[0].clone());
    assert_invalid(&duplicate);
    let mut descending = descriptor();
    descending.dependencies.insert(
        0,
        ArtifactDependency::new(
            ArtifactDependencyRole::Resources,
            ContentDigest::sha256(b"z"),
        ),
    );
    assert_invalid(&descending);
}

#[test]
fn descriptor_and_json_shape_budgets_fail_with_resource_limits() {
    let mut producer = descriptor();
    producer.producer.name = "x".repeat(MAX_ARTIFACT_JSON_STRING_BYTES + 1);
    assert_eq!(
        producer.validate().unwrap_err().kind,
        ArtifactErrorKind::ResourceLimit
    );

    let mut dependency = descriptor();
    dependency.dependencies =
        vec![dependency.dependencies[0].clone(); MAX_ARTIFACT_DEPENDENCIES + 1];
    assert_eq!(
        dependency.validate().unwrap_err().kind,
        ArtifactErrorKind::ResourceLimit
    );

    let nodes = serde_json::Value::Array(vec![serde_json::Value::Null; MAX_ARTIFACT_JSON_NODES]);
    assert_eq!(
        validate_artifact_json(&nodes).unwrap_err().kind,
        ArtifactErrorKind::ResourceLimit
    );
}

#[test]
fn artifact_error_exposes_kind_message_and_source() {
    let io = std::fs::read("/path/that/does/not/exist").unwrap_err();
    let error: ArtifactError = io.into();
    assert_eq!(error.kind, ArtifactErrorKind::Io);
    assert_eq!(error.to_string(), "artifact filesystem operation failed");
    assert!(error.source().is_some());
}

fn assert_invalid(value: &ArtifactDescriptor) {
    assert_eq!(
        value.validate().unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
}
