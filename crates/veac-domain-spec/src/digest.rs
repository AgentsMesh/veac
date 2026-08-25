use sha2::{Digest, Sha256};

use super::{DomainOperationContract, DomainOpsetVersion};

mod encoding;
use encoding::{
    axis, builtin_plugins, effect, exposure, framed, instruction, max_stage, plugin_identities,
    runtime_action, shape, temporal_lowering,
};

const DIGEST_DOMAIN: &[u8] = b"veac.domain-operation-registry.v5\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DomainRegistryDigest([u8; 32]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DomainPluginIdentity<'a> {
    effect_type: &'a str,
    digest: &'a str,
}

impl<'a> DomainPluginIdentity<'a> {
    pub const fn new(effect_type: &'a str, digest: &'a str) -> Self {
        Self {
            effect_type,
            digest,
        }
    }

    pub const fn effect_type(self) -> &'a str {
        self.effect_type
    }

    pub const fn digest(self) -> &'a str {
        self.digest
    }
}

impl DomainRegistryDigest {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Display for DomainRegistryDigest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

pub(super) fn registry<'a>(
    version: DomainOpsetVersion,
    values: impl ExactSizeIterator<Item = &'a DomainOperationContract>,
) -> DomainRegistryDigest {
    registry_with_plugins(version, values, builtin_plugins())
}

pub(crate) fn registry_with_plugins<'a>(
    version: DomainOpsetVersion,
    values: impl ExactSizeIterator<Item = &'a DomainOperationContract>,
    plugins: impl IntoIterator<Item = DomainPluginIdentity<'a>>,
) -> DomainRegistryDigest {
    let mut digest = Sha256::new();
    digest.update(DIGEST_DOMAIN);
    digest.update(version.raw().to_be_bytes());
    digest.update((values.len() as u64).to_be_bytes());
    for value in values {
        digest.update(value.id().opcode().to_be_bytes());
        framed(&mut digest, value.name().as_bytes());
        exposure(&mut digest, value.exposure());
        digest.update([
            instruction(value.instruction()),
            runtime_action(value.runtime_action()),
            effect(value.effect()),
            max_stage(value.max_stage()),
        ]);
        temporal_lowering(&mut digest, value.temporal_lowering());
        digest.update((value.operands().len() as u64).to_be_bytes());
        for operand in value.operands() {
            framed(&mut digest, operand.name().as_bytes());
            shape(&mut digest, operand.shape());
            digest.update([axis(operand.axis())]);
        }
        shape(&mut digest, value.result());
    }
    plugin_identities(&mut digest, plugins);
    DomainRegistryDigest::from_bytes(digest.finalize().into())
}

#[cfg(test)]
#[path = "digest/tests.rs"]
mod tests;
