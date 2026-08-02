use crate::{test_support::*, *};

#[test]
fn manifest_captures_plan_tools_inputs_and_artifacts() {
    let plan = plan(b"media");
    let descriptor = descriptor();
    let artifact = ArtifactDeclaration {
        key: artifact_key(&descriptor).unwrap(),
        descriptor,
    };
    let tool = ToolFingerprint {
        name: "ffmpeg".to_owned(),
        version: "8.0".to_owned(),
        configuration: ContentDigest::sha256(b"buildconf"),
    };
    let manifest = BuildManifest::from_plan(&plan, vec![tool], vec![artifact]).unwrap();
    assert_eq!(manifest.inputs.len(), 1);
    assert_eq!(manifest.source, plan.header.source);
    assert_eq!(manifest.output, plan.output);
    assert_eq!(build_manifest_hash(&manifest).unwrap().value.len(), 64);
    assert_eq!(
        canonical_manifest_bytes(&manifest).unwrap(),
        canonical_manifest_bytes(&manifest).unwrap()
    );
}

#[test]
fn manifest_rejects_bad_header_and_unsorted_collections() {
    let mut value = BuildManifest::from_plan(&plan(b"media"), vec![], vec![]).unwrap();
    value.schema = "other".to_owned();
    assert_invalid(&value);
    value = BuildManifest::from_plan(&plan(b"media"), vec![], vec![]).unwrap();
    value.schema_version = 2;
    assert_invalid(&value);
    value = BuildManifest::from_plan(&plan(b"media"), vec![], vec![]).unwrap();
    value.inputs.push(value.inputs[0].clone());
    assert_invalid(&value);

    let mut invalid_plan = plan(b"media");
    invalid_plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .opacity = veac_ir::Animatable::constant(f64::NAN);
    let error = BuildManifest::from_plan(&invalid_plan, vec![], vec![]).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::Serialization);
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn manifest_requires_sorted_valid_tool_fingerprints() {
    let base = ToolFingerprint {
        name: "ffmpeg".to_owned(),
        version: "8".to_owned(),
        configuration: ContentDigest::sha256(b"config"),
    };
    let mut value = BuildManifest::from_plan(&plan(b"media"), vec![base.clone()], vec![]).unwrap();
    value.tools.push(base.clone());
    assert_invalid(&value);
    for mutate in [0, 1, 2] {
        let mut tool = base.clone();
        match mutate {
            0 => tool.name.clear(),
            1 => tool.version.clear(),
            _ => tool.configuration.value = "bad".to_owned(),
        }
        let value = BuildManifest {
            tools: vec![tool],
            ..BuildManifest::from_plan(&plan(b"media"), vec![], vec![]).unwrap()
        };
        assert_invalid(&value);
    }
}

#[test]
fn manifest_rejects_unsorted_or_mismatched_artifacts() {
    let first_descriptor = descriptor();
    let first = ArtifactDeclaration {
        key: artifact_key(&first_descriptor).unwrap(),
        descriptor: first_descriptor,
    };
    let mut second_descriptor = descriptor();
    second_descriptor.parameters = serde_json::json!({"height": 360});
    let second = ArtifactDeclaration {
        key: artifact_key(&second_descriptor).unwrap(),
        descriptor: second_descriptor,
    };
    let mut artifacts = vec![first, second];
    artifacts.sort_by(|left, right| right.key.value.cmp(&left.key.value));
    let value = BuildManifest {
        artifacts,
        ..BuildManifest::from_plan(&plan(b"media"), vec![], vec![]).unwrap()
    };
    assert_invalid(&value);
    let mut value = BuildManifest::from_plan(&plan(b"media"), vec![], vec![]).unwrap();
    value.artifacts.push(ArtifactDeclaration {
        key: ContentDigest::sha256(b"wrong"),
        descriptor: descriptor(),
    });
    assert_invalid(&value);
}

fn assert_invalid(value: &BuildManifest) {
    assert_eq!(
        value.validate().unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
}
