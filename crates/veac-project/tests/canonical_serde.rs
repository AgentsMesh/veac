mod support;

use schemars::schema_for;
use serde_json::json;
use support::manifest;
use veac_project::*;

#[test]
fn canonical_json_and_digest_are_order_independent_for_maps() {
    let mut first = manifest();
    let exact = InstanceSelector::Exact {
        locale: Some(LocaleId::from("zh-cn")),
        axes: [(AxisId::from("theme"), AxisValue::from("dark"))]
            .into_iter()
            .collect(),
    };
    first.targets[1].needs.push(TargetRef {
        target: TargetId::from("plate"),
        profile: Some(ProfileId::from("preview")),
        selector: exact,
    });
    let bytes = canonical_manifest_bytes(&first).unwrap();
    let json = canonical_manifest_json(&first).unwrap();
    assert_eq!(bytes, json.as_bytes());
    assert_eq!(manifest_digest(&first).unwrap().len(), 71);
    assert_eq!(
        manifest_digest(&first).unwrap(),
        manifest_digest(&first.clone()).unwrap()
    );
    assert!(!json.contains("\n"));
}

#[test]
fn serde_rejects_unknown_fields_at_every_contract_layer() {
    let value = serde_json::to_value(manifest()).unwrap();
    let mut top = value.clone();
    top.as_object_mut()
        .unwrap()
        .insert("extra".into(), json!(true));
    assert!(serde_json::from_value::<ProjectManifestV1>(top).is_err());

    let mut profile = value.clone();
    profile["profiles"][0]["execution"]["extra"] = json!(true);
    assert!(serde_json::from_value::<ProjectManifestV1>(profile).is_err());

    let mut input = value.clone();
    input["targets"][0]["inputs"][0]["source"]["extra"] = json!(true);
    assert!(serde_json::from_value::<ProjectManifestV1>(input).is_err());

    let mut selector = value;
    selector["targets"][1]["inputs"][0]["source"]["target"]["selector"]["extra"] = json!(true);
    assert!(serde_json::from_value::<ProjectManifestV1>(selector).is_err());
}

#[test]
fn schemas_cover_manifest_and_resolved_graph() {
    let manifest_schema = serde_json::to_value(schema_for!(ProjectManifestV1)).unwrap();
    let graph_schema = serde_json::to_value(schema_for!(ResolvedTargetGraph)).unwrap();
    assert_eq!(manifest_schema["title"], "ProjectManifestV1");
    assert_eq!(graph_schema["title"], "ResolvedTargetGraph");
    assert!(manifest_schema.to_string().contains("all_matching"));
    assert!(graph_schema.to_string().contains("manifest_digest"));
}

#[test]
fn typed_identifiers_and_paths_are_displayable() {
    let id = TargetId::new("render");
    let path = ProjectPath::new("veac/main.veac");
    let template = DeliveryPathTemplate::new("dist/{target}.mp4");
    assert_eq!(id.to_string(), "render");
    assert_eq!(id.as_str(), "render");
    assert_eq!(path.as_str(), "veac/main.veac");
    assert_eq!(template.as_str(), "dist/{target}.mp4");
}

#[test]
fn defaults_are_bounded_and_version_constants_are_stable() {
    let defaults = ProjectDefaults::default();
    assert_eq!(defaults.max_instances_per_target, 256);
    assert_eq!(defaults.max_total_instances, 4_096);
    assert_eq!(PROJECT_SCHEMA, "veac.project");
    assert_eq!(PROJECT_MANIFEST_VERSION, 1);
    assert_eq!(RESOLVED_GRAPH_VERSION, 1);
}
