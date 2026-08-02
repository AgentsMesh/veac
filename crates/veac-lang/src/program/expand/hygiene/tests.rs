use super::{encode, encode_path, InstancePath, Registry};
use crate::authoring::Span;

#[test]
fn byte_lengths_make_unicode_and_delimiter_boundaries_explicit() {
    assert_eq!(encode("片头", "a--b"), "veac-h-6-片头-4-a--b");
    assert_ne!(encode("a--b", "c"), encode("a", "b--c"));
    assert_eq!(
        encode_path("shot", &["card", "title"]),
        "veac-h-4-shot-4-card-5-title"
    );
}

#[test]
fn registry_rejects_explicit_names_registered_after_generation() {
    let mut registry = Registry::default();
    let value = registry
        .generated(
            "component.veac",
            &InstancePath::root("shot"),
            "title",
            Span::default(),
        )
        .unwrap();
    let error = registry
        .explicit("main.veac", &value, Span { start: 4, end: 8 })
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_HYGIENIC_ID_COLLISION");
    assert_eq!(error.path, "main.veac");
}

#[test]
fn registry_fails_closed_if_distinct_logical_ids_ever_encode_alike() {
    let mut registry = Registry::default();
    let value = encode("shot", "title");
    registry
        .generated
        .insert(value, InstancePath::root("other").child("identity"));
    let error = registry
        .generated(
            "main.veac",
            &InstancePath::root("shot"),
            "title",
            Span::default(),
        )
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_HYGIENIC_ID_COLLISION");
}

#[test]
fn explicit_source_propagates_lexer_errors() {
    let error = Registry::default().source("main.veac", "@").unwrap_err();
    assert_eq!(error.code, "PROGRAM_LEX_TOKEN");
}
