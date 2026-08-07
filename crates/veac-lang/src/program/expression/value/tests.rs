use std::sync::Arc;

use super::{ListValue, MapValue, MapValueEntry, TupleValue, Value};
use crate::program::expression::execution_budget::LOGICAL_COLLECTION_HANDLE_BYTES;
use crate::program::expression::{MapKeyType, PrimitiveType, ValueType, ValueTypeKind};

#[path = "tests/domain.rs"]
mod domain;
#[path = "tests/nominal.rs"]
mod nominal;
#[path = "tests/nominal_accessors.rs"]
mod nominal_accessors;
#[path = "tests/nominal_capture.rs"]
mod nominal_capture;
#[path = "tests/nominal_function.rs"]
mod nominal_function;
#[path = "tests/value_edges.rs"]
mod value_edges;

fn integer(value: i64) -> Value {
    Value::Integer(value)
}

fn text(value: &str) -> Value {
    Value::Text(value.into())
}

#[test]
fn lists_retain_the_resolved_type_and_validate_every_element() {
    let element = ValueType::primitive(PrimitiveType::Integer);
    let list = ListValue::new(element.clone(), vec![integer(1), integer(2)]).unwrap();
    assert_eq!(list.values(), [integer(1), integer(2)]);
    assert!(matches!(list.value_type().kind(), ValueTypeKind::List(found) if found == &element));
    let error = Value::list(element, vec![text("wrong")]).unwrap_err();
    assert_eq!(error.code(), "VALUE_LIST_ELEMENT_TYPE");

    let empty = Value::List(Arc::new(
        ListValue::new(ValueType::primitive(PrimitiveType::Text), Vec::new()).unwrap(),
    ));
    assert!(matches!(empty.kind().kind(), ValueTypeKind::List(_)));
    assert_eq!(empty.evaluated_bytes(), 0);
}

#[test]
fn tuples_reject_invalid_arity_and_account_for_nested_payloads() {
    assert_eq!(
        Value::tuple(Vec::new()).unwrap_err().code(),
        "VALUE_TYPE_TUPLE_ARITY"
    );
    assert!(TupleValue::new(vec![integer(1)]).is_err());
    let tuple = Value::Tuple(Arc::new(
        TupleValue::new(vec![text("ab"), text("c")]).unwrap(),
    ));
    assert_eq!(
        tuple.retained_bytes(),
        2 * LOGICAL_COLLECTION_HANDLE_BYTES + 3
    );
    assert_eq!(tuple.render(), r#"("ab", "c")"#);
}

#[test]
fn maps_sort_utf8_keys_and_reject_duplicates_before_publication() {
    let entries = vec![
        MapValueEntry::new(text("é"), integer(2)),
        MapValueEntry::new(text("z"), integer(1)),
    ];
    let map = MapValue::new(
        MapKeyType::Text,
        ValueType::primitive(PrimitiveType::Integer),
        entries,
    )
    .unwrap();
    assert_eq!(map.entries()[0].key(), &text("z"));
    assert_eq!(map.entries()[1].key(), &text("é"));
    let rendered = Value::Map(Arc::new(map)).render();
    assert_eq!(rendered, "#{ \"z\": 1, \"é\": 2 }");

    let duplicate = vec![
        MapValueEntry::new(text("same"), integer(1)),
        MapValueEntry::new(text("same"), integer(2)),
    ];
    let error = MapValue::new(
        MapKeyType::Text,
        ValueType::primitive(PrimitiveType::Integer),
        duplicate,
    )
    .unwrap_err();
    assert_eq!(error.code(), "VALUE_MAP_DUPLICATE_KEY");
}

#[test]
fn identifier_keys_render_as_parseable_constructor_calls() {
    let entries = vec![MapValueEntry::new(
        Value::Identifier("asset".into()),
        integer(1),
    )];
    let map = MapValue::new(
        MapKeyType::Identifier,
        ValueType::primitive(PrimitiveType::Integer),
        entries,
    )
    .unwrap();
    assert_eq!(
        Value::Map(Arc::new(map)).render(),
        "#{ identifier(\"asset\"): 1 }"
    );
}

#[test]
fn variable_width_primitives_clone_only_immutable_handles() {
    let color = Value::Color(Arc::from("#12345678"));
    let identifier = Value::Identifier(Arc::from("cover-art"));
    let (Value::Color(color), Value::Color(color_clone)) = (color.clone(), color) else {
        unreachable!()
    };
    let (Value::Identifier(identifier), Value::Identifier(identifier_clone)) =
        (identifier.clone(), identifier)
    else {
        unreachable!()
    };
    assert!(Arc::ptr_eq(&color, &color_clone));
    assert!(Arc::ptr_eq(&identifier, &identifier_clone));
}
