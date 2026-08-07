use sha2::{Digest, Sha256};
use veac_artifact::ContentDigest;
use veac_ir::PluginBackend;

pub const CODEGEN_SOURCE_SHA256: &str = env!("VEAC_CODEGEN_SOURCE_SHA256");
pub const CODEGEN_BUILD_SHA256: &str = env!("VEAC_CODEGEN_BUILD_SHA256");
pub const FFMPEG_PLUGIN_ADAPTER_OPSET_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderImplementationIdentity {
    pub source_sha256: &'static str,
    pub build_sha256: &'static str,
    pub render_contract_version: u32,
    pub core_version: u16,
    pub domain_opset_version: u16,
    pub temporal_opset_version: u16,
    pub plugin_adapter_opset_version: u16,
    pub digest: ContentDigest,
}

struct Inputs<'a> {
    source: &'a str,
    build: &'a str,
    render: u32,
    core: u16,
    domain: u16,
    temporal: u16,
    adapter: u16,
    plugins: Vec<PluginInput<'a>>,
}

#[derive(Clone, Copy)]
struct PluginInput<'a> {
    effect_type: &'a str,
    descriptor_digest: &'a str,
    implementation: &'a str,
}

pub fn render_implementation_identity() -> RenderImplementationIdentity {
    let input = current();
    RenderImplementationIdentity {
        source_sha256: CODEGEN_SOURCE_SHA256,
        build_sha256: CODEGEN_BUILD_SHA256,
        render_contract_version: crate::RENDER_IMPLEMENTATION_CONTRACT_VERSION,
        core_version: veac_ir::CURRENT_CORE_VERSION,
        domain_opset_version: veac_ir::CURRENT_DOMAIN_OPSET_VERSION,
        temporal_opset_version: veac_ir::TEMPORAL_OPSET_VERSION,
        plugin_adapter_opset_version: FFMPEG_PLUGIN_ADAPTER_OPSET_VERSION,
        digest: calculate(&input),
    }
}

fn current() -> Inputs<'static> {
    let plugins = veac_ir::plugin_effects()
        .iter()
        .filter(|value| value.supports(PluginBackend::Ffmpeg8))
        .map(|value| PluginInput {
            effect_type: value.effect_type,
            descriptor_digest: value.digest,
            implementation: value.implementation,
        })
        .collect();
    Inputs {
        source: CODEGEN_SOURCE_SHA256,
        build: CODEGEN_BUILD_SHA256,
        render: crate::RENDER_IMPLEMENTATION_CONTRACT_VERSION,
        core: veac_ir::CURRENT_CORE_VERSION,
        domain: veac_ir::CURRENT_DOMAIN_OPSET_VERSION,
        temporal: veac_ir::TEMPORAL_OPSET_VERSION,
        adapter: FFMPEG_PLUGIN_ADAPTER_OPSET_VERSION,
        plugins,
    }
}

fn calculate(input: &Inputs<'_>) -> ContentDigest {
    let mut digest = Sha256::new();
    digest.update(b"veac.render-implementation.v1\0");
    field(&mut digest, b"source", input.source.as_bytes());
    field(&mut digest, b"build", input.build.as_bytes());
    field(&mut digest, b"render", &input.render.to_be_bytes());
    field(&mut digest, b"core", &input.core.to_be_bytes());
    field(&mut digest, b"domain", &input.domain.to_be_bytes());
    field(&mut digest, b"temporal", &input.temporal.to_be_bytes());
    field(&mut digest, b"adapter", &input.adapter.to_be_bytes());
    field(
        &mut digest,
        b"plugin-count",
        &(input.plugins.len() as u64).to_be_bytes(),
    );
    for plugin in &input.plugins {
        field(&mut digest, b"plugin-type", plugin.effect_type.as_bytes());
        field(
            &mut digest,
            b"plugin-digest",
            plugin.descriptor_digest.as_bytes(),
        );
        field(
            &mut digest,
            b"plugin-implementation",
            plugin.implementation.as_bytes(),
        );
    }
    ContentDigest::sha256(digest.finalize())
}

fn field(digest: &mut Sha256, name: &[u8], value: &[u8]) {
    digest.update((name.len() as u64).to_be_bytes());
    digest.update(name);
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

#[cfg(test)]
#[path = "identity/tests.rs"]
mod tests;
