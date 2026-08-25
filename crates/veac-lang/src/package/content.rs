use sha2::{Digest, Sha256};

use super::{LockedFile, PackageError, Sha256Digest};

const DOMAIN: &[u8] = b"veac.package.content.v1\0";

pub fn package_content_digest(files: &[LockedFile]) -> Result<Sha256Digest, PackageError> {
    super::validation::file_set(files)?;
    let mut digest = Sha256::new();
    digest.update(DOMAIN);
    digest.update((files.len() as u64).to_be_bytes());
    for file in files {
        framed(&mut digest, file.path.as_bytes());
        framed(&mut digest, file.sha256.as_str().as_bytes());
    }
    Ok(Sha256Digest::from_bytes(digest.finalize().into()))
}

pub fn sha256_bytes(value: &[u8]) -> Sha256Digest {
    Sha256Digest::from_bytes(Sha256::digest(value).into())
}

fn framed(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}
