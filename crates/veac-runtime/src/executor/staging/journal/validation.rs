use std::path::{Component, Path};

use super::{Journal, SCHEMA_VERSION};
use crate::RuntimeError;

pub(super) fn validate(journal: &Journal) -> Result<(), RuntimeError> {
    if journal.schema_version != SCHEMA_VERSION || journal.entries.is_empty() {
        return invalid("unsupported or empty commit journal");
    }
    let mut previous = None;
    for entry in &journal.entries {
        if !safe_name(&entry.target)
            || entry
                .source
                .as_deref()
                .is_some_and(|value| !safe_name(value))
            || entry.source.is_some() != entry.source_identity.is_some()
            || previous.is_some_and(|value| value >= entry.target.as_str())
        {
            return invalid(
                "commit journal paths must be paired, sorted, unique, and single-component",
            );
        }
        previous = Some(entry.target.as_str());
    }
    Ok(())
}

fn safe_name(value: &str) -> bool {
    !value.is_empty()
        && Path::new(value).components().count() == 1
        && matches!(
            Path::new(value).components().next(),
            Some(Component::Normal(_))
        )
}

pub(super) fn invalid<T>(message: &str) -> Result<T, RuntimeError> {
    Err(RuntimeError::new(format!(
        "invalid render commit journal: {message}"
    )))
}
