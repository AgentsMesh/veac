use crate::{test_support::sample_project, *};

#[test]
fn project_envelope_materializes_schema_contract() {
    let envelope = sample_project();
    assert_eq!(envelope.schema, SCHEMA_ID);
    assert_eq!(envelope.schema_version, CURRENT_SCHEMA_VERSION);
    assert_eq!(envelope.min_reader_version, MIN_READER_VERSION);
}

#[test]
fn animatable_exposes_keyframes_without_hiding_constants() {
    let constant = Animatable::constant(4.0);
    assert_eq!(constant.keyframes(), None);
    let animated = Animatable::Keyframes {
        keyframes: vec![Keyframe {
            id: KeyframeId::new("kf_one").unwrap(),
            time: RationalTime::new(0, 600).unwrap(),
            value: 1.0,
            interpolation: Interpolation::Hold,
        }],
    };
    assert_eq!(animated.keyframes().unwrap().len(), 1);
}

#[test]
fn clip_source_reports_only_typed_font_material_references() {
    let style = TextStyle {
        font: FontRef::Material {
            material_id: MaterialId::new("med_font").unwrap(),
        },
        size_pixels: 20.0,
        color: Color {
            red: 1,
            green: 2,
            blue: 3,
            alpha: 4,
        },
        background: None,
        outline: None,
        shadow: None,
        ..TextStyle::default()
    };
    let text = ClipSource::Text {
        text: "a".to_owned(),
        style: style.clone(),
    };
    let caption = ClipSource::Caption {
        text: "b".to_owned(),
        speaker: None,
        cue: Box::default(),
        style,
    };
    assert_eq!(text.font_material().unwrap().as_str(), "med_font");
    assert_eq!(caption.font_material().unwrap().as_str(), "med_font");

    let family = ClipSource::Text {
        text: "c".to_owned(),
        style: TextStyle {
            font: FontRef::Family {
                family: "Inter".to_owned(),
            },
            size_pixels: 20.0,
            color: Color {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 255,
            },
            background: None,
            outline: None,
            shadow: None,
            ..TextStyle::default()
        },
    };
    assert_eq!(family.font_material(), None);
    assert_eq!(
        ClipSource::Generated {
            generator: Generator::Silence
        }
        .font_material(),
        None
    );
}

#[test]
fn spring_interpolation_has_an_explicit_canonical_shape() {
    let spring = Interpolation::Spring {
        frequency: 1.5,
        decay: 6.0,
        initial_velocity: 0.0,
    };
    let json = serde_json::to_value(&spring).unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "type": "spring",
            "frequency": 1.5,
            "decay": 6.0,
            "initial_velocity": 0.0
        })
    );
    assert_eq!(
        serde_json::from_value::<Interpolation>(json).unwrap(),
        spring
    );
}

#[test]
fn cubic_interpolation_reports_intermediate_overshoot_extrema() {
    let easing = Interpolation::CubicBezier {
        x1: 0.2,
        y1: -0.4,
        x2: 0.8,
        y2: 1.4,
    };
    let extrema = easing.intermediate_extrema().unwrap();
    assert_eq!(extrema.len(), 2);
    assert!((extrema[0] + 0.058_406_766).abs() < 1e-6);
    assert!((extrema[1] - 1.058_406_766).abs() < 1e-6);
    assert_eq!(Interpolation::Linear.intermediate_extrema(), Some(vec![]));
    assert_eq!(
        Interpolation::CubicBezier {
            x1: 0.2,
            y1: f64::MAX,
            x2: 0.8,
            y2: -f64::MAX,
        }
        .intermediate_extrema(),
        None
    );
}
