use super::super::{ListValue, MapValue, TupleValue, Value};

pub(super) fn expression(value: &Value) -> String {
    match value {
        Value::Identifier(value) => {
            format!("identifier({})", crate::string_codec::quote(value))
        }
        Value::List(value) => list(value, expression),
        Value::Map(value) => map(value, expression),
        Value::Tuple(value) => tuple(value, expression),
        _ => value.render(),
    }
}

pub(super) fn list(value: &ListValue, render: fn(&Value) -> String) -> String {
    if value.values().is_empty() {
        return typed_empty(value.value_type(), "[]");
    }
    sequence("[", "]", value.values(), render)
}

pub(super) fn tuple(value: &TupleValue, render: fn(&Value) -> String) -> String {
    sequence("(", ")", value.values(), render)
}

pub(super) fn map(value: &MapValue, render: fn(&Value) -> String) -> String {
    if value.entries().is_empty() {
        return typed_empty(value.value_type(), "#{}");
    }
    let entries = value
        .entries()
        .iter()
        .map(|entry| format!("{}: {}", render(entry.key()), render(entry.value())))
        .collect::<Vec<_>>()
        .join(", ");
    format!("#{{ {entries} }}")
}

fn sequence(
    opening: &str,
    closing: &str,
    values: &[Value],
    render: fn(&Value) -> String,
) -> String {
    let values = values.iter().map(render).collect::<Vec<_>>().join(", ");
    format!("{opening}{values}{closing}")
}

fn typed_empty(value_type: &crate::program::expression::ValueType, literal: &str) -> String {
    format!("{{ let value: {value_type} = {literal}; value }}")
}
