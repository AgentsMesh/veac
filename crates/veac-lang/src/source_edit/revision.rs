use std::collections::BTreeSet;

use sha2::{Digest, Sha256};

use super::{SourceEditError, SourceRevision};

const HASH_DOMAIN: &[u8] = b"veac-source-graph-v1\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceModule<'a> {
    pub path: &'a str,
    pub bytes: &'a [u8],
}

impl<'a> SourceModule<'a> {
    pub fn new(path: &'a str, bytes: &'a [u8]) -> Self {
        Self { path, bytes }
    }

    pub fn utf8(path: &'a str, source: &'a str) -> Self {
        Self::new(path, source.as_bytes())
    }
}

pub fn source_graph_revision(
    modules: &[SourceModule<'_>],
) -> Result<SourceRevision, SourceEditError> {
    if modules.is_empty() {
        return Err(SourceEditError::EmptySourceGraph);
    }
    let mut ordered = modules.to_vec();
    ordered.sort_unstable_by(|left, right| left.path.cmp(right.path));
    let mut seen = BTreeSet::new();
    let mut hash = Sha256::new();
    hash.update(HASH_DOMAIN);
    for module in ordered {
        validate_module_path(module.path)?;
        if !seen.insert(module.path) {
            return Err(SourceEditError::DuplicateModulePath(module.path.to_owned()));
        }
        hash_framed(&mut hash, module.path.as_bytes());
        hash_framed(&mut hash, module.bytes);
    }
    Ok(SourceRevision {
        source_graph_sha256: hex(hash.finalize()),
    })
}

pub fn validate_module_path(path: &str) -> Result<(), SourceEditError> {
    let segments = path.split('/');
    let invalid = path.is_empty()
        || path.len() > 4096
        || path.starts_with('/')
        || path.contains('\0')
        || path.contains('\\')
        || path.chars().any(char::is_control)
        || segments.into_iter().any(|value| {
            value.is_empty()
                || matches!(value, "." | ".." | ".veac-source.lock")
                || value.contains(':')
        });
    if invalid {
        Err(SourceEditError::InvalidModulePath(path.to_owned()))
    } else {
        Ok(())
    }
}

pub fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn hash_framed(hash: &mut Sha256, value: &[u8]) {
    hash.update((value.len() as u64).to_be_bytes());
    hash.update(value);
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.as_ref().len() * 2);
    for byte in bytes.as_ref() {
        result.push(char::from(HEX[usize::from(byte >> 4)]));
        result.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    result
}
