use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum DigestAlgorithm {
    Sha256,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContentDigest {
    pub algorithm: DigestAlgorithm,
    pub value: String,
}

impl ContentDigest {
    pub fn sha256(bytes: impl AsRef<[u8]>) -> Self {
        Self {
            algorithm: DigestAlgorithm::Sha256,
            value: hex(Sha256::digest(bytes.as_ref())),
        }
    }

    pub fn validate(&self) -> ArtifactResult<()> {
        if self.value.len() == 64
            && self
                .value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            Ok(())
        } else {
            Err(ArtifactError::new(
                ArtifactErrorKind::InvalidContract,
                "digest must contain 64 lowercase hexadecimal characters",
            ))
        }
    }
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.as_ref().len() * 2);
    for byte in bytes.as_ref() {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
