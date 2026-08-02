use std::collections::BTreeMap;

use super::inject_with_limit;
use crate::program::expand::budget::MAX_EXPANDED_BYTES;
use crate::program::expand::{hygiene, text};
use crate::program::expression::Environment;
use crate::program::model::PresetMap;

#[test]
fn repeated_large_slot_fills_fail_without_cloning_each_fill() {
    let fills = BTreeMap::from([("picture".to_owned(), "x".repeat(2 * 1024 * 1024))]);
    let source = "source slot picture;".repeat(17);
    let error = inject_with_limit("test.veac", source, &fills, MAX_EXPANDED_BYTES).unwrap_err();
    assert_eq!(error.code, "PROGRAM_EXPANSION_BUDGET");
}

#[test]
fn no_op_expansion_stages_reuse_the_materialized_source() {
    let presets = PresetMap::new();
    let values = Environment::new();
    let expanded = text::expand_with_limit(
        "test.veac",
        "source generated transparent;",
        &presets,
        &values,
        0,
        MAX_EXPANDED_BYTES,
    )
    .unwrap();
    let pointer = expanded.as_ptr();
    let mut registry = hygiene::Registry::default();
    let path = hygiene::InstancePath::root("root");
    let hygienic = hygiene::expand_with_limit(
        "test.veac",
        expanded,
        &path,
        &mut registry,
        MAX_EXPANDED_BYTES,
    )
    .unwrap();
    assert_eq!(hygienic.as_ptr(), pointer);
    let injected =
        inject_with_limit("test.veac", hygienic, &BTreeMap::new(), MAX_EXPANDED_BYTES).unwrap();
    assert_eq!(injected.as_ptr(), pointer);
}
