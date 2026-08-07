use crate::*;

#[test]
fn numeric_bounds_and_non_finite_values_fail_closed() {
    let specification = ParameterSpec {
        parameter: EffectParameter::Amount,
        value_type: ParameterType::Number,
        minimum: Some(0.0),
        maximum: Some(1.0),
        supports_curve: true,
    };
    for (value, expected) in [(0.0, true), (1.0, true), (-0.1, false), (1.1, false)] {
        assert_eq!(
            parameter_matches(
                specification,
                EffectParameterRef::Curve(&Animatable::constant(value))
            ),
            expected
        );
    }
    assert!(!parameter_matches(
        specification,
        EffectParameterRef::Curve(&Animatable::constant(f64::NAN))
    ));
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
        assert_eq!(
            EffectParameter::from_name(parameter.parameter.name()),
            Some(parameter.parameter)
        );
    }
    assert!(number && boolean && color);
}

#[test]
fn registry_names_and_parameter_schemas_are_closed() {
    for effect in built_in_effects() {
        assert_eq!(built_in_effect(effect.kind), Some(*effect));
        assert_eq!(built_in_effect_type(effect.effect_type), Some(*effect));
        for parameter in effect.parameters {
            if parameter.value_type == ParameterType::Number {
                assert!(parameter.minimum.is_some() && parameter.maximum.is_some());
                assert!(parameter.minimum <= parameter.maximum);
            } else {
                assert!(parameter.minimum.is_none() && parameter.maximum.is_none());
            }
        }
    }
    for unknown in ["", "VIDEO.BLUR", " video.blur", "video.blur "] {
        assert_eq!(built_in_effect_type(unknown), None);
    }
}
