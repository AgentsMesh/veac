use serde::Serialize;

use super::ProgramIdentity;

impl ProgramIdentity {
    pub(crate) const fn core_version(&self) -> u16 {
        self.core_version
    }

    pub(crate) const fn domain_opset(&self) -> u16 {
        self.domain_opset
    }

    pub(crate) fn domain_registry_sha256(&self) -> &str {
        &self.domain_registry_sha256
    }

    pub(crate) fn main_content_sha256(&self) -> &str {
        &self.main_content_sha256
    }

    pub(crate) fn source_graph_sha256(&self) -> &str {
        &self.source_graph_sha256
    }

    pub(crate) fn declared_inputs_sha256(&self) -> &str {
        &self.declared_inputs_sha256
    }

    pub(crate) fn logical_bytes(&self) -> usize {
        json_bytes(&(
            self.core_version,
            self.domain_opset,
            &self.domain_registry_sha256,
            &self.main_content_sha256,
            &self.source_graph_sha256,
            &self.declared_inputs_sha256,
        ))
    }
}

pub(crate) fn json_bytes(value: &impl Serialize) -> usize {
    serde_json::to_vec(value)
        .expect("provenance JSON contains only closed values")
        .len()
}
