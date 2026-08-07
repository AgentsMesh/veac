use std::{fs, path::PathBuf};

use veac_ir::{decode_edit_batch_json, EditOperation, ItemId, MaterialKind};
use veac_lang::program::{build_path_with_inputs, build_source, parse_build_input_manifest};
use veac_lang::source_edit::{
    decode_source_edit_batch_json, SourceEditOperation, SourceNodeKind, SourceNodePath,
};
use veac_lang::vocabulary::{
    language_spec, CanonicalRole, GrammarPosition, IdentifierPolicy, LanguageLayer,
    VocabularyCategory, LANGUAGE_SPEC_SCHEMA_VERSION,
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
fn documented_complete_project_builds_and_validates() {
    let relative = "docs/language-design/agent-authoring.md";
    let source = read(relative);
    let projects = fenced_blocks(&source, "veac");
    assert_eq!(
        projects.len(),
        1,
        "{relative} must contain one executable source"
    );
    let built = build_source(projects[0])
        .unwrap_or_else(|error| panic!("{relative} does not build: {error:?}"));
    veac_ir::validate(built.envelope())
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
    for typed_body_token in ["`BodySite`", "`body_equals`", "`set_body`"] {
        assert!(source.contains(typed_body_token));
    }
}

#[test]
fn programming_reference_points_to_an_executable_source_graph() {
    let relative = "examples/programming-language/main.veac";
    assert!(read("docs/language-reference/programming.md").contains(relative));
    let inputs =
        parse_build_input_manifest(&read("examples/programming-language/build-inputs.json"))
            .unwrap();
    let built = build_path_with_inputs(&workspace_root().join(relative), &inputs)
        .unwrap_or_else(|errors| panic!("{relative} does not build: {errors}"));
    assert_eq!(
        built
            .sources()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["brand.veac", "main.veac", "showcase.veac"]
    );
    let envelope = built.envelope();
    assert_eq!(envelope.project.sequences.len(), 1);
    assert_eq!(envelope.project.sequences[0].tracks.len(), 2);
    assert!(envelope.project.sequences[0]
        .tracks
        .iter()
        .all(|track| track.clips.len() == 2));
    assert_eq!(envelope.project.materials.len(), 1);
    assert_eq!(envelope.project.materials[0].kind, MaterialKind::Font);
    veac_ir::validate(envelope)
        .unwrap_or_else(|errors| panic!("{relative} is invalid: {errors:?}"));
}

#[test]
fn vocabulary_reference_tracks_the_public_contract() {
    let relative = "docs/language-reference/vocabulary.md";
    let source = read(relative);
    let index = read("docs/language-reference/README.md");
    assert!(index.contains("[Versioned language vocabulary](vocabulary.md)"));

    let spec = language_spec();
    spec.validate().unwrap();
    assert!(source.contains(&format!(
        "`schema_version` 为 `{LANGUAGE_SPEC_SCHEMA_VERSION}`"
    )));
    assert!(!source.contains("AST payload union"));
    let use_count = spec
        .vocabulary
        .entries
        .iter()
        .map(|entry| entry.uses.len())
        .sum::<usize>();
    let totals = format!(
        "`{}` 个不同 spelling 和 `{use_count}` 个精确 syntax use",
        spec.vocabulary.entries.len()
    );
    assert!(
        source.contains(&totals),
        "missing vocabulary totals: {totals}"
    );
    for field in [
        "`lexer_keywords`",
        "`identifier_policy`",
        "`uses`",
        "`category`",
        "`layer`",
        "`position`",
        "`canonical_role`",
    ] {
        assert!(source.contains(field), "missing v2 field {field}");
    }
    for category in VocabularyCategory::ALL {
        assert!(source.contains(&format!("`{}`", json_name(category))));
    }
    for layer in LanguageLayer::ALL {
        assert!(source.contains(&format!("`{}`", json_name(layer))));
    }
    for policy in IdentifierPolicy::ALL {
        assert!(source.contains(&format!("`{}`", json_name(policy))));
    }
    for role in CanonicalRole::ALL {
        assert!(source.contains(&format!("`{}`", json_name(role))));
    }
    for position in GrammarPosition::ALL {
        let name = json_name(position);
        assert!(
            source.contains(&format!("`{name}`")),
            "missing position {name}"
        );
    }
    for count in spec.vocabulary.counts() {
        let row = format!("| `{}` | {} |", json_name(count.category), count.count);
        assert!(source.contains(&row), "missing vocabulary count row: {row}");
    }
}

fn json_name(value: impl serde::Serialize) -> String {
    serde_json::to_value(value)
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned()
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
