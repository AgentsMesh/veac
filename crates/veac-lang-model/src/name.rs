pub(crate) fn is_name(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value.chars().next().is_some_and(is_start)
        && value.chars().all(is_continue)
        && !value.ends_with('-')
        && !value.contains("--")
        && !matches!(value, "true" | "false")
}

pub(crate) fn is_qualified_name(value: &str) -> bool {
    value.split('.').all(is_name)
}

fn is_start(value: char) -> bool {
    value.is_ascii_alphabetic() || value == '_'
}

fn is_continue(value: char) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, '_' | '-')
}
