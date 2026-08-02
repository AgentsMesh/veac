use std::borrow::Cow;
use std::collections::BTreeMap;

use super::expand_source;
use crate::program::expand::budget::MAX_EXPANDED_BYTES;
use crate::program::model::PresetKind;

#[test]
fn repeated_large_preset_bodies_fail_without_cloning_each_body() {
    let key = ("large".to_owned(), PresetKind::TextStyle);
    let presets = BTreeMap::from([(key, "x".repeat(2 * 1024 * 1024))]);
    let source = "use text-style large;".repeat(17);
    let error = expand_source(
        "test.veac",
        Cow::Borrowed(&source),
        &presets,
        MAX_EXPANDED_BYTES,
    )
    .unwrap_err();
    assert_eq!(error.code, "PROGRAM_EXPANSION_BUDGET");
}
