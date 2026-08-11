use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::ProjectManifestV1;

pub fn canonical_manifest_bytes(
    manifest: &ProjectManifestV1,
) -> Result<Vec<u8>, serde_json::Error> {
    serde_json_canonicalizer::to_vec(manifest)
}

pub fn canonical_manifest_json(manifest: &ProjectManifestV1) -> Result<String, serde_json::Error> {
    serde_json_canonicalizer::to_string(manifest)
}

pub fn manifest_digest(manifest: &ProjectManifestV1) -> Result<String, serde_json::Error> {
    digest(manifest)
}

pub(crate) fn digest(value: &impl Serialize) -> Result<String, serde_json::Error> {
    let bytes = serde_json_canonicalizer::to_vec(value)?;
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(71);
    output.push_str("sha256:");
    for byte in digest {
        use std::fmt::Write;
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    Ok(output)
}
