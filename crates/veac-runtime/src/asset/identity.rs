use std::path::Path;

use veac_ir::MediaIdentity;

use super::ProbeError;

/// Compute the lowercase SHA-256 identity pinned by the current probe implementation.
pub fn sha256_identity(path: &Path) -> Result<MediaIdentity, ProbeError> {
    veac_artifact::verify_source(path, None)
        .map(|verified| verified.identity)
        .map_err(|error| ProbeError::Io {
            operation: "verify for hashing",
            path: path.to_path_buf(),
            source: std::io::Error::other(error),
        })
}
