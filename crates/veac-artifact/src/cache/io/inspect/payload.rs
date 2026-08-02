use sha2::{Digest, Sha256};

use crate::{ArtifactRecord, ArtifactResult, ContentDigest, DigestAlgorithm};

pub(super) fn verify_bytes_while(
    record: &ArtifactRecord,
    payload: &[u8],
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<()> {
    let mut digest = Sha256::new();
    for chunk in payload.chunks(64 * 1024) {
        checked(&mut guard)?;
        digest.update(chunk);
        checked(&mut guard)?;
    }
    checked(&mut guard)?;
    let content = ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: format!("{:x}", digest.finalize()),
    };
    if record.content != content || record.size_bytes != payload.len() as u64 {
        return super::corrupt("cached artifact payload failed identity verification");
    }
    Ok(())
}

fn checked(guard: &mut impl FnMut() -> bool) -> ArtifactResult<()> {
    crate::cache::guard::check(guard)
}
