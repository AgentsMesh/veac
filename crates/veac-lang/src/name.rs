pub(crate) const NAME_CONTRACT: &str =
    "1..128 ASCII bytes matching [A-Za-z_][A-Za-z0-9_]*(-[A-Za-z0-9_]+)*; `true` and `false` are reserved";

pub(crate) fn is_name(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value.chars().next().is_some_and(is_name_start)
        && value.chars().all(is_name_continue)
        && !value.ends_with('-')
        && !value.contains("--")
        && !matches!(value, "true" | "false")
}

pub(crate) fn is_qualified_name(value: &str) -> bool {
    value.split('.').all(is_name)
}

pub(crate) fn is_name_start(value: char) -> bool {
    value.is_ascii_alphabetic() || value == '_'
}

pub(crate) fn is_name_continue(value: char) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, '_' | '-')
}

#[cfg(test)]
mod tests;
