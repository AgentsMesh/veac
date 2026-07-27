use serde_json::json;
use veac_artifact::{validate_artifact_json, MAX_ARTIFACT_JSON_STRING_BYTES};
use veac_codegen::emitter::{MAX_ASS_PAYLOAD_BYTES, MAX_FILTER_GRAPH_BYTES};

#[test]
fn graph_budget_matches_the_canonical_artifact_string_budget() {
    let envelope = json!({
        "kind": "ffmpeg",
        "filter_graph": "x".repeat(MAX_FILTER_GRAPH_BYTES),
        "maps": ["[outv]", "[outa]"],
        "resources": ["font", "media"]
    });
    validate_artifact_json(&envelope).unwrap();
    assert_eq!(
        MAX_ARTIFACT_JSON_STRING_BYTES - MAX_FILTER_GRAPH_BYTES,
        32 * 1024
    );
    let error = validate_artifact_json(&json!({
        "filter_graph": "x".repeat(MAX_FILTER_GRAPH_BYTES + 32 * 1024)
    }))
    .unwrap_err();
    assert_eq!(error.kind, veac_artifact::ArtifactErrorKind::ResourceLimit);
}

#[test]
fn ass_budget_accounts_for_base64_expansion_and_filter_overhead() {
    let payload = std::hint::black_box(MAX_ASS_PAYLOAD_BYTES);
    let encoded = payload.div_ceil(3) * 4;
    assert!(encoded < MAX_FILTER_GRAPH_BYTES);
    assert!(MAX_FILTER_GRAPH_BYTES - encoded >= 16 * 1024);
    assert!(payload > 246_599);
}
