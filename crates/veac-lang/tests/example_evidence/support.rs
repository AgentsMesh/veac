use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use veac_ir::{Clip, ClipSource, ProjectEnvelope, Sequence, TextStyle};
use veac_lang::authoring::{lower_document, parse};

#[derive(Debug)]
pub struct Claim {
    pub id: String,
    pub preview_target: String,
    pub registry_key: Option<String>,
}

pub fn assert_preview_evidence(
    catalog_name: &str,
    extract: fn(&ProjectEnvelope) -> BTreeSet<String>,
) {
    let claims = preview_claims(catalog_name);
    assert!(
        !claims.is_empty(),
        "{catalog_name} has no preview_required rows"
    );
    let mut evidence = BTreeMap::new();
    let mut missing = Vec::new();
    for claim in claims {
        let present = evidence
            .entry(claim.preview_target.clone())
            .or_insert_with(|| extract(&lower_target(&claim.preview_target)));
        if !present.contains(&claim.id) {
            missing.push(format!("{} @ {}", claim.id, claim.preview_target));
        }
    }
    assert!(
        missing.is_empty(),
        "preview_required mechanisms lack typed lowered-IR evidence:\n{}",
        missing.join("\n")
    );
}

pub fn entry_sequence(envelope: &ProjectEnvelope) -> &Sequence {
    envelope
        .project
        .sequences
        .iter()
        .find(|sequence| sequence.id == envelope.project.entry_sequence_id)
        .expect("entry sequence")
}

pub fn clips(envelope: &ProjectEnvelope) -> impl Iterator<Item = &Clip> {
    entry_sequence(envelope)
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
}

pub fn text_styles(envelope: &ProjectEnvelope) -> impl Iterator<Item = &TextStyle> {
    clips(envelope).filter_map(|clip| match &clip.source {
        ClipSource::Text { style, .. } | ClipSource::Caption { style, .. } => Some(style),
        _ => None,
    })
}

#[test]
fn every_preview_catalog_is_backed_by_a_typed_extractor() {
    const TYPED: &[&str] = &[
        "effects.json",
        "generators.json",
        "project-structure.json",
        "text-color.json",
        "timing-audio.json",
        "transitions-composition.json",
    ];
    let capabilities = read_json(&examples_root().join("capabilities.json"));
    for path in required_array(&capabilities, "mechanism_catalogs") {
        let path = path
            .as_str()
            .expect("mechanism catalog path must be a string");
        let name = Path::new(path)
            .file_name()
            .and_then(|value| value.to_str())
            .expect("mechanism catalog path must end in a UTF-8 file name");
        if !preview_claims(name).is_empty() {
            assert!(TYPED.contains(&name), "{name} lacks a typed extractor");
        }
    }
}

pub fn preview_claims(catalog_name: &str) -> Vec<Claim> {
    let catalog = read_json(
        &examples_root()
            .join("catalog/mechanisms")
            .join(catalog_name),
    );
    required_array(&catalog, "mechanisms")
        .iter()
        .filter(|row| required_str(row, "coverage") == "preview_required")
        .map(|row| Claim {
            id: required_str(row, "id").to_owned(),
            preview_target: required_str(row, "preview_target").to_owned(),
            registry_key: row
                .get("registry_key")
                .and_then(Value::as_str)
                .map(str::to_owned),
        })
        .collect()
}

fn lower_target(target_id: &str) -> ProjectEnvelope {
    let gallery = read_json(&examples_root().join("catalog/gallery.json"));
    let row = required_array(&gallery, "targets")
        .iter()
        .find(|row| required_str(row, "id") == target_id)
        .unwrap_or_else(|| panic!("unknown gallery target {target_id}"));
    let relative = required_str(row, "example");
    let relative = relative
        .strip_prefix("examples/")
        .unwrap_or_else(|| panic!("gallery target {target_id} has invalid source {relative}"));
    lower_example(relative)
}

pub fn lower_example(relative: &str) -> ProjectEnvelope {
    let path = examples_root().join(relative);
    let source = fs::read_to_string(&path).unwrap();
    let document =
        parse(&source).unwrap_or_else(|diagnostics| panic!("{}: {diagnostics:?}", path.display()));
    let envelope = lower_document(&document)
        .unwrap_or_else(|diagnostics| panic!("{}: {diagnostics:?}", path.display()));
    veac_ir::validate(&envelope)
        .unwrap_or_else(|diagnostics| panic!("{}: {diagnostics:?}", path.display()));
    envelope
}

fn required_array<'a>(value: &'a Value, field: &str) -> &'a [Value] {
    value
        .get(field)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("catalog field {field} must be an array"))
}

fn required_str<'a>(value: &'a Value, field: &str) -> &'a str {
    value
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("catalog field {field} must be a string"))
}

fn read_json(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap())
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn examples_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}
