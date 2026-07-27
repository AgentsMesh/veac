use crate::authoring::TypedReference;

pub(super) fn quoted(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}

pub(super) fn reference(value: &TypedReference) -> String {
    format!("{} {}", value.kind.value, value.id.value)
}
