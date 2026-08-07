use std::sync::Arc;

use super::nominal::nested_registry;
use crate::program::expression::{evaluate, ClosureValue, Environment, Value};

#[test]
fn stale_nominal_layout_nested_in_closure_capture_is_rejected() {
    let (stale_registry, _, child) = nested_registry("stale");
    let (current_registry, _, _) = nested_registry("current");
    let stale = Value::structure(&stale_registry, child, vec![Value::Integer(1)]).unwrap();
    let Value::Closure(template) =
        evaluate("fn() -> int effect pure { 1 }", &Environment::new()).unwrap()
    else {
        unreachable!()
    };
    let value = Value::Closure(Arc::new(ClosureValue::new(
        Arc::clone(template.definition()),
        vec![stale],
        Arc::clone(template.registry()),
        template.provenance().cloned(),
        0,
    )));

    let error = value
        .validate_nominal_registry(&current_registry)
        .unwrap_err();
    assert_eq!(error.code(), "VALUE_NOMINAL_DEFINITION");
}
