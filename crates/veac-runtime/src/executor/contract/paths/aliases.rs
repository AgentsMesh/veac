use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use tempfile::Builder;
use unicode_normalization::UnicodeNormalization;

use super::declaration::{self, Declaration};
use crate::executor::contract::invalid;
use crate::RuntimeError;

#[derive(Debug, Clone, Copy)]
struct Policy {
    case_insensitive: bool,
    normalization_insensitive: bool,
}

pub(super) fn validate(
    declarations: &[Declaration],
    protected: &BTreeSet<PathBuf>,
) -> Result<(), RuntimeError> {
    let policies = policies(declarations)?;
    validate_with(declarations, protected, &policies)
}

fn policies(declarations: &[Declaration]) -> Result<BTreeMap<PathBuf, Policy>, RuntimeError> {
    declaration::parents(declarations)
        .into_iter()
        .map(|parent| Ok((parent.clone(), detect(&parent)?)))
        .collect()
}

fn detect(parent: &Path) -> Result<Policy, RuntimeError> {
    let probe = Builder::new()
        .prefix(".veac-path-probe-")
        .tempdir_in(parent)
        .map_err(probe_error)?;
    Ok(Policy {
        case_insensitive: alias(probe.path(), "VEAC-Case-A", "veac-case-a")?,
        normalization_insensitive: alias(probe.path(), "v\u{e9}ac", "ve\u{301}ac")?,
    })
}

fn alias(parent: &Path, authored: &str, alternate: &str) -> Result<bool, RuntimeError> {
    fs::write(parent.join(authored), b"probe").map_err(probe_error)?;
    match fs::symlink_metadata(parent.join(alternate)) {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(probe_error(error)),
    }
}

fn validate_with(
    declarations: &[Declaration],
    protected: &BTreeSet<PathBuf>,
    policies: &BTreeMap<PathBuf, Policy>,
) -> Result<(), RuntimeError> {
    let folded = declarations
        .iter()
        .map(|value| fold(value, policies.get(&declaration::parent(value)).unwrap()))
        .collect::<Vec<_>>();
    for (index, value) in folded.iter().enumerate() {
        if reserved(value) {
            return invalid("backend outputs may not use the reserved .veac-* namespace");
        }
        if folded[..index]
            .iter()
            .any(|other| declaration::overlaps(other, value))
        {
            return invalid("backend outputs alias under the destination filesystem policy");
        }
        if protected.iter().any(|path| {
            policies
                .get(path.parent().unwrap_or(Path::new(".")))
                .is_some_and(|policy| declaration::consumes(value, &fold_path(path, policy)))
        }) {
            return invalid("backend output aliases a protected resource on this filesystem");
        }
    }
    Ok(())
}

fn fold(value: &Declaration, policy: &Policy) -> Declaration {
    match value {
        Declaration::Static(path) => Declaration::Static(fold_path(path, policy)),
        Declaration::Pattern {
            parent,
            prefix,
            suffix,
            minimum_width,
        } => Declaration::Pattern {
            parent: parent.clone(),
            prefix: fold_bytes(prefix, policy),
            suffix: fold_bytes(suffix, policy),
            minimum_width: *minimum_width,
        },
        Declaration::Passlog { parent, prefix } => Declaration::Passlog {
            parent: parent.clone(),
            prefix: fold_bytes(prefix, policy),
        },
    }
}

fn fold_path(path: &Path, policy: &Policy) -> PathBuf {
    path.parent().unwrap_or(Path::new(".")).join(fold_name(
        path.file_name().unwrap().to_str().unwrap(),
        policy,
    ))
}

fn fold_bytes(value: &[u8], policy: &Policy) -> Vec<u8> {
    fold_name(std::str::from_utf8(value).unwrap(), policy).into_bytes()
}

fn fold_name(value: &str, policy: &Policy) -> String {
    let value = if policy.case_insensitive {
        value.chars().flat_map(char::to_lowercase).collect()
    } else {
        value.to_owned()
    };
    if policy.normalization_insensitive {
        value.nfc().collect()
    } else {
        value
    }
}

fn reserved(value: &Declaration) -> bool {
    match value {
        Declaration::Static(path) => reserved_prefix(path.file_name().unwrap().as_encoded_bytes()),
        Declaration::Pattern { prefix, .. } | Declaration::Passlog { prefix, .. } => {
            reserved_prefix(prefix)
        }
    }
}

fn reserved_prefix(value: &[u8]) -> bool {
    value
        .get(..6)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b".veac-"))
}

fn probe_error(error: std::io::Error) -> RuntimeError {
    RuntimeError::new(format!(
        "cannot probe output filesystem alias policy: {error}"
    ))
}

#[cfg(test)]
mod tests;
