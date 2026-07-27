use crate::authoring::{lower_document, parse, Diagnostics};

use super::project;

fn source(interpolation: &str) -> String {
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
            key b {{ at 1s; value 0.9; interpolation linear; }}
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
