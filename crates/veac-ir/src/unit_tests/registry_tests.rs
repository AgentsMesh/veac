use crate::{test_support::time, *};

#[test]
fn built_in_registry_is_closed_and_discoverable() {
    assert_eq!(built_in_effects().len(), 10);
    for effect in [
        "video.color_adjust",
        "video.blur",
        "video.sharpen",
        "video.vignette",
        "video.grain",
        "video.chroma_key",
        "video.luma_key",
        "video.chroma_spill",
        "video.stabilize",
        "audio.normalize",
    ] {
        let specification = built_in_effect(effect).unwrap();
        assert_eq!(specification.effect_type, effect);
        assert!(!specification.parameters.is_empty());
    }
    assert_eq!(built_in_effect("video.typo"), None);
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
fn parameter_schema_checks_types_curves_and_ranges() {
    let color = ParameterValue::Color {
        value: Color {
            red: 1,
            green: 2,
            blue: 3,
            alpha: 4,
        },
    };
    let text = ParameterValue::Text {
        value: "soft".to_owned(),
    };
    let boolean = ParameterValue::Boolean { value: true };
    let number = ParameterValue::Number { value: 0.5 };
    let curve = ParameterValue::NumberCurve {
        value: Animatable::Keyframes {
            keyframes: vec![Keyframe {
                id: KeyframeId::new("kf_registry").unwrap(),
                time: time(0),
                value: 0.5,
                interpolation: Interpolation::Hold,
            }],
        },
    };
    let specs = [
        ParameterSpec {
            name: "number",
            value_type: ParameterType::Number,
            minimum: Some(0.0),
            maximum: Some(1.0),
            supports_curve: true,
        },
        ParameterSpec {
            name: "bool",
            value_type: ParameterType::Boolean,
            minimum: None,
            maximum: None,
            supports_curve: false,
        },
        ParameterSpec {
            name: "color",
            value_type: ParameterType::Color,
            minimum: None,
            maximum: None,
            supports_curve: false,
        },
        ParameterSpec {
            name: "text",
            value_type: ParameterType::Text,
            minimum: None,
            maximum: None,
            supports_curve: false,
        },
    ];
    assert!(parameter_matches(specs[0], &number));
    assert!(parameter_matches(specs[0], &curve));
    assert!(parameter_matches(specs[1], &boolean));
    assert!(parameter_matches(specs[2], &color));
    assert!(parameter_matches(specs[3], &text));
    assert!(!parameter_matches(specs[0], &text));
    assert!(!parameter_matches(
        specs[0],
        &ParameterValue::Number { value: 2.0 },
    ));
    let normalize = built_in_effect("audio.normalize").unwrap().parameters[0];
    assert!(!normalize.supports_curve);
    assert!(!parameter_matches(normalize, &curve));
}
