use std::collections::BTreeSet;

use crate::{project_json_schema, ParameterValue, ParameterValueKind};

#[test]
fn parameter_value_schema_exactly_matches_the_executable_union() {
    let schema = project_json_schema().unwrap();
    let variants = schema["$defs"]["ParameterValue"]["oneOf"]
        .as_array()
        .expect("tagged parameter union");
    let actual: BTreeSet<_> = variants
        .iter()
        .map(|variant| {
            variant["properties"]["type"]["const"]
                .as_str()
                .expect("parameter type tag")
        })
        .collect();
    let expected: BTreeSet<_> = ParameterValueKind::ALL
        .into_iter()
        .map(ParameterValueKind::schema_name)
        .collect();
    assert_eq!(actual.len(), variants.len(), "duplicate schema variant");
    assert_eq!(actual, expected);
}

#[test]
fn removed_parameter_variants_fail_canonical_deserialization() {
    for kind in ["integer", "vec2", "time", "text"] {
        let value = serde_json::json!({"type": kind, "value": 0});
        assert!(
            serde_json::from_value::<ParameterValue>(value).is_err(),
            "removed `{kind}` parameter unexpectedly decoded"
        );
    }
}
