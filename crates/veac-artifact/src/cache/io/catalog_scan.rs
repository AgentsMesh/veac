use std::ffi::{OsStr, OsString};
use std::path::Path;

use super::authority::BoundChain;
use super::common::corrupt;
use super::directory::{directory_entries, BoundDirectory};
use super::lock::DirectoryLock;
use super::state;
use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult, ContentDigest, DigestAlgorithm};

const MAX_CATALOG_ARTIFACTS: usize = 4_096;

pub(in crate::cache) fn catalog_keys(root: &Path) -> ArtifactResult<Vec<ContentDigest>> {
    catalog_keys_with(root, |_| {})
}

pub(super) fn catalog_keys_with(
    root: &Path,
    after_digest_open: impl FnOnce(&Path),
) -> ArtifactResult<Vec<ContentDigest>> {
    let Some(mut digest) = BoundChain::root(root, false)? else {
        return Ok(Vec::new());
    };
    let root_entries = directory_entries(digest.current(), 2)?;
    if root_entries.is_empty() {
        return Ok(Vec::new());
    }
    if root_entries != [OsString::from("sha256")] {
        return corrupt("artifact store root contains unexpected entries");
    }
    if !digest.extend(&[OsString::from("sha256")], false)? {
        return corrupt("artifact digest directory disappeared while cataloging");
    }
    after_digest_open(digest.path());
    digest.verify()?;
    let prefixes = directory_entries(digest.current(), 257)?;
    let mut keys = Vec::new();
    for prefix in prefixes {
        require_hex(&prefix, 2)?;
        let mut prefix_chain = digest.try_clone()?;
        if !prefix_chain.extend(std::slice::from_ref(&prefix), false)? {
            return corrupt("artifact prefix disappeared while cataloging");
        }
        prefix_chain.verify()?;
        let _lock = DirectoryLock::shared_while(prefix_chain.current(), || true)?;
        prefix_chain.verify()?;
        let available = MAX_CATALOG_ARTIFACTS.saturating_sub(keys.len());
        let suffixes = directory_entries(prefix_chain.current(), available)?;
        for suffix in suffixes {
            if is_orphan_name(&suffix) {
                require_orphan(&prefix_chain, &suffix)?;
                continue;
            }
            require_hex(&suffix, 62)?;
            let artifact = BoundDirectory::open(prefix_chain.try_clone()?, suffix.clone())?;
            let Some(artifact) = artifact else {
                return corrupt("artifact disappeared while cataloging");
            };
            if state::marker(&artifact)?.is_none() {
                continue;
            }
            artifact.verify()?;
            keys.push(ContentDigest {
                algorithm: DigestAlgorithm::Sha256,
                value: format!("{}{}", text(&prefix)?, text(&suffix)?),
            });
        }
        prefix_chain.verify()?;
    }
    digest.verify()?;
    keys.sort_by(|left, right| left.value.cmp(&right.value));
    Ok(keys)
}

fn require_orphan(prefix: &BoundChain, name: &OsStr) -> ArtifactResult<()> {
    let directory = BoundDirectory::open(prefix.try_clone()?, name.to_owned())?
        .ok_or_else(|| ArtifactError::new(ArtifactErrorKind::CorruptCache, "orphan vanished"))?;
    if state::marker(&directory)?.is_some() {
        return corrupt("private cache orphan unexpectedly contains a commit marker");
    }
    directory.verify()
}

fn is_orphan_name(name: &OsStr) -> bool {
    let Some(name) = name.to_str() else {
        return false;
    };
    let Some(suffix) = name.strip_prefix(".veac-cache-") else {
        return false;
    };
    suffix.len() == 32
        && suffix
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn require_hex(name: &OsStr, length: usize) -> ArtifactResult<()> {
    let name = text(name)?;
    if name.len() != length
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return corrupt("artifact catalog contains a malformed digest path");
    }
    Ok(())
}

fn text(name: &OsStr) -> ArtifactResult<&str> {
    name.to_str().ok_or_else(|| {
        ArtifactError::new(
            ArtifactErrorKind::CorruptCache,
            "artifact store entry names must be UTF-8",
        )
    })
}

#[cfg(test)]
#[path = "catalog_scan/tests.rs"]
mod tests;
