use std::time::{Duration, Instant};

use veac_artifact::ContentDigest;

use super::{version, ProbeError, SystemFfprobe};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Exact identity of a pinned ffprobe executable snapshot.
pub struct FfprobeFingerprint {
    pub version: String,
    pub configuration: ContentDigest,
}

impl SystemFfprobe {
    pub fn fingerprint(&self) -> Result<FfprobeFingerprint, ProbeError> {
        let deadline =
            Instant::now() + Duration::from_secs(veac_artifact::MAX_MEDIA_DERIVATION_WALL_SECONDS);
        let pinned = self
            .pinned_until(deadline)
            .map_err(|error| version::tool_error(self.binary(), error))?;
        let version = self.version_until(pinned, deadline)?;
        let mut payload = b"veac.ffprobe-fingerprint.v1\0".to_vec();
        field(&mut payload, pinned.identity().digest.as_bytes());
        field(&mut payload, version.as_bytes());
        Ok(FfprobeFingerprint {
            version,
            configuration: ContentDigest::sha256(payload),
        })
    }
}

fn field(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(&(value.len() as u64).to_be_bytes());
    target.extend_from_slice(value);
}

#[cfg(test)]
#[path = "fingerprint/tests.rs"]
mod tests;
