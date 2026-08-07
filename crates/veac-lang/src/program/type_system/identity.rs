use std::fmt;
use std::sync::Arc;

use sha2::{Digest, Sha256};

const ID_DOMAIN: &[u8] = b"veac.nominal-type-identity.v1\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId([u8; 32]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeDefinitionDigest([u8; 32]);

#[derive(Debug, Clone)]
pub struct TypeRef {
    id: TypeId,
    diagnostic_name: Arc<str>,
}

impl TypeId {
    pub fn derive(canonical_source_id: &str, declared_name: &str) -> Self {
        let mut digest = Sha256::new();
        digest.update(ID_DOMAIN);
        framed(&mut digest, canonical_source_id.as_bytes());
        framed(&mut digest, declared_name.as_bytes());
        Self(digest.finalize().into())
    }

    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl TypeDefinitionDigest {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl TypeRef {
    pub fn new(id: TypeId, diagnostic_name: impl Into<Arc<str>>) -> Self {
        Self {
            id,
            diagnostic_name: diagnostic_name.into(),
        }
    }

    pub const fn id(&self) -> TypeId {
        self.id
    }

    pub fn diagnostic_name(&self) -> &str {
        &self.diagnostic_name
    }

    pub fn with_diagnostic_name(&self, value: impl Into<Arc<str>>) -> Self {
        Self::new(self.id, value)
    }
}

impl PartialEq for TypeRef {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TypeRef {}

impl PartialOrd for TypeRef {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TypeRef {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl std::hash::Hash for TypeRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl fmt::Display for TypeRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.diagnostic_name)
    }
}

impl fmt::Display for TypeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        hex(formatter, &self.0)
    }
}

impl fmt::Display for TypeDefinitionDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        hex(formatter, &self.0)
    }
}

fn framed(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

fn hex(formatter: &mut fmt::Formatter<'_>, bytes: &[u8; 32]) -> fmt::Result {
    for byte in bytes {
        write!(formatter, "{byte:02x}")?;
    }
    Ok(())
}
