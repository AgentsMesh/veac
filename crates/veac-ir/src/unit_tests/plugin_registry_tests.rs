use crate::*;

#[test]
fn plugin_descriptor_pins_every_execution_identity() {
    let [descriptor] = plugin_effects() else {
        panic!("expected one reference plugin descriptor")
    };
    assert_eq!(descriptor.schema, PLUGIN_EFFECT_DESCRIPTOR_SCHEMA);
    assert_eq!(descriptor.schema_version, 1);
    assert_eq!(
        descriptor.descriptor_constructor,
        "plugin_reference_monochrome_v1"
    );
    assert_eq!(
        descriptor.application_constructor,
        "video_plugin_scalar_effect"
    );
    assert_eq!(descriptor.effect_type, REFERENCE_MONOCHROME_EFFECT_TYPE);
    assert_eq!(
        descriptor.effect_kind,
        EffectKind::VideoPluginReferenceMonochromeV1
    );
    assert_eq!(descriptor.digest, REFERENCE_MONOCHROME_DIGEST);
    assert!(descriptor.effect_type.ends_with(descriptor.digest));
    assert_eq!(descriptor.determinism, PluginDeterminism::Deterministic);
    assert_eq!(descriptor.supported_backends, [PluginBackend::Ffmpeg8]);
    assert!(descriptor.supports(PluginBackend::Ffmpeg8));
    let computed = plugin_effect_descriptor_digest(*descriptor).unwrap();
    assert_eq!(computed, descriptor.digest);
    assert!(descriptor.digest_matches());
}

#[test]
fn plugin_parameter_schema_is_typed_and_registered_separately() {
    let descriptor = plugin_effect(EffectKind::VideoPluginReferenceMonochromeV1).unwrap();
    assert_eq!(
        descriptor.parameters,
        [ParameterSpec {
            parameter: EffectParameter::Amount,
            value_type: ParameterType::Number,
            minimum: Some(0.0),
            maximum: Some(1.0),
            supports_curve: true,
        }]
    );
    assert_eq!(
        registered_effect(EffectKind::VideoPluginReferenceMonochromeV1),
        Some(descriptor.effect_spec())
    );
    assert_eq!(
        built_in_effect(EffectKind::VideoPluginReferenceMonochromeV1),
        None
    );
    assert_eq!(built_in_effects().len(), 11);
    assert_eq!(registered_effects().count(), 12);
}

#[test]
fn descriptor_digest_detects_identity_or_schema_drift() {
    let descriptor = plugin_effects()[0];
    let changed = PluginEffectDescriptor {
        implementation: "veac.ffmpeg8.monochrome-mix.v2",
        ..descriptor
    };
    assert_ne!(
        plugin_effect_descriptor_digest(changed).unwrap(),
        descriptor.digest
    );
    assert!(!changed.digest_matches());
    assert_eq!(plugin_effect_type("video.plugin.unknown"), None);
    assert_eq!(registered_effect_type("video.plugin.unknown"), None);
}
