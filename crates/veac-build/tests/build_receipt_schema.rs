use serde_json::json;
use veac_artifact::ContentDigest;
use veac_build::{
    build_receipt_json_schema, ArtifactOutputs, BuildOutcome, BuildReceipt, NodeCacheKey, NodeId,
    NodeReceipt, NodeStatus, PortName,
};

fn sample_receipt() -> BuildReceipt {
    let output = PortName::new("video").unwrap();
    let outputs =
        ArtifactOutputs::try_from_iter([(output, ContentDigest::sha256(b"rendered video"))])
            .unwrap();
    BuildReceipt {
        graph_digest: ContentDigest::sha256(b"graph"),
        outcome: BuildOutcome::Succeeded,
        nodes: vec![NodeReceipt {
            node_id: NodeId::new("render").unwrap(),
            status: NodeStatus::Executed,
            cache_key: Some(
                NodeCacheKey::computation("fixture", 1, ContentDigest::sha256(b"action")).unwrap(),
            ),
            outputs: Some(outputs),
            message: None,
        }],
    }
}

#[test]
fn receipt_json_round_trips_with_descriptor() {
    let receipt = sample_receipt();
    let value = serde_json::to_value(&receipt).unwrap();
    assert!(value["nodes"][0]["cache_key"]["descriptor"].is_object());
    assert_eq!(
        serde_json::from_value::<BuildReceipt>(value).unwrap(),
        receipt
    );
}

#[test]
fn receipt_rejects_unknown_fields_and_invalid_ids() {
    let mut value = serde_json::to_value(sample_receipt()).unwrap();
    value["unexpected"] = json!(true);
    assert!(serde_json::from_value::<BuildReceipt>(value).is_err());

    let mut value = serde_json::to_value(sample_receipt()).unwrap();
    value["nodes"][0]["node_id"] = json!("invalid node");
    assert!(serde_json::from_value::<BuildReceipt>(value).is_err());
}

#[test]
fn schema_exposes_receipt_contract() {
    let schema = build_receipt_json_schema().unwrap();
    assert_eq!(schema["title"], "BuildReceipt");
    let text = serde_json::to_string(&schema).unwrap();
    for expected in [
        "NodeReceipt",
        "NodeStatus",
        "BuildOutcome",
        "NodeCacheKey",
        "descriptor",
        "cache_hit",
    ] {
        assert!(text.contains(expected), "schema omitted {expected}");
    }
    assert_eq!(
        schema["$defs"]["NodeReceipt"]["properties"]["outputs"]["additionalProperties"]["$ref"],
        "#/$defs/ContentDigest"
    );
}
