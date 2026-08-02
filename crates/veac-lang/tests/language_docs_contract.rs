use std::{fs, path::PathBuf};

use veac_ir::{decode_edit_batch_json, EditOperation, ItemId};
use veac_lang::authoring::{lower_document, parse};

const EDIT_BATCH_FENCE: &str = "json,canonical-edit-batch";
const EDIT_BATCH_DOCS: &[&str] = &[
    "docs/language-design/agent-authoring.md",
    "docs/language-design/mapping.md",
];

#[test]
fn documented_edit_batches_decode_with_the_production_contract() {
    for relative in EDIT_BATCH_DOCS {
        let source = read(relative);
        let blocks = fenced_blocks(&source, EDIT_BATCH_FENCE);
        assert_eq!(blocks.len(), 1, "{relative} must contain one EditBatch");
        let batch = decode_edit_batch_json(blocks[0])
            .unwrap_or_else(|error| panic!("{relative} has an invalid EditBatch: {error}"));
        assert!(batch.operation_id.as_str().starts_with("op_"));
        let [EditOperation::SetClipEnabled { clip_id, .. }] = batch.operations.as_slice() else {
            panic!("{relative} must demonstrate one set_clip_enabled operation");
        };
        ItemId::new(clip_id.as_str())
            .unwrap_or_else(|error| panic!("{relative} has a non-canonical clip ID: {error}"));
    }
}

#[test]
fn documented_complete_project_parses_lowers_and_validates() {
    let relative = "docs/language-design/agent-authoring.md";
    let source = read(relative);
    let projects: Vec<_> = fenced_blocks(&source, "veac")
        .into_iter()
        .filter(|block| block.trim_start().starts_with("project "))
        .collect();
    assert_eq!(
        projects.len(),
        1,
        "{relative} must contain one full project"
    );
    let document =
        parse(projects[0]).unwrap_or_else(|error| panic!("{relative} does not parse: {error:?}"));
    let envelope = lower_document(&document)
        .unwrap_or_else(|error| panic!("{relative} does not lower: {error:?}"));
    veac_ir::validate(&envelope)
        .unwrap_or_else(|error| panic!("{relative} is invalid after lowering: {error:?}"));
}

fn fenced_blocks<'a>(source: &'a str, language: &str) -> Vec<&'a str> {
    let marker = format!("```{language}\n");
    source
        .split(&marker)
        .skip(1)
        .map(|tail| tail.split_once("\n```").expect("closed code fence").0)
        .collect()
}

fn read(relative: &str) -> String {
    let path = workspace_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
