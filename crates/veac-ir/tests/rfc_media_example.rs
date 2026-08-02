use std::path::{Path, PathBuf};

use veac_ir::{Material, MaterialKind, StreamChoice};

#[test]
fn accepted_rfc_media_json_deserializes_as_the_current_model() {
    let path = repository_root().join("docs/rfcs/agent-authoring-and-canonical-ir.md");
    let markdown = std::fs::read_to_string(&path).unwrap();
    let source = marked_fence(&markdown, "```json,veac-media");
    let material: Material = serde_json::from_str(source).unwrap();
    assert_eq!(material.kind, MaterialKind::Image);
    assert!(material.probe.is_none());
    assert_eq!(material.stream_intent.video, StreamChoice::Auto);
    assert_eq!(material.stream_intent.audio, StreamChoice::Disabled);
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn marked_fence<'a>(markdown: &'a str, marker: &str) -> &'a str {
    let start = markdown
        .find(marker)
        .unwrap_or_else(|| panic!("missing {marker} fence"));
    let source_start = start + marker.len() + 1;
    let remainder = &markdown[source_start..];
    let end = remainder.find("\n```").expect("unterminated RFC fence");
    &remainder[..end]
}
