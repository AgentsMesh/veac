use super::project;
use crate::authoring::{format_document, parse, ModifierDecl, ParameterDecl, PlacementDecl};

pub(super) const MODIFIERS: &str = r#"
  sequence main {
    layer visual graphics {
      item badge {
        source generated transparent;
        record { at 0s; duration 2s; }
        modifiers {
          layout badge-layout {
            placement anchor {
              at bottom-right;
              inset { x 24px; y 32px; }
            }
            frame { width 320px; height 180px; fit contain; }
          }
          transform badge-motion {
            position curve {
              key start {
                at 0s;
                value { x 0px; y 24px; }
                interpolation ease-out;
              }
              key settle {
                at 400ms;
                value { x 0px; y 0px; }
                interpolation linear;
              }
            }
            scale { x 1; y 1; }
            rotation 0deg;
            anchor { x 0.5; y 0.5; }
            crop { x 0; y 0; width 1; height 1; }
            flip-horizontal false;
            flip-vertical false;
          }
          composite badge-composite {
            opacity 0.85;
            z-index 10;
            blend screen;
          }
          effect badge-blur {
            type video.blur;
            enabled true;
            record { at 0s; duration 300ms; }
            parameter radius curve {
              key soft { at 0s; value 8; interpolation linear; }
              key sharp { at 300ms; value 0; interpolation ease-in; }
            }
          }
        }
      }
    }
  }
"#;

#[test]
fn modifiers_are_closed_typed_and_canonically_formatted() {
    let parsed = parse(&project(MODIFIERS)).expect("typed modifiers parse");
    let modifiers = &parsed.project.sequences[0].layers[0].items[0].modifiers;
    let ModifierDecl::Layout(layout) = &modifiers[0] else {
        panic!("layout expected")
    };
    assert!(matches!(
        layout.placement,
        Some(PlacementDecl::Anchor { .. })
    ));
    let ModifierDecl::Transform(transform) = &modifiers[1] else {
        panic!("transform expected")
    };
    assert!(matches!(
        transform.position,
        Some(ParameterDecl::Curve { .. })
    ));
    let formatted = format_document(&parsed);
    let reparsed = parse(&formatted).expect("canonical modifiers parse");
    assert_eq!(format_document(&reparsed), formatted);
}

#[test]
fn modifier_schemas_reject_wrong_kind_field_and_parameter_type() {
    let wrong_kind = MODIFIERS.replace("layout badge-layout", "filter badge-layout");
    let diagnostics = parse(&project(&wrong_kind)).unwrap_err().to_string();
    assert!(diagnostics.contains("AUTHORING_MODIFIER_KIND"));

    let wrong_field = MODIFIERS.replace("rotation 0deg;", "volume 1;");
    let diagnostics = parse(&project(&wrong_field)).unwrap_err().to_string();
    assert!(diagnostics.contains("AUTHORING_UNKNOWN_FIELD"));

    let wrong_parameter =
        MODIFIERS.replace("parameter radius curve {", "parameter radius #ffffff;");
    let diagnostics = parse(&project(&wrong_parameter)).unwrap_err().to_string();
    assert!(diagnostics.contains("AUTHORING_EFFECT_PARAMETER_TYPE"));
}
