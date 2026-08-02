use crate::authoring::TypedReference;

pub(super) fn quoted(value: &str) -> String {
    crate::string_codec::quote(value)
}

pub(super) fn reference(value: &TypedReference) -> String {
    format!("{} {}", value.kind.value, value.id.value)
}
