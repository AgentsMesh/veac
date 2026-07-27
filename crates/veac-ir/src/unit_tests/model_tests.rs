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
