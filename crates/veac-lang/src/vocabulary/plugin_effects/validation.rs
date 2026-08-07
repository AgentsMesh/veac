use std::collections::BTreeSet;

use super::*;

pub(super) fn validate(
    values: &[PluginEffectSpec],
    library: &StandardLibrarySpec,
) -> Result<(), VocabularyValidationError> {
    if values.len() != veac_ir::plugin_effects().len() || values.is_empty() {
        return Err(error(
            "plugin effects must publish the complete closed registry",
        ));
    }
    if values
        .windows(2)
        .any(|pair| pair[0].effect_type >= pair[1].effect_type)
    {
        return Err(error(
            "plugin effects must be unique and ordered by effect type",
        ));
    }
    let mut constructors = BTreeSet::new();
    let mut digests = BTreeSet::new();
    for value in values {
        if !constructors.insert(value.descriptor_constructor.as_str())
            || !digests.insert(value.digest.as_str())
        {
            return Err(error("plugin descriptor identities must be unique"));
        }
        validate_plugin(value, library)?;
    }
    if values != super::catalog::current() {
        return Err(error(
            "plugin effects do not match the current closed registry",
        ));
    }
    Ok(())
}

fn validate_plugin(
    value: &PluginEffectSpec,
    library: &StandardLibrarySpec,
) -> Result<(), VocabularyValidationError> {
    if value.schema != veac_ir::PLUGIN_EFFECT_DESCRIPTOR_SCHEMA
        || value.schema_version == 0
        || !lower_hex(&value.digest)
        || !value.effect_type.ends_with(&value.digest)
        || value.supported_backends.is_empty()
    {
        return Err(error("plugin descriptor identity is not canonical"));
    }
    for constructor in [
        value.descriptor_constructor.as_str(),
        value.application_constructor.as_str(),
    ] {
        if !library
            .free_functions
            .iter()
            .any(|function| function.name == constructor)
        {
            return Err(error("plugin constructor is not in the standard library"));
        }
    }
    let mut parameters = BTreeSet::new();
    for parameter in &value.parameters {
        if !parameters.insert(parameter.name.as_str()) || !parameter_valid(parameter) {
            return Err(error("plugin parameter contract is not canonical"));
        }
    }
    Ok(())
}

fn parameter_valid(value: &PluginParameterSpec) -> bool {
    let (Ok(minimum), Ok(maximum)) = (
        value.minimum.as_deref().map(decimal).transpose(),
        value.maximum.as_deref().map(decimal).transpose(),
    ) else {
        return false;
    };
    let ordered = minimum
        .zip(maximum)
        .is_none_or(|(minimum, maximum)| minimum <= maximum);
    let typed = value.value_type == PluginParameterTypeSpec::Number
        || (value.minimum.is_none() && value.maximum.is_none() && !value.supports_curve);
    ordered && typed
}

fn decimal(value: &str) -> Result<f64, ()> {
    let parsed = value.parse::<f64>().map_err(|_| ())?;
    if !parsed.is_finite() || serde_json_canonicalizer::to_string(&parsed).map_err(|_| ())? != value
    {
        return Err(());
    }
    Ok(parsed)
}

fn lower_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn error(message: impl Into<String>) -> VocabularyValidationError {
    VocabularyValidationError::new(message)
}
