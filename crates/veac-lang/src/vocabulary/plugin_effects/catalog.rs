use super::*;
use veac_ir::{
    ParameterSpec, ParameterType, PluginBackend, PluginDeterminism, PluginEffectDescriptor,
};

pub(super) fn current() -> Vec<PluginEffectSpec> {
    let mut values = veac_ir::plugin_effects()
        .iter()
        .copied()
        .map(plugin)
        .collect::<Vec<_>>();
    values.sort_by(|left, right| left.effect_type.cmp(&right.effect_type));
    values
}

fn plugin(value: PluginEffectDescriptor) -> PluginEffectSpec {
    PluginEffectSpec {
        descriptor_constructor: value.descriptor_constructor.to_owned(),
        application_constructor: value.application_constructor.to_owned(),
        schema: value.schema.to_owned(),
        schema_version: value.schema_version,
        namespace: value.namespace.to_owned(),
        implementation: value.implementation.to_owned(),
        effect_type: value.effect_type.to_owned(),
        digest: value.digest.to_owned(),
        parameters: value.parameters.iter().map(parameter).collect(),
        determinism: determinism(value.determinism),
        supported_backends: value
            .supported_backends
            .iter()
            .copied()
            .map(backend)
            .collect(),
    }
}

fn parameter(value: &ParameterSpec) -> PluginParameterSpec {
    PluginParameterSpec {
        name: value.parameter.name().to_owned(),
        value_type: match value.value_type {
            ParameterType::Number => PluginParameterTypeSpec::Number,
            ParameterType::Boolean => PluginParameterTypeSpec::Boolean,
            ParameterType::Color => PluginParameterTypeSpec::Color,
        },
        minimum: value.minimum.map(decimal),
        maximum: value.maximum.map(decimal),
        supports_curve: value.supports_curve,
    }
}

fn decimal(value: f64) -> String {
    serde_json_canonicalizer::to_string(&value)
        .expect("verified plugin bounds are finite canonical numbers")
}

fn determinism(value: PluginDeterminism) -> PluginDeterminismSpec {
    match value {
        PluginDeterminism::Deterministic => PluginDeterminismSpec::Deterministic,
    }
}

fn backend(value: PluginBackend) -> PluginBackendSpec {
    match value {
        PluginBackend::Ffmpeg8 => PluginBackendSpec::Ffmpeg8,
    }
}
