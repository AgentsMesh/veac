use crate::authoring::Span;
use crate::program::expression::ValueType;
use crate::program::model::ParameterDecl;

use super::*;

#[test]
fn no_parameter_frames_borrow_the_same_large_base_value() {
    let payload = Arc::new(Value::Text("x".repeat(1024 * 1024)));
    let base = ValueMap::from([("large".to_owned(), Arc::clone(&payload))]);
    let interface = ComponentInterface::from_members(&[], &[]);
    let frames = (0..64)
        .map(|_| BoundValues::new(&base, &interface))
        .collect::<Vec<_>>();
    for frame in frames {
        assert_eq!(frame.local.len(), 0);
        assert_eq!(frame.retained_bytes(), 0);
        assert!(std::ptr::eq(
            frame.value("large").unwrap(),
            payload.as_ref()
        ));
    }
}

#[test]
fn parameter_names_shadow_the_captured_base_until_resolved() {
    let base = ValueMap::from([("value".to_owned(), Arc::new(Value::Text("base".into())))]);
    let parameters = [parameter("value")];
    let interface = ComponentInterface::from_members(&parameters, &[]);
    let mut values = BoundValues::new(&base, &interface);
    assert!(values.value("value").is_none());
    values
        .insert("component.veac", "value", Value::Text("local".into()), 256)
        .unwrap();
    assert_eq!(values.value("value"), Some(&Value::Text("local".into())));
}

#[test]
fn overlay_budget_checks_each_insert_and_does_not_commit_failures() {
    let base = ValueMap::new();
    let parameters = [parameter("first"), parameter("second")];
    let interface = ComponentInterface::from_members(&parameters, &[]);
    let mut values = BoundValues::new(&base, &interface);
    let exact = ENTRY_BYTES + "first".len() + 3;
    values
        .insert("component.veac", "first", Value::Text("one".into()), exact)
        .unwrap();
    let error = values
        .insert("component.veac", "second", Value::Text("two".into()), exact)
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_EXPANSION_BUDGET");
    assert_eq!(values.local.len(), 1);
    assert_eq!(values.retained_bytes(), exact);
}

fn parameter(name: &str) -> ParameterDecl {
    ParameterDecl {
        name: name.to_owned(),
        value_type: ValueType::Text,
        default: None,
        span: Span::default(),
    }
}
