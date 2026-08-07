use super::super::{
    language_spec, PluginBackendSpec, PluginDeterminismSpec, PluginParameterTypeSpec,
};

#[test]
fn current_plugin_inventory_exposes_the_complete_typed_descriptor() {
    let spec = language_spec();
    let [plugin] = spec.plugin_effects.as_slice() else {
        panic!("expected one closed plugin descriptor")
    };
    assert_eq!(plugin.schema, veac_ir::PLUGIN_EFFECT_DESCRIPTOR_SCHEMA);
    assert_eq!(plugin.schema_version, 1);
    assert_eq!(
        plugin.descriptor_constructor,
        "plugin_reference_monochrome_v1"
    );
    assert_eq!(plugin.application_constructor, "video_plugin_scalar_effect");
    assert_eq!(
        plugin.effect_type,
        veac_ir::REFERENCE_MONOCHROME_EFFECT_TYPE
    );
    assert_eq!(plugin.digest, veac_ir::REFERENCE_MONOCHROME_DIGEST);
    assert_eq!(plugin.determinism, PluginDeterminismSpec::Deterministic);
    assert_eq!(plugin.supported_backends, [PluginBackendSpec::Ffmpeg8]);
    assert_eq!(plugin.parameters.len(), 1);
    let parameter = &plugin.parameters[0];
    assert_eq!(parameter.name, "amount");
    assert_eq!(parameter.value_type, PluginParameterTypeSpec::Number);
    assert_eq!(
        (parameter.minimum.as_deref(), parameter.maximum.as_deref()),
        (Some("0"), Some("1"))
    );
    assert!(parameter.supports_curve);
    spec.validate().unwrap();
}

#[test]
fn plugin_inventory_validation_rejects_identity_schema_and_backend_drift() {
    let mut missing = language_spec();
    missing.plugin_effects.clear();
    assert_invalid(&missing, "complete closed registry");

    let mut effect_type = language_spec();
    effect_type.plugin_effects[0]
        .effect_type
        .push_str(".changed");
    assert_invalid(&effect_type, "canonical");

    let mut digest = language_spec();
    digest.plugin_effects[0].digest = "00".repeat(32);
    assert_invalid(&digest, "canonical");

    let mut constructor = language_spec();
    constructor.plugin_effects[0].descriptor_constructor = "unknown".into();
    assert_invalid(&constructor, "standard library");

    let mut bounds = language_spec();
    bounds.plugin_effects[0].parameters[0].minimum = Some("2".into());
    assert_invalid(&bounds, "parameter contract");

    let mut decimal = language_spec();
    decimal.plugin_effects[0].parameters[0].minimum = Some("0.0".into());
    assert_invalid(&decimal, "parameter contract");

    let mut non_finite = language_spec();
    non_finite.plugin_effects[0].parameters[0].minimum = Some("NaN".into());
    assert_invalid(&non_finite, "parameter contract");

    let mut backend = language_spec();
    backend.plugin_effects[0].supported_backends.clear();
    assert_invalid(&backend, "canonical");
}

#[test]
fn canonical_decimal_bounds_round_trip_without_json_float_drift() {
    let spec = language_spec();
    let encoded = serde_json_canonicalizer::to_string(&spec).unwrap();
    assert!(encoded.contains(r#""minimum":"0""#));
    assert!(encoded.contains(r#""maximum":"1""#));
    let decoded: super::super::LanguageSpec = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, spec);
    decoded.validate().unwrap();
}

#[test]
fn strict_json_rejects_unknown_plugin_descriptor_fields() {
    let mut value = serde_json::to_value(language_spec()).unwrap();
    value["plugin_effects"][0]["dynamic_parameters"] = true.into();
    assert!(serde_json::from_value::<super::super::LanguageSpec>(value).is_err());
}

fn assert_invalid(value: &super::super::LanguageSpec, message: &str) {
    let error = value.validate().unwrap_err().to_string();
    assert!(error.contains(message), "unexpected error: {error}");
}
