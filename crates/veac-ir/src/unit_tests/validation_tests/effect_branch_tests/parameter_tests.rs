use super::*;
use crate::test_support::time;

#[test]
fn registered_and_unregistered_effects_keep_distinct_parameter_contracts() {
    let mut registered = sample_project();
    let effect = effect_mut(&mut registered);
    effect.parameters.clear();
    effect.parameters.insert(
        "brightness".to_owned(),
        ParameterValue::Number { value: -1.0 },
    );
    validate(&registered).unwrap();

    let mut unregistered = sample_project();
    let effect = effect_mut(&mut unregistered);
    effect.effect_type = "vendor.private".to_owned();
    effect.parameters.clear();
    effect.parameters.insert(
        "arbitrary".to_owned(),
        ParameterValue::Boolean { value: true },
    );
    let codes = effect_codes(&unregistered);
    assert_code(&codes, "UNKNOWN_EFFECT");
    assert!(!codes.iter().any(|code| code == "UNKNOWN_EFFECT_PARAMETER"));
    assert!(!codes.iter().any(|code| code == "EFFECT_PARAMETER_TYPE"));
}

#[test]
fn generic_number_parameter_rejects_non_finite_values() {
    let mut project = sample_project();
    let effect = effect_mut(&mut project);
    effect.effect_type = "vendor.private".to_owned();
    effect.parameters.clear();
    effect.parameters.insert(
        "invalid".to_owned(),
        ParameterValue::Number { value: f64::NAN },
    );
    assert_effect_code(&project, "EFFECT_PARAMETER");
}

#[test]
fn number_curve_parameters_validate_curve_shape_after_schema_matching() {
    let mut empty = sample_project();
    let effect = effect_mut(&mut empty);
    effect.parameters.clear();
    effect.parameters.insert(
        "brightness".to_owned(),
        ParameterValue::NumberCurve {
            value: Animatable::Keyframes { keyframes: vec![] },
        },
    );
    assert_effect_code(&empty, "EMPTY_KEYFRAMES");

    let mut invalid = sample_project();
    let effect = effect_mut(&mut invalid);
    effect.parameters.clear();
    effect.parameters.insert(
        "brightness".to_owned(),
        ParameterValue::NumberCurve {
            value: Animatable::Keyframes {
                keyframes: vec![Keyframe {
                    id: KeyframeId::new("kf_effect_invalid").unwrap(),
                    time: time(601),
                    value: f64::NAN,
                    interpolation: Interpolation::Hold,
                }],
            },
        },
    );
    let codes = effect_codes(&invalid);
    assert_code(&codes, "EFFECT_PARAMETER_TYPE");
    assert_code(&codes, "KEYFRAME_ORDER");
    assert_code(&codes, "ANIMATION_VALUE");
}

#[test]
fn every_registered_video_curve_parameter_accepts_keyframes() {
    let mut covered = 0;
    for effect in built_in_effects()
        .iter()
        .filter(|effect| effect.effect_type.starts_with("video."))
    {
        for parameter in effect
            .parameters
            .iter()
            .filter(|parameter| parameter.supports_curve)
        {
            assert_eq!(parameter.value_type, ParameterType::Number);
            let mut project = sample_project();
            let instance = effect_mut(&mut project);
            instance.effect_type = effect.effect_type.to_owned();
            instance.parameters.clear();
            instance.parameters.insert(
                parameter.name.to_owned(),
                curve_parameter(
                    parameter.minimum.unwrap(),
                    parameter.maximum.unwrap(),
                    covered,
                ),
            );
            validate(&project).unwrap_or_else(|errors| {
                panic!(
                    "{}.{}, diagnostics={:?}",
                    effect.effect_type,
                    parameter.name,
                    errors.into_diagnostics()
                )
            });
            covered += 1;
        }
    }
    assert_eq!(covered, 14);
}

#[test]
fn audio_normalize_requires_a_static_target_and_allows_partial_range() {
    let mut project = sample_project();
    let effect = effect_mut(&mut project);
    effect.effect_type = "audio.normalize".to_owned();
    effect.enable_range = Some(crate::test_support::range(100, 300));
    effect.parameters.clear();
    effect.parameters.insert(
        "target_lufs".to_owned(),
        ParameterValue::Number { value: -18.0 },
    );
    validate(&project).unwrap();

    effect_mut(&mut project)
        .parameters
        .insert("target_lufs".to_owned(), curve_parameter(-18.0, -16.0, 99));
    let json = serde_json::to_string(&project).unwrap();
    let CanonicalError::Validation(errors) = decode_canonical_json(&json).unwrap_err() else {
        panic!("normalize curve must fail canonical semantic validation")
    };
    let codes: Vec<_> = errors
        .into_diagnostics()
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect();
    assert_code(&codes, "EFFECT_PARAMETER_TYPE");
    assert!(!codes.iter().any(|code| code == "EFFECT_RANGE"));
}

fn curve_parameter(start: f64, end: f64, index: usize) -> ParameterValue {
    ParameterValue::NumberCurve {
        value: Animatable::Keyframes {
            keyframes: vec![
                Keyframe {
                    id: KeyframeId::new(format!("kf_effect_{index}_start")).unwrap(),
                    time: time(0),
                    value: start,
                    interpolation: Interpolation::Linear,
                },
                Keyframe {
                    id: KeyframeId::new(format!("kf_effect_{index}_end")).unwrap(),
                    time: time(300),
                    value: end,
                    interpolation: Interpolation::Linear,
                },
            ],
        },
    }
}
