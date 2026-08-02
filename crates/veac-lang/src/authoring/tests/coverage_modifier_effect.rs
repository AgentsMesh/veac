use crate::authoring::{format_document, lower_document, parse};

use super::project;

#[test]
fn boolean_color_and_curve_effect_parameters_lower_to_typed_values() {
    let source = project(
        r#"sequence main {
  layer visual picture {
    item blank {
      source generated transparent;
      record { at 0s; duration 2s; }
      modifiers {
effect a-rgb {
  type video.chroma_key;
  enabled false;
  record { at 0s; duration 1s; }
  parameter blend 0.1;
  parameter color #112233;
  parameter similarity 0.2;
}
effect b-rgba {
  type video.chroma_key;
  parameter blend 0.2;
  parameter color #44556677;
  parameter similarity 0.3;
}
effect c-luma {
  type video.luma_key;
  parameter invert true;
  parameter threshold curve {
    key start { at 0s; value 0.1; interpolation hold; }
    key end { at 1s; value 0.9; interpolation linear; }
  }
}
      }
    }
  }
}"#,
    );
    let document = parse(&source).unwrap_or_else(|error| panic!("{error}"));
    let formatted = format_document(&document);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);

    let envelope = lower_document(&document).unwrap_or_else(|error| panic!("{error}"));
    let effects = &envelope.project.sequences[0].tracks[0].clips[0].effects;
    assert_eq!(effects.len(), 3);
    assert!(!effects[0].enabled);
    assert!(effects[0].enable_range.is_some());

    let colors = effects[0..2]
        .iter()
        .map(|effect| effect.parameters.get("color").unwrap())
        .map(|parameter| match parameter {
            veac_ir::ParameterValue::Color { value } => *value,
            _ => panic!("color parameter expected"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        colors,
        [
            veac_ir::Color {
                red: 0x11,
                green: 0x22,
                blue: 0x33,
                alpha: 0xff
            },
            veac_ir::Color {
                red: 0x44,
                green: 0x55,
                blue: 0x66,
                alpha: 0x77
            },
        ]
    );

    let invert = effects[2].parameters.get("invert").unwrap();
    assert_eq!(*invert, veac_ir::ParameterValue::Boolean { value: true });
    let threshold = effects[2].parameters.get("threshold").unwrap();
    let veac_ir::ParameterValue::NumberCurve {
        value: veac_ir::Animatable::Keyframes { keyframes },
    } = threshold
    else {
        panic!("threshold curve expected")
    };
    assert_eq!(keyframes.len(), 2);
    assert_eq!(keyframes[0].interpolation, veac_ir::Interpolation::Hold);
    assert_eq!(keyframes[1].interpolation, veac_ir::Interpolation::Linear);
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn invalid_effect_parameter_types_and_ranges_have_specific_diagnostics() {
    let bad_type = project(
        r#"sequence main {
  layer visual picture {
    item blank {
      source generated transparent;
      record { at 0s; duration 1s; }
      modifiers { effect keyed { type video.mystery; } }
    }
  }
}"#,
    );
    let error = parse(&bad_type).unwrap_err();
    assert!(error
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_EFFECT_TYPE"));
    for parameter in ["parameter invert 1;", "parameter threshold true;"] {
        let source = project(&format!(
            r#"sequence main {{
  layer visual picture {{
    item blank {{
      source generated transparent;
      record {{ at 0s; duration 1s; }}
      modifiers {{ effect keyed {{ type video.luma_key; {parameter} }} }}
    }}
  }}
}}"#
        ));
        let error = parse(&source).unwrap_err();
        assert!(
            error
                .as_slice()
                .iter()
                .any(|value| value.code == "AUTHORING_EFFECT_PARAMETER_TYPE"),
            "missing type diagnostic: {error}"
        );
    }
    let source = project(
        r#"sequence main {
  layer visual picture {
    item blank {
      source generated transparent;
      record { at 0s; duration 1s; }
      modifiers { effect keyed { type video.luma_key; parameter threshold 1.5; } }
    }
  }
}"#,
    );
    let document = parse(&source).unwrap();
    let error = lower_document(&document).unwrap_err();
    assert!(
        error.as_slice().iter().any(|value| {
            value.code == "AUTHORING_LOWER_IR_VALIDATION"
                && value.message.contains("EFFECT_PARAMETER_TYPE")
                && value.message.contains("outside the canonical IR contract")
        }),
        "unexpected range diagnostic: {error}"
    );
}
