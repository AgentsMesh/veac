use super::*;

#[test]
fn owner_path_helpers_require_exact_closed_shapes() {
    let item = item_path(&["project", "sequence", "layer", "item"]).unwrap();
    assert_eq!(item.item, "item");
    assert_eq!(
        item_path(&["project"]).unwrap_err().reason_code(),
        "EXECUTABLE_TEMPORAL_OWNER"
    );

    let apply = apply_path(&["project", "sequence", "apply"]).unwrap();
    assert_eq!(apply.apply, "apply");
    assert_eq!(
        apply_path(&["project", "sequence"])
            .unwrap_err()
            .reason_code(),
        "EXECUTABLE_TEMPORAL_OWNER"
    );
}

#[test]
fn selector_helpers_reject_wrong_types_ranges_and_arity() {
    assert_eq!(ordinal(&[Value::Integer(3)]).unwrap(), 3);
    for values in [
        vec![],
        vec![Value::Text("bad".into())],
        vec![Value::Integer(-1)],
    ] {
        assert_eq!(
            ordinal(&values).unwrap_err().reason_code(),
            "EXECUTABLE_TEMPORAL_SELECTOR"
        );
    }
    let selectors = [Value::Identifier("blur".into()), Value::Integer(1)];
    assert_eq!(identifier(&selectors, 0).unwrap(), "blur");
    for index in [1, 2] {
        assert_eq!(
            identifier(&selectors, index).unwrap_err().reason_code(),
            "EXECUTABLE_TEMPORAL_SELECTOR"
        );
    }
}
