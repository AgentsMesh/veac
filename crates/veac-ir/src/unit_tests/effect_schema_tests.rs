use std::collections::BTreeSet;

use crate::*;

#[test]
fn effect_schema_exactly_matches_the_closed_executable_union() {
    let schema = project_json_schema().unwrap();
    let variants = schema["$defs"]["Effect"]["oneOf"]
        .as_array()
        .expect("tagged effect union");
    let actual: BTreeSet<String> = variants
        .iter()
        .map(|variant| {
            variant["properties"]["type"]["const"]
                .as_str()
                .expect("effect type tag")
                .to_owned()
        })
        .collect();
    let expected: BTreeSet<_> = EffectKind::ALL
        .into_iter()
        .map(|kind| serde_json::to_value(kind).unwrap())
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(actual.len(), variants.len(), "duplicate schema variant");
    assert_eq!(actual, expected);
}

#[test]
fn property_bag_effects_fail_deserialization() {
    let legacy = serde_json::json!({
        "effect_type": "video.blur",
        "parameters": {"radius": {"type": "number", "value": 2.0}}
    });
    assert!(serde_json::from_value::<Effect>(legacy).is_err());
    assert!(serde_json::from_value::<Effect>(serde_json::json!({
        "type": "video_blur",
        "radius": {"type": "constant", "value": 2.0},
        "unknown": true
    }))
    .is_err());
}

#[test]
fn closed_effect_accessors_cover_every_parameter_storage_class() {
    let mut chroma = Effect::neutral(EffectKind::VideoChromaKey);
    assert!(matches!(
        chroma.parameter(EffectParameter::Color),
        Some(EffectParameterRef::Color(_))
    ));
    let blue = Color {
        red: 0,
        green: 0,
        blue: 255,
        alpha: 255,
    };
    assert_eq!(
        chroma.set_parameter(EffectParameter::Color, EffectParameterValue::Color(blue)),
        Some(true)
    );
    assert_eq!(chroma.color(EffectParameter::Color), Some(&blue));
    assert_eq!(
        chroma.set_parameter(EffectParameter::Color, EffectParameterValue::Color(blue)),
        Some(false)
    );

    let mut luma = Effect::neutral(EffectKind::VideoLumaKey);
    assert_eq!(
        luma.set_parameter(EffectParameter::Invert, EffectParameterValue::Boolean(true)),
        Some(true)
    );
    assert_eq!(luma.boolean(EffectParameter::Invert), Some(true));

    let mut normalize = Effect::neutral(EffectKind::AudioNormalize);
    assert_eq!(
        normalize.set_parameter(
            EffectParameter::TargetLufs,
            EffectParameterValue::Number(-18.0)
        ),
        Some(true)
    );
    assert_eq!(normalize.number(EffectParameter::TargetLufs), Some(-18.0));
    assert_eq!(
        normalize.set_parameter(
            EffectParameter::Radius,
            EffectParameterValue::Curve(Animatable::constant(2.0))
        ),
        None
    );
}

#[test]
fn effect_catalog_and_plugin_digest_types_are_total() {
    for kind in EffectKind::ALL {
        let effect = Effect::neutral(kind);
        assert_eq!(effect.kind(), kind);
        assert_eq!(EffectKind::from_type_name(kind.type_name()), Some(kind));
        assert_eq!(
            effect.domain(),
            if kind == EffectKind::AudioNormalize {
                EffectDomain::Audio
            } else {
                EffectDomain::Video
            }
        );
    }
    let digest = PluginEffectDigest::new(REFERENCE_MONOCHROME_DIGEST).unwrap();
    assert_eq!(digest.as_str(), REFERENCE_MONOCHROME_DIGEST);
    assert_eq!(digest.to_string(), REFERENCE_MONOCHROME_DIGEST);
    let error = PluginEffectDigest::new("invalid").unwrap_err();
    assert!(error
        .to_string()
        .contains("invalid plugin effect descriptor digest"));
}
