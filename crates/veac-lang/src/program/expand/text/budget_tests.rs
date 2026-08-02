use std::collections::BTreeMap;

use super::expand_with_limit;
use crate::program::expression::Value;
use crate::program::model::PresetMap;

#[test]
fn expression_values_are_rejected_as_soon_as_the_final_budget_is_exhausted() {
    let values = BTreeMap::from([("value".to_owned(), Value::Text("x".repeat(400)))]);
    let presets = PresetMap::new();
    let source = "${value}".repeat(3);
    let error = expand_with_limit("test.veac", &source, &presets, &values, 0, 1_000).unwrap_err();
    assert_eq!(error.code, "PROGRAM_EXPANSION_BUDGET");
}
