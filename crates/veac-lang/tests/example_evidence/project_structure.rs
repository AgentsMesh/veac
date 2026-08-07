use std::{collections::BTreeSet, fs, path::Path};

use serde_json::Value;
use veac_ir::{
    AnnotationPayload, ApplyTarget, ClipSource, ProjectEnvelope, RelationKind, SlotKind,
};

use crate::support::{
    assert_preview_evidence, clips, entry_sequence, lower_example, sequence_by_key,
};

#[test]
fn project_structure_preview_rows_have_entry_sequence_evidence() {
    assert_preview_evidence("project-structure.json", preview_evidence);
}

#[test]
fn project_structure_workflow_rows_match_typed_examples() {
    let mut actual = BTreeSet::new();
    for relative in [
        "nested-and-multicam/main.veac",
        "executable-mechanisms/main.veac",
        "all-features/main.veac",
    ] {
        actual.extend(workflow_evidence(&lower_example(relative)));
    }
    assert_eq!(actual, catalog_ids("workflow_evidence"));
}

#[test]
fn non_entry_sequence_structure_is_not_preview_evidence() {
    let mut envelope = lower_example("nested-and-multicam/main.veac");
    envelope.project.entry_sequence_id = sequence_by_key(&envelope, "intro").id.clone();
    let actual = preview_evidence(&envelope);
    assert!(actual.is_empty(), "non-entry evidence leaked: {actual:?}");
}

fn preview_evidence(envelope: &ProjectEnvelope) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let entry = entry_sequence(envelope);
    for apply in &entry.applies {
        found.insert(
            match &apply.target {
                ApplyTarget::CompositeBand { .. } => "apply.scope.composite-band",
                ApplyTarget::Layer { .. } => "apply.scope.layer",
                ApplyTarget::ItemSet { .. } => "apply.scope.items",
            }
            .to_owned(),
        );
    }
    for clip in clips(envelope) {
        match &clip.source {
            ClipSource::Sequence { .. } => {
                found.insert("source.sequence".to_owned());
            }
            ClipSource::Multicam { .. } => {
                found.insert("source.multicam".to_owned());
            }
            _ => {}
        }
        if let Some(slot) = &clip.replaceable {
            if slot.kind != SlotKind::Text {
                found.insert("template.slot.media".to_owned());
            } else if clip.template_editable_text && matches!(clip.source, ClipSource::Text { .. })
            {
                found.insert("template.slot.text".to_owned());
            }
        }
    }
    for relation in envelope
        .project
        .relations
        .iter()
        .filter(|relation| relation.sequence_id == entry.id)
    {
        match &relation.kind {
            RelationKind::Group { .. } => {
                found.insert("relation.group".to_owned());
            }
            RelationKind::AvLink { .. } => {
                found.insert("relation.av-link".to_owned());
            }
            _ => {}
        }
    }
    found
}

fn workflow_evidence(envelope: &ProjectEnvelope) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    if !envelope.project.sequences.is_empty() {
        found.insert("project.sequence".to_owned());
    }
    if !envelope.project.multicam_groups.is_empty() {
        found.insert("project.multicam".to_owned());
    }
    if envelope
        .project
        .relations
        .iter()
        .any(|relation| matches!(relation.kind, RelationKind::AvLink { .. }))
    {
        found.insert("relation.av-link".to_owned());
    }
    for annotation in &envelope.project.annotations {
        let id = match &annotation.payload {
            AnnotationPayload::Marker { .. } => "annotation.marker",
            AnnotationPayload::Language { .. } => "annotation.language",
            AnnotationPayload::SceneBoundary { .. } => "annotation.scene-boundary",
            AnnotationPayload::Scene => "annotation.scene",
            AnnotationPayload::Beat { .. } => "annotation.beat",
            AnnotationPayload::Silence { .. } => "annotation.silence",
            AnnotationPayload::Filler { .. } => "annotation.filler",
            AnnotationPayload::Highlight { .. } => "annotation.highlight",
            AnnotationPayload::Review { .. } => "annotation.review",
        };
        found.insert(id.to_owned());
    }
    found
}

fn catalog_ids(coverage: &str) -> BTreeSet<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = root.join("examples/catalog/mechanisms/project-structure.json");
    let value: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    value["mechanisms"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["coverage"] == coverage)
        .map(|row| row["id"].as_str().unwrap().to_owned())
        .collect()
}
