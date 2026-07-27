use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use veac_lang::authoring::{
    parse, AnnotationPayloadDecl, ApplyScope, Document, RelationDecl, RelationKind, SourceDecl,
    StructureDecl, TemplateSlotDecl,
};

const EXAMPLES: &[&str] = &[
    "examples/executable-mechanisms/main.veac",
    "examples/apply-scopes/main.veac",
    "examples/all-features/main.veac",
    "examples/nested-and-multicam/main.veac",
    "examples/template-fill/main.veac",
];

#[test]
fn project_structure_catalog_matches_typed_examples() {
    let root = workspace_root();
    let catalog = root.join("examples/catalog/mechanisms/project-structure.json");
    let expected: Value =
        serde_json::from_str(&fs::read_to_string(catalog).expect("read project structure catalog"))
            .expect("parse project structure catalog");
    let expected = expected
        .get("mechanisms")
        .and_then(Value::as_array)
        .expect("mechanisms must be an array")
        .iter()
        .map(|mechanism| {
            mechanism
                .get("id")
                .and_then(Value::as_str)
                .expect("mechanism id must be a string")
                .to_owned()
        })
        .collect::<BTreeSet<_>>();

    let mut actual = BTreeSet::new();
    for path in EXAMPLES {
        let source = fs::read_to_string(root.join(path)).expect("read example");
        collect_document(&parse(&source).expect("parse example"), &mut actual);
    }
    assert_eq!(actual, expected);
}

fn collect_document(document: &Document, found: &mut BTreeSet<String>) {
    let project = &document.project;
    if !project.multicams.is_empty() {
        insert(found, "project.multicam");
    }
    for annotation in &project.annotations {
        collect_annotation(&annotation.payload, found);
    }
    for sequence in &project.sequences {
        for layer in &sequence.layers {
            for item in &layer.items {
                match &item.source {
                    SourceDecl::Sequence { .. } => insert(found, "source.sequence"),
                    SourceDecl::Multicam { .. } => insert(found, "source.multicam"),
                    _ => {}
                }
                if let Some(slot) = &item.template_slot {
                    insert(
                        found,
                        match slot {
                            TemplateSlotDecl::Media { .. } => "template.slot.media",
                            TemplateSlotDecl::Text { .. } => "template.slot.text",
                        },
                    );
                }
            }
        }
        for structure in &sequence.structures {
            match structure {
                StructureDecl::Apply(apply) => insert(
                    found,
                    match &apply.scope {
                        ApplyScope::CompositeBand { .. } => "apply.scope.composite-band",
                        ApplyScope::Layer { .. } => "apply.scope.layer",
                        ApplyScope::Items { .. } => "apply.scope.items",
                    },
                ),
                StructureDecl::Relation(relation) => collect_relation(relation, found),
            }
        }
    }
}

fn collect_relation(relation: &RelationDecl, found: &mut BTreeSet<String>) {
    match &relation.kind {
        RelationKind::Group(_) => insert(found, "relation.group"),
        RelationKind::AvLink(_) => insert(found, "relation.av-link"),
        RelationKind::Transition(_) | RelationKind::Matte(_) | RelationKind::Sidechain(_) => {}
    }
}

fn collect_annotation(payload: &AnnotationPayloadDecl, found: &mut BTreeSet<String>) {
    insert(
        found,
        match payload {
            AnnotationPayloadDecl::Marker(_) => "annotation.marker",
            AnnotationPayloadDecl::Language(_) => "annotation.language",
            AnnotationPayloadDecl::SceneBoundary(_) => "annotation.scene-boundary",
            AnnotationPayloadDecl::Scene => "annotation.scene",
            AnnotationPayloadDecl::Beat(_) => "annotation.beat",
            AnnotationPayloadDecl::Silence(_) => "annotation.silence",
            AnnotationPayloadDecl::Filler(_) => "annotation.filler",
            AnnotationPayloadDecl::Highlight(_) => "annotation.highlight",
            AnnotationPayloadDecl::Review(_) => "annotation.review",
        },
    );
}

fn insert(found: &mut BTreeSet<String>, id: &str) {
    found.insert(id.to_owned());
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
