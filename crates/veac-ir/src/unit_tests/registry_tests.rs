use crate::{test_support::time, *};

#[test]
fn built_in_registry_is_closed_and_discoverable() {
    assert_eq!(built_in_effects().len(), EffectKind::BUILT_IN.len());
    for kind in EffectKind::BUILT_IN {
        let specification = built_in_effect(kind).unwrap();
        assert_eq!(specification.kind, kind);
        assert_eq!(specification.effect_type, kind.type_name());
        assert!(!specification.parameters.is_empty());
    }
    assert_eq!(
        built_in_effect(EffectKind::VideoPluginReferenceMonochromeV1),
        None
    );
    assert!(transition_parameters_valid(&TransitionKind::Wipe {
        direction: CardinalDirection::Left,
        angle_degrees: 15.0,
        softness: 0.2,
    }));
    assert!(!transition_parameters_valid(&TransitionKind::Pixelize {
        amount: 0.0,
    }));
}

#[test]
fn parameter_schema_checks_typed_values_and_ranges() {
    let blur = built_in_effect(EffectKind::VideoBlur).unwrap().parameters[0];
    assert!(parameter_matches(
        blur,
        EffectParameterRef::Curve(&Animatable::constant(0.5))
    ));
    assert!(!parameter_matches(blur, EffectParameterRef::Boolean(true)));
    assert!(!parameter_matches(
        blur,
        EffectParameterRef::Curve(&Animatable::constant(101.0))
    ));
    let normalize = built_in_effect(EffectKind::AudioNormalize)
        .unwrap()
        .parameters[0];
    assert!(!normalize.supports_curve);
    assert!(parameter_matches(
        normalize,
        EffectParameterRef::Number(-16.0)
    ));

    let directional = built_in_effect(EffectKind::VideoDirectionalBlur).unwrap();
    assert_eq!(directional.effect_type, "video.directional_blur");
    assert_eq!(directional.parameters.len(), 2);
    let angle = directional.parameters[0];
    let radius = directional.parameters[1];
    assert_eq!(angle.parameter, EffectParameter::AngleDegrees);
    assert_eq!((angle.minimum, angle.maximum), (Some(0.0), Some(360.0)));
    assert_eq!(radius.parameter, EffectParameter::Radius);
    assert_eq!((radius.minimum, radius.maximum), (Some(0.0), Some(100.0)));
    assert!(!parameter_matches(
        angle,
        EffectParameterRef::Curve(&Animatable::constant(-0.1))
    ));
    assert!(!parameter_matches(
        radius,
        EffectParameterRef::Curve(&Animatable::constant(100.1))
    ));
}

#[test]
fn spring_and_cubic_extrema_stay_inside_parameter_bounds() {
    let threshold = built_in_effect(EffectKind::VideoLumaKey)
        .unwrap()
        .parameters[0];
    let curve = |interpolation| Animatable::Keyframes {
        keyframes: vec![
            Keyframe {
                id: KeyframeId::new("kf_threshold_start").unwrap(),
                time: time(0),
                value: 0.1,
                interpolation,
            },
            Keyframe {
                id: KeyframeId::new("kf_threshold_end").unwrap(),
                time: time(600),
                value: 0.9,
                interpolation: Interpolation::Linear,
            },
        ],
    };
    assert!(!parameter_matches(
        threshold,
        EffectParameterRef::Curve(&curve(Interpolation::Spring {
            frequency: 1.5,
            decay: 6.0,
            initial_velocity: 0.0,
        }))
    ));
    assert!(parameter_matches(
        threshold,
        EffectParameterRef::Curve(&curve(Interpolation::CubicBezier {
            x1: 0.2,
            y1: 0.0,
            x2: 0.8,
            y2: 1.0,
        }))
    ));
}
