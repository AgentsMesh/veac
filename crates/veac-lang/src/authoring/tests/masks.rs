use crate::authoring::{format_document, lower_document, parse};

const PROJECT: &str = r#"project masks {
  settings {
    timebase 1/1000;
    canvas 1920px by 1080px;
    frame-rate 30/1fps;
    sample-rate 48000hz;
  }
  entry sequence main;
  sequence main {
    layer visual graphics {
      item hero {
        source generated solid { color #336699ff; }
        record { at 0s; duration 2s; }
        modifiers {
          mask reveal {
            shape circle;
            position { x 50%; y 50%; }
            scale { x 80%; y 80%; }
            rotation 0deg;
            feather 24px;
            expansion 2px;
            invert false;
          }
          mask crop {
            shape path {
              point { x 10%; y 10%; }
              point { x 90%; y 10%; }
              point { x 50%; y 90%; }
            }
          }
          mask rounded {
            shape rounded-rectangle { radius 20%; }
          }
          mask polygon {
            shape polygon {
              point { x 10%; y 10%; }
              point { x 90%; y 10%; }
              point { x 50%; y 90%; }
            }
          }
        }
      }
    }
  }
}"#;

#[test]
fn masks_parse_format_lower_and_validate() {
    let document = parse(PROJECT).unwrap();
    let formatted = format_document(&document);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    let envelope = lower_document(&document).unwrap();
    let masks = &envelope.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_ref()
        .unwrap()
        .masks;
    assert_eq!(masks.len(), 4);
    assert!(matches!(masks[0].shape, veac_ir::MaskShape::Circle));
    assert!(matches!(masks[1].shape, veac_ir::MaskShape::Path { .. }));
    assert!(matches!(
        masks[2].shape,
        veac_ir::MaskShape::RoundedRectangle { .. }
    ));
    assert!(matches!(masks[3].shape, veac_ir::MaskShape::Polygon { .. }));
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn unknown_mask_shape_is_rejected() {
    let source = PROJECT.replace("shape circle;", "shape triangle;");
    let diagnostics = parse(&source).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_UNKNOWN_MASK_SHAPE"));
}

#[test]
fn short_mask_path_is_rejected() {
    let source = PROJECT.replace("point { x 50%; y 90%; }", "");
    let diagnostics = parse(&source).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_MASK_PATH_POINTS"));
}
