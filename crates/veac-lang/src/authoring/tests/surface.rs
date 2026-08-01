use crate::authoring::{format_document, lower_document, parse};

use super::project;

const SURFACE: &str = r#"surface raised {
  corner-radius 36px;
  shadow {
    color #000000ff;
    opacity 38%;
    blur 28px;
    offset { x 0px; y 16px; }
  }
}"#;

#[test]
fn surface_round_trips_and_lowers_to_typed_card_style() {
    let document = parse(&visual_project(SURFACE)).unwrap();
    let formatted = format_document(&document);
    for line in [
        "surface raised {",
        "corner-radius 36px;",
        "opacity 38%;",
        "offset {",
    ] {
        assert!(
            formatted.contains(line),
            "missing `{line}` in:\n{formatted}"
        );
    }
    let reparsed = parse(&formatted).unwrap();
    assert_eq!(format_document(&reparsed), formatted);

    let envelope = lower_document(&reparsed).unwrap();
    let card = envelope.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_ref()
        .unwrap()
        .card
        .as_ref()
        .unwrap();
    assert_eq!(card.corner_radius_pixels, 36.0);
    let shadow = card.shadow.as_ref().unwrap();
    assert_eq!(shadow.blur_pixels, 28.0);
    assert_eq!(shadow.opacity, 0.38);
    assert_eq!(shadow.offset.x, 0.0);
    assert_eq!(shadow.offset.y, 16.0);
    assert_eq!(
        shadow.color,
        veac_ir::Color {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 255,
        }
    );
}

#[test]
fn surface_rejects_non_pixel_geometry_units() {
    for invalid in [
        SURFACE.replace("36px", "36%"),
        SURFACE.replace("28px", "28deg"),
        SURFACE.replace("16px", "16%"),
    ] {
        let document = parse(&visual_project(&invalid)).unwrap();
        let diagnostics = lower_document(&document).unwrap_err();
        assert!(diagnostics
            .as_slice()
            .iter()
            .any(|value| value.code == "AUTHORING_LOWER_UNIT"));
    }
}

#[test]
fn surface_requires_complete_card_and_shadow_fields() {
    for invalid in [
        SURFACE.replace("  corner-radius 36px;\n", ""),
        SURFACE.replace("    opacity 38%;\n", ""),
        SURFACE.replace("offset { x 0px; y 16px; }", "offset { x 0px; }"),
    ] {
        let diagnostics = parse(&visual_project(&invalid)).unwrap_err();
        assert!(diagnostics
            .as_slice()
            .iter()
            .any(|value| value.code == "AUTHORING_REQUIRED_FIELD"));
    }
}

#[test]
fn item_accepts_only_one_surface_modifier() {
    let source = visual_project(&format!("{SURFACE}\n{SURFACE}"));
    let document = parse(&source).unwrap();
    let diagnostics = lower_document(&document).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_LOWER_DUPLICATE_MODIFIER"));
}

#[test]
fn audio_layers_reject_surface_as_a_visual_modifier() {
    let document = parse(&layer_project(
        "audio",
        "source generated silence;",
        SURFACE,
    ))
    .unwrap();
    let diagnostics = lower_document(&document).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_LOWER_TRACK_COMPONENT"));
}

fn visual_project(surface: &str) -> String {
    layer_project(
        "visual",
        "source generated solid { color #f7f7f2ff; }",
        surface,
    )
}

fn layer_project(kind: &str, source: &str, surface: &str) -> String {
    project(&format!(
        r#"sequence main {{
  layer {kind} base {{
    item panel {{
      {source}
      record {{ at 0s; duration 4s; }}
      modifiers {{ {surface} }}
    }}
  }}
}}"#
    ))
}
