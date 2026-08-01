use crate::authoring::{format_document, lower_document, parse, Diagnostics};

use super::project;

fn source(interpolation: &str) -> String {
    source_ending_at(interpolation, "0.8")
}

fn source_ending_at(interpolation: &str, end: &str) -> String {
    project(&format!(
        r#"sequence main {{
  layer visual picture {{
    item blank {{
      source generated transparent;
      record {{ at 0s; duration 2s; }}
      modifiers {{
        effect keyed {{
          type video.luma_key;
          parameter invert true;
          parameter threshold curve {{
            key a {{ at 0s; value 0.1; interpolation {interpolation} }}
            key b {{ at 1s; value {end}; interpolation linear; }}
          }}
        }}
      }}
    }}
  }}
}}"#
    ))
}

fn assert_code(error: &Diagnostics, code: &str) {
    assert!(
        error.as_slice().iter().any(|value| value.code == code),
        "missing {code}: {error}"
    );
}

#[test]
fn cubic_bezier_interpolation_preserves_all_four_control_points() {
    let document = parse(&source("cubic-bezier { x1 0.1; y1 0.2; x2 0.8; y2 0.9; }")).unwrap();
    let envelope = lower_document(&document).unwrap_or_else(|error| panic!("{error}"));
    let parameter = envelope.project.sequences[0].tracks[0].clips[0].effects[0]
        .parameters
        .get("threshold")
        .unwrap();
    let veac_ir::ParameterValue::NumberCurve {
        value: veac_ir::Animatable::Keyframes { keyframes },
    } = parameter
    else {
        panic!("number curve expected")
    };
    assert_eq!(
        keyframes[0].interpolation,
        veac_ir::Interpolation::CubicBezier {
            x1: 0.1,
            y1: 0.2,
            x2: 0.8,
            y2: 0.9,
        }
    );
}

#[test]
fn spring_interpolation_preserves_physics_parameters() {
    let document = parse(&source(
        "spring { frequency 1.5; decay 6; initial-velocity -0.25; }",
    ))
    .unwrap();
    let formatted = format_document(&document);
    assert!(formatted.contains("interpolation spring"));
    assert!(formatted.contains("initial-velocity -0.25;"));
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    let envelope = lower_document(&document).unwrap_or_else(|error| panic!("{error}"));
    let parameter = envelope.project.sequences[0].tracks[0].clips[0].effects[0]
        .parameters
        .get("threshold")
        .unwrap();
    let veac_ir::ParameterValue::NumberCurve {
        value: veac_ir::Animatable::Keyframes { keyframes },
    } = parameter
    else {
        panic!("number curve expected")
    };
    assert_eq!(
        keyframes[0].interpolation,
        veac_ir::Interpolation::Spring {
            frequency: 1.5,
            decay: 6.0,
            initial_velocity: -0.25,
        }
    );
}

#[test]
fn lowering_rejects_spring_extrema_outside_effect_parameter_bounds() {
    let document = parse(&source_ending_at(
        "spring { frequency 1.5; decay 6; initial-velocity 0; }",
        "0.9",
    ))
    .unwrap();
    let error = lower_document(&document).unwrap_err();
    assert!(error.as_slice().iter().any(|diagnostic| {
        diagnostic.code == "AUTHORING_LOWER_IR_VALIDATION"
            && diagnostic.message.contains("EFFECT_PARAMETER_TYPE")
    }));
}

#[test]
fn interpolation_rejects_non_closed_unknown_and_leaf_body_forms() {
    for (interpolation, code) in [
        ("linear extra;", "AUTHORING_INTERPOLATION"),
        ("mystery;", "AUTHORING_INTERPOLATION"),
        ("linear { x1 0; }", "AUTHORING_INTERPOLATION"),
    ] {
        assert_code(&parse(&source(interpolation)).unwrap_err(), code);
    }
}

#[test]
fn cubic_bezier_requires_every_field_and_reports_unknown_fields() {
    let missing = parse(&source("cubic-bezier { x1 0.1; y1 0.2; x2 0.8; }")).unwrap_err();
    assert_code(&missing, "AUTHORING_REQUIRED_FIELD");
    assert!(missing.as_slice()[0].message.contains("y2"));

    let unknown = parse(&source(
        "cubic-bezier { x1 0.1; y1 0.2; x2 0.8; y2 0.9; bogus 1; }",
    ))
    .unwrap_err();
    assert_code(&unknown, "AUTHORING_UNKNOWN_FIELD");
    assert!(unknown.as_slice()[0].message.contains("bogus"));
}

#[test]
fn spring_requires_every_field_and_reports_unknown_fields() {
    let missing = parse(&source("spring { frequency 1.5; decay 6; }")).unwrap_err();
    assert_code(&missing, "AUTHORING_REQUIRED_FIELD");
    assert!(missing.as_slice()[0].message.contains("initial-velocity"));

    let unknown = parse(&source(
        "spring { frequency 1.5; decay 6; initial-velocity 0; mass 1; }",
    ))
    .unwrap_err();
    assert_code(&unknown, "AUTHORING_UNKNOWN_FIELD");
    assert!(unknown.as_slice()[0].message.contains("mass"));
}
