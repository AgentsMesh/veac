use crate::{test_support::time, *};

#[test]
fn number_parameters_cover_optional_bounds_constants_and_curve_values() {
    let unbounded = spec(None, None);
    let minimum = spec(Some(0.0), None);
    let maximum = spec(None, Some(1.0));
    for (specification, value, expected) in [
        (unbounded, -99.0, true),
        (unbounded, f64::NAN, false),
        (minimum, 0.0, true),
        (minimum, -0.1, false),
        (maximum, 1.0, true),
        (maximum, 1.1, false),
    ] {
        assert_eq!(
            parameter_matches(specification, &ParameterValue::Number { value }),
            expected
        );
    }

    assert!(parameter_matches(
        minimum,
        &ParameterValue::NumberCurve {
            value: Animatable::constant(0.5),
        }
    ));
    assert!(parameter_matches(
        minimum,
        &ParameterValue::NumberCurve {
            value: Animatable::Keyframes { keyframes: vec![] },
        }
    ));
    assert!(!parameter_matches(
        maximum,
        &ParameterValue::NumberCurve {
            value: Animatable::Keyframes {
                keyframes: vec![Keyframe {
                    id: KeyframeId::new("kf_outside").unwrap(),
                    time: time(0),
                    value: 2.0,
                    interpolation: Interpolation::Linear,
                }],
            },
        }
    ));
}

#[test]
fn parameter_kinds_match_only_their_typed_value_variants() {
    let values = [
        (
            ParameterType::Boolean,
            ParameterValue::Boolean { value: false },
        ),
        (
            ParameterType::Color,
            ParameterValue::Color {
                value: Color {
                    red: 1,
                    green: 2,
                    blue: 3,
                    alpha: 4,
                },
            },
        ),
    ];
    for (kind, value) in values {
        let specification = ParameterSpec {
            name: "typed",
            value_type: kind,
            minimum: None,
            maximum: None,
            supports_curve: false,
        };
        assert!(parameter_matches(specification, &value));
        assert!(!parameter_matches(
            specification,
            &ParameterValue::Number { value: 1.0 }
        ));
    }
}

#[test]
fn registry_consumes_every_declared_parameter_kind() {
    let mut number = false;
    let mut boolean = false;
    let mut color = false;
    for parameter in built_in_effects()
        .iter()
        .flat_map(|effect| effect.parameters)
    {
        match parameter.value_type {
            ParameterType::Number => number = true,
            ParameterType::Boolean => boolean = true,
            ParameterType::Color => color = true,
        }
    }
    assert!((number && boolean && color));
}

#[test]
fn registry_names_and_parameter_schemas_are_closed() {
    for effect in built_in_effects() {
        assert_eq!(built_in_effect(effect.effect_type), Some(*effect));
        for parameter in effect.parameters {
            assert!(!parameter.name.is_empty());
            if parameter.value_type == ParameterType::Number {
                assert!(parameter.minimum.is_some() && parameter.maximum.is_some());
                assert!(parameter.minimum <= parameter.maximum);
            } else {
                assert!(parameter.minimum.is_none() && parameter.maximum.is_none());
            }
        }
    }
    for unknown in ["", "VIDEO.BLUR", " video.blur", "video.blur "] {
        assert_eq!(built_in_effect(unknown), None);
    }
}

fn spec(minimum: Option<f64>, maximum: Option<f64>) -> ParameterSpec {
    ParameterSpec {
        name: "number",
        value_type: ParameterType::Number,
        minimum,
        maximum,
        supports_curve: true,
    }
}
