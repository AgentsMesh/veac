use super::*;
use crate::test_support::time;

#[test]
fn number_curve_parameters_validate_shape_and_range() {
    let mut empty = sample_project();
    set_curve(&mut empty, Animatable::Keyframes { keyframes: vec![] });
    assert_effect_code(&empty, "EMPTY_KEYFRAMES");

    let mut invalid = sample_project();
    set_curve(
        &mut invalid,
        Animatable::Keyframes {
            keyframes: vec![Keyframe {
                id: KeyframeId::new("kf_effect_invalid").unwrap(),
                time: time(601),
                value: f64::NAN,
                interpolation: Interpolation::Hold,
            }],
        },
    );
    let codes = effect_codes(&invalid);
    assert_code(&codes, "EFFECT_PARAMETER_RANGE");
    assert_code(&codes, "KEYFRAME_ORDER");
    assert_code(&codes, "ANIMATION_VALUE");
}

#[test]
fn every_built_in_video_curve_parameter_accepts_keyframes() {
    let mut covered = 0;
    for specification in built_in_effects()
        .iter()
        .filter(|effect| effect.kind != EffectKind::AudioNormalize)
    {
        for parameter in specification
            .parameters
            .iter()
            .filter(|parameter| parameter.supports_curve)
        {
            let mut project = sample_project();
            let mut effect = Effect::neutral(specification.kind);
            let curve = bounded_curve(
                parameter.minimum.unwrap(),
                parameter.maximum.unwrap(),
                covered,
            );
            assert_eq!(
                effect.set_parameter(parameter.parameter, EffectParameterValue::Curve(curve)),
                Some(true)
            );
            effect_mut(&mut project).effect = effect;
            validate(&project).unwrap_or_else(|errors| {
                panic!(
                    "{}.{} diagnostics={:?}",
                    specification.effect_type,
                    parameter.parameter.name(),
                    errors.into_diagnostics()
                )
            });
            covered += 1;
        }
    }
    assert_eq!(covered, 16);
}

#[test]
fn directional_blur_rejects_backend_unsafe_bounds() {
    let mut project = sample_project();
    effect_mut(&mut project).effect = Effect::VideoDirectionalBlur {
        angle_degrees: Animatable::constant(360.0),
        radius: Animatable::constant(100.0),
    };
    validate(&project).unwrap();

    let Effect::VideoDirectionalBlur { angle_degrees, .. } = &mut effect_mut(&mut project).effect
    else {
        unreachable!()
    };
    *angle_degrees = Animatable::constant(360.1);
    assert_effect_code(&project, "EFFECT_PARAMETER_RANGE");

    effect_mut(&mut project).effect = Effect::VideoDirectionalBlur {
        angle_degrees: Animatable::constant(90.0),
        radius: Animatable::constant(100.1),
    };
    assert_effect_code(&project, "EFFECT_PARAMETER_RANGE");
}

#[test]
fn audio_normalize_requires_a_finite_bounded_static_target() {
    let mut project = sample_project();
    let effect = effect_mut(&mut project);
    effect.effect = Effect::AudioNormalize { target_lufs: -18.0 };
    effect.enable_range = Some(crate::test_support::range(100, 300));
    validate(&project).unwrap();

    effect_mut(&mut project).effect = Effect::AudioNormalize {
        target_lufs: f64::NAN,
    };
    assert_effect_code(&project, "EFFECT_PARAMETER_RANGE");
    effect_mut(&mut project).effect = Effect::AudioNormalize { target_lufs: -4.0 };
    assert_effect_code(&project, "EFFECT_PARAMETER_RANGE");
}

#[test]
fn spring_extrema_must_stay_inside_effect_bounds() {
    let mut project = sample_project();
    effect_mut(&mut project).effect = Effect::VideoLumaKey {
        threshold: Animatable::Keyframes {
            keyframes: vec![
                spring_key("kf_threshold_start", 0, 0.1),
                spring_key("kf_threshold_end", 600, 0.9),
            ],
        },
        tolerance: Animatable::constant(0.1),
        softness: Animatable::constant(0.1),
        invert: false,
    };
    assert_effect_code(&project, "EFFECT_PARAMETER_RANGE");
}

fn set_curve(project: &mut ProjectEnvelope, value: Animatable<f64>) {
    effect_mut(project)
        .effect
        .set_parameter(
            EffectParameter::Brightness,
            EffectParameterValue::Curve(value),
        )
        .unwrap();
}

fn bounded_curve(start: f64, end: f64, index: usize) -> Animatable<f64> {
    Animatable::Keyframes {
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
    }
}

fn spring_key(id: &str, at: i64, value: f64) -> Keyframe<f64> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value,
        interpolation: Interpolation::Spring {
            frequency: 1.5,
            decay: 6.0,
            initial_velocity: 0.0,
        },
    }
}
