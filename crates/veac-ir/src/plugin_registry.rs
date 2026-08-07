use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    built_in_effect, built_in_effects, EffectKind, EffectParameter, EffectSpec, ParameterSpec,
    ParameterType,
};

pub const PLUGIN_EFFECT_DESCRIPTOR_SCHEMA: &str =
    "https://veac.dev/schemas/plugin-effect-descriptor";
pub const REFERENCE_MONOCHROME_DIGEST: &str =
    "e208978cbc18912f43b3501f09ea1236c083f6c871567b824655000e7a02e4d9";
pub const REFERENCE_MONOCHROME_EFFECT_TYPE: &str = "video.plugin.veac.reference_monochrome.v1.e208978cbc18912f43b3501f09ea1236c083f6c871567b824655000e7a02e4d9";

const DIGEST_DOMAIN: &str = "veac.plugin-effect-descriptor.v1";
const REFERENCE_NAMESPACE: &str = "video.plugin.veac.reference_monochrome.v1";
const REFERENCE_IMPLEMENTATION: &str = "veac.ffmpeg8.monochrome-mix.v1";
const REFERENCE_DESCRIPTOR_CONSTRUCTOR: &str = "plugin_reference_monochrome_v1";
const REFERENCE_APPLICATION_CONSTRUCTOR: &str = "video_plugin_scalar_effect";
const REFERENCE_PARAMETERS: &[ParameterSpec] = &[ParameterSpec {
    parameter: EffectParameter::Amount,
    value_type: ParameterType::Number,
    minimum: Some(0.0),
    maximum: Some(1.0),
    supports_curve: true,
}];
const REFERENCE_BACKENDS: &[PluginBackend] = &[PluginBackend::Ffmpeg8];
const PLUGINS: &[PluginEffectDescriptor] = &[PluginEffectDescriptor {
    schema: PLUGIN_EFFECT_DESCRIPTOR_SCHEMA,
    schema_version: 1,
    namespace: REFERENCE_NAMESPACE,
    implementation: REFERENCE_IMPLEMENTATION,
    descriptor_constructor: REFERENCE_DESCRIPTOR_CONSTRUCTOR,
    application_constructor: REFERENCE_APPLICATION_CONSTRUCTOR,
    determinism: PluginDeterminism::Deterministic,
    supported_backends: REFERENCE_BACKENDS,
    digest: REFERENCE_MONOCHROME_DIGEST,
    effect_type: REFERENCE_MONOCHROME_EFFECT_TYPE,
    effect_kind: EffectKind::VideoPluginReferenceMonochromeV1,
    parameters: REFERENCE_PARAMETERS,
}];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginDeterminism {
    Deterministic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PluginBackend {
    #[serde(rename = "ffmpeg-8")]
    Ffmpeg8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PluginEffectDescriptor {
    pub schema: &'static str,
    pub schema_version: u16,
    pub namespace: &'static str,
    pub implementation: &'static str,
    pub descriptor_constructor: &'static str,
    pub application_constructor: &'static str,
    pub determinism: PluginDeterminism,
    pub supported_backends: &'static [PluginBackend],
    pub digest: &'static str,
    pub effect_type: &'static str,
    pub effect_kind: EffectKind,
    pub parameters: &'static [ParameterSpec],
}

impl PluginEffectDescriptor {
    pub const fn effect_spec(self) -> EffectSpec {
        EffectSpec {
            kind: self.effect_kind,
            effect_type: self.effect_type,
            parameters: self.parameters,
        }
    }

    pub fn supports(self, backend: PluginBackend) -> bool {
        self.supported_backends.contains(&backend)
    }

    pub fn digest_matches(self) -> bool {
        plugin_effect_descriptor_digest(self).is_ok_and(|digest| digest == self.digest)
    }
}

#[derive(Serialize)]
struct DigestContent<'a> {
    domain: &'static str,
    schema: &'a str,
    schema_version: u16,
    namespace: &'a str,
    implementation: &'a str,
    descriptor_constructor: &'a str,
    application_constructor: &'a str,
    parameters: &'a [ParameterSpec],
    determinism: PluginDeterminism,
    supported_backends: &'a [PluginBackend],
}

pub fn plugin_effect_descriptor_digest(
    descriptor: PluginEffectDescriptor,
) -> Result<String, serde_json::Error> {
    let content = DigestContent {
        domain: DIGEST_DOMAIN,
        schema: descriptor.schema,
        schema_version: descriptor.schema_version,
        namespace: descriptor.namespace,
        implementation: descriptor.implementation,
        descriptor_constructor: descriptor.descriptor_constructor,
        application_constructor: descriptor.application_constructor,
        parameters: descriptor.parameters,
        determinism: descriptor.determinism,
        supported_backends: descriptor.supported_backends,
    };
    let bytes = serde_json_canonicalizer::to_vec(&content)?;
    Ok(hex(Sha256::digest(bytes)))
}

pub fn plugin_effect(kind: EffectKind) -> Option<PluginEffectDescriptor> {
    PLUGINS
        .iter()
        .copied()
        .find(|descriptor| descriptor.effect_kind == kind)
}

pub fn plugin_effect_type(effect_type: &str) -> Option<PluginEffectDescriptor> {
    PLUGINS
        .iter()
        .copied()
        .find(|descriptor| descriptor.effect_type == effect_type)
}

pub fn plugin_effects() -> &'static [PluginEffectDescriptor] {
    PLUGINS
}

pub fn registered_effect(kind: EffectKind) -> Option<EffectSpec> {
    built_in_effect(kind).or_else(|| plugin_effect(kind).map(PluginEffectDescriptor::effect_spec))
}

pub fn registered_effect_type(effect_type: &str) -> Option<EffectSpec> {
    crate::EffectKind::from_type_name(effect_type).and_then(registered_effect)
}

pub fn registered_effects() -> impl Iterator<Item = EffectSpec> + Clone {
    built_in_effects().iter().copied().chain(
        PLUGINS
            .iter()
            .copied()
            .map(PluginEffectDescriptor::effect_spec),
    )
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.as_ref().len() * 2);
    for byte in bytes.as_ref() {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
