use crate::program::expression::{
    evaluate, Environment, MapKeyType, PrimitiveType, Value, ValueConstructionError, ValueType,
};

#[test]
fn range_accessors_render_default_and_explicit_steps() {
    let forward = Value::range(1, 4, 1).unwrap();
    let Value::Range(forward) = forward else {
        unreachable!()
    };
    assert_eq!(forward.start(), 1);
    assert_eq!(forward.end(), 4);
    assert_eq!(forward.step(), 1);
    assert_eq!(forward.count(), 3);
    assert_eq!(forward.render(), "1 .. 4");

    let reverse = Value::range(5, -2, -2).unwrap();
    assert_eq!(reverse.render(), "5 .. -2 by -2");
    assert_eq!(
        Value::range(0, 1, 0).unwrap_err().code(),
        "VALUE_RANGE_STEP"
    );
}

#[test]
fn closure_equality_is_bound_to_verified_definition_and_registry_identity() {
    let value = evaluate("fn() -> int effect pure { 1 }", &Environment::new()).unwrap();
    let same = value.clone();
    assert_eq!(value, same);
    let other = evaluate("fn() -> int effect pure { 1 }", &Environment::new()).unwrap();
    assert_ne!(value, other);
    let Value::Closure(value) = value else {
        unreachable!()
    };
    assert_eq!(
        value.logical_capture_bytes(),
        crate::program::expression::execution_budget::LOGICAL_CLOSURE_BASE_BYTES
    );
}

#[test]
fn structural_errors_and_map_retention_expose_stable_diagnostics() {
    let error = ValueConstructionError::new("VALUE_TEST", "invalid aggregate");
    assert_eq!(error.code(), "VALUE_TEST");
    assert_eq!(error.message(), "invalid aggregate");
    assert_eq!(error.to_string(), "invalid aggregate");

    let map = Value::map(
        MapKeyType::Text,
        ValueType::primitive(PrimitiveType::Text),
        vec![(Value::Text("k".into()), Value::Text("payload".into()))],
    )
    .unwrap();
    assert!(map.retained_bytes() > "k".len() + "payload".len());
}
