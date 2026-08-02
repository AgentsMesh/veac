use std::{fs, path::PathBuf};

use veac_ir::{decode_edit_batch_json, EditOperation, ItemId};
use veac_lang::authoring::{lower_document, parse};
use veac_lang::program::compile_path;
use veac_lang::source_edit::{
    decode_source_edit_batch_json, SourceEditOperation, SourceNodeKind, SourceNodePath,
};

const EDIT_BATCH_FENCE: &str = "json,canonical-edit-batch";
const EDIT_BATCH_DOCS: &[&str] = &[
    "docs/language-design/agent-authoring.md",
    "docs/language-design/mapping.md",
];
const SOURCE_EDIT_FENCE: &str = "json,source-edit-batch";

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

#[test]
fn documented_source_edit_batch_uses_the_production_contract() {
    let relative = "docs/language-reference/source-editing.md";
    let source = read(relative);
    let blocks = fenced_blocks(&source, SOURCE_EDIT_FENCE);
    assert_eq!(
        blocks.len(),
        1,
        "{relative} must contain one SourceEditBatch"
    );
    let batch = decode_source_edit_batch_json(blocks[0])
        .unwrap_or_else(|error| panic!("{relative} has an invalid SourceEditBatch: {error}"));
    let [SourceEditOperation::SetExpression { target, .. }] = batch.operations.as_slice() else {
        panic!("{relative} must demonstrate one set_expression operation");
    };
    assert_eq!(target.module, "main.veac");
    assert_eq!(target.kind(), SourceNodeKind::Constant);
    assert_eq!(
        target.path,
        SourceNodePath::Constant {
            constant: "section_duration".to_owned()
        }
    );
}

#[test]
fn programming_reference_points_to_an_executable_source_graph() {
    let relative = "examples/programming-language/main.veac";
    assert!(read("docs/language-reference/programming.md").contains(relative));
    let compiled = compile_path(&workspace_root().join(relative))
        .unwrap_or_else(|errors| panic!("{relative} does not compile: {errors}"));
    assert_eq!(
        compiled
            .sources()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["brand.veac", "main.veac"]
    );
    assert!(compiled.expanded_source().contains("sequence first-card"));
    assert!(compiled
        .expanded_source()
        .contains("item veac-h-10-first-card-5-title"));
    assert!(compiled
        .expanded_source()
        .contains("item veac-h-11-second-card-5-title"));
    let envelope = lower_document(compiled.document())
        .unwrap_or_else(|errors| panic!("{relative} does not lower: {errors:?}"));
    veac_ir::validate(&envelope)
        .unwrap_or_else(|errors| panic!("{relative} is invalid: {errors:?}"));
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
