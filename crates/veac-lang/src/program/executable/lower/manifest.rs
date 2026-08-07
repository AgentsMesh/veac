use sha2::{Digest, Sha256};

use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use veac_ir::{ExecutableDigests, ExecutableManifest, TemporalProgramLibrary};

use super::error::ExecutableLowerError;

pub(super) fn executable(
    graph: &FrozenDomainGraph,
) -> Result<ExecutableManifest, ExecutableLowerError> {
    let identity = graph.typed_program_identity().ok_or_else(missing)?;
    if identity.domain_opset() != veac_ir::CURRENT_DOMAIN_OPSET_VERSION {
        return Err(missing());
    }
    Ok(ExecutableManifest::current(
        env!("CARGO_PKG_VERSION"),
        ExecutableDigests {
            domain_registry_sha256: identity.domain_registry_sha256().to_owned(),
            main_core_sha256: identity.main_content_sha256().to_owned(),
            source_graph_sha256: identity.source_graph_sha256().to_owned(),
            declared_inputs_sha256: identity.declared_inputs_sha256().to_owned(),
            compiler_sha256: compiler(identity.core_version(), identity.domain_opset()),
        },
    ))
}

pub(super) fn empty_temporal() -> TemporalProgramLibrary {
    TemporalProgramLibrary {
        opset_version: veac_ir::TEMPORAL_OPSET_VERSION,
        programs: Vec::new(),
        bindings: Vec::new(),
        provenance: Vec::new(),
    }
}

fn compiler(core_version: u16, domain_opset: u16) -> String {
    compiler_for_build(
        env!("VEAC_COMPILER_BUILD_SHA256"),
        core_version,
        domain_opset,
        veac_ir::TEMPORAL_OPSET_VERSION,
    )
}

pub(super) fn compiler_for_build(
    build_sha256: &str,
    core_version: u16,
    domain_opset: u16,
    temporal_opset: u16,
) -> String {
    let mut payload = Vec::new();
    payload.extend_from_slice(build_sha256.as_bytes());
    payload.extend_from_slice(&core_version.to_be_bytes());
    payload.extend_from_slice(&domain_opset.to_be_bytes());
    payload.extend_from_slice(&temporal_opset.to_be_bytes());
    digest(b"veac.compiler-manifest.v2\0", &payload)
}

fn digest(domain: &[u8], value: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(domain);
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
    format!("{:x}", digest.finalize())
}

fn missing() -> ExecutableLowerError {
    ExecutableLowerError::lower(
        "EXECUTABLE_LOWER_MANIFEST",
        "the executable graph has no compatible verified program identity",
    )
}
