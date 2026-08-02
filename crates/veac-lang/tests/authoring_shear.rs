use veac_ir::Vec2;
use veac_lang::{format_document, lower_document, parse};

const SOURCE: &str = r#"project shear-primitive {
  settings {
    timebase 1/600;
    canvas 320px by 180px;
    frame-rate 30fps;
    sample-rate 48000hz;
  }
  entry sequence main;
  sequence main {
    layer visual graphics {
      item badge {
        source generated solid { color #ff0000ff; }
        record { at 0s; duration 1s; }
        modifiers {
          transform lean {
            shear { x 0.35; y -0.2; }
          }
        }
      }
    }
  }
}"#;

#[test]
fn static_unitless_shear_round_trips_and_lowers_to_canonical_ir() {
    let parsed = parse(SOURCE).unwrap();
    let formatted = format_document(&parsed);
    assert!(
        formatted.contains("shear {\n              x 0.35;\n              y -0.2;\n            }")
    );
    let project = lower_document(&parse(&formatted).unwrap()).unwrap();
    let transform = &project.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_ref()
        .unwrap()
        .transform;
    assert_eq!(transform.shear, Vec2 { x: 0.35, y: -0.2 });
}

#[test]
fn shear_rejects_units_animation_and_backend_unsafe_factors() {
    let unit = SOURCE.replace("x 0.35", "x 0.35deg");
    let error = lower_document(&parse(&unit).unwrap()).unwrap_err();
    assert!(error.to_string().contains("AUTHORING_LOWER_UNIT"));

    let animated = SOURCE.replace(
        "shear { x 0.35; y -0.2; }",
        "shear curve {\n              key a { at 0s; value { x 0; y 0; } }\n            }",
    );
    assert!(parse(&animated)
        .unwrap_err()
        .to_string()
        .contains("AUTHORING_STATIC_VALUE"));

    let unsafe_factor = SOURCE.replace("x 0.35", "x 2.01");
    let error = lower_document(&parse(&unsafe_factor).unwrap()).unwrap_err();
    assert!(error.to_string().contains("SHEAR"));
    assert!(error.to_string().contains("[-2, 2]"));
}
