use crate::{test_support::sample_project, *};

use super::support::{binding, codes};

#[test]
fn manifest_constructor_and_binding_shape_are_explicit() {
    let project = sample_project();
    let manifest = &project.executable;
    assert_eq!(manifest.language, ExecutableLanguage::Veac);
    assert_eq!(manifest.core_version, CURRENT_CORE_VERSION);
    assert_eq!(manifest.domain_opset_version, CURRENT_DOMAIN_OPSET_VERSION);
    assert_eq!(manifest.temporal_opset_version, TEMPORAL_OPSET_VERSION);

    let id = TemporalBindingId::new("tbd_shape").unwrap();
    let value = binding::<f64>(&id);
    assert_eq!(value.binding_id(), Some(&id));
    assert_eq!(value.keyframes(), None);
    assert_eq!(Animatable::constant(1.0).binding_id(), None);
    assert_eq!(
        serde_json::to_value(&value).unwrap(),
        serde_json::json!({"type": "binding", "binding_id": "tbd_shape"})
    );
}

#[test]
fn schema_v7_requires_manifest_temporal_library_and_binding_ids() {
    let schema = project_json_schema().unwrap();
    let required = schema["required"].as_array().unwrap();
    assert!(required.iter().any(|value| value == "executable"));
    assert!(required.iter().any(|value| value == "temporal"));
    let text = serde_json::to_string(&schema).unwrap();
    assert!(text.contains("^[0-9a-f]{64}$"));
    assert!(text.contains("TemporalBindingId"));

    for field in ["executable", "temporal"] {
        let mut value = serde_json::to_value(sample_project()).unwrap();
        value.as_object_mut().unwrap().remove(field);
        assert!(decode_canonical_json(&value.to_string()).is_err());
    }
}

#[test]
fn manifest_versions_language_and_library_identity_are_strict() {
    for invalid in ["", "1.2", "01.2.3", "1.a.3", "1.2.3.4"] {
        let mut project = sample_project();
        project.executable.language_version = invalid.to_owned();
        assert!(codes(&project).contains(&"EXECUTABLE_LANGUAGE_VERSION".to_owned()));
    }

    let mut project = sample_project();
    project.executable.core_version += 1;
    project.executable.domain_opset_version += 1;
    project.executable.temporal_opset_version += 1;
    let actual = codes(&project);
    for expected in [
        "EXECUTABLE_CORE_VERSION",
        "EXECUTABLE_DOMAIN_OPSET",
        "EXECUTABLE_TEMPORAL_OPSET",
        "EXECUTABLE_TEMPORAL_LIBRARY_OPSET",
    ] {
        assert!(actual.contains(&expected.to_owned()));
    }
}

#[test]
fn every_manifest_digest_is_lowercase_sha256() {
    let mut project = sample_project();
    project.executable.digests.domain_registry_sha256 = "A".repeat(64);
    project.executable.digests.main_core_sha256.clear();
    project.executable.digests.source_graph_sha256 = "0".repeat(63);
    project.executable.digests.declared_inputs_sha256 = "g".repeat(64);
    project.executable.digests.compiler_sha256.push('0');
    let actual = codes(&project);
    assert_eq!(
        actual
            .iter()
            .filter(|code| code.as_str() == "EXECUTABLE_DIGEST")
            .count(),
        5
    );
}

#[test]
fn schema_v7_has_no_implicit_or_alias_highlight_contract() {
    let animation = TextAnimation {
        granularity: TextGranularity::Whole,
        transform: TextUnitTransform::default(),
        reveal: Animatable::constant(1.0),
        highlight: None,
        opacity: Animatable::constant(1.0),
        stagger: RationalTime::zero(600).unwrap(),
    };
    let mut value = serde_json::to_value(animation).unwrap();
    value.as_object_mut().unwrap().remove("highlight");
    assert!(serde_json::from_value::<TextAnimation>(value).is_err());
}
