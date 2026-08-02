use crate::authoring::{format_document, lower_document, parse};

pub(super) const PROJECT: &str = r#"project color {
  settings {
    timebase 1/1000; canvas 1920px by 1080px;
    frame-rate 30/1fps; sample-rate 48000hz;
  }
  resource lut-3d cinematic { locator local { path "cinematic.cube"; } }
  entry sequence main;
  sequence main {
    layer visual picture {
      item hero {
        source generated solid { color #223344ff; }
        record { at 0s; duration 2s; }
        modifiers {
          color grade {
            input-space {
              primaries bt709; transfer bt709; matrix bt709; range limited;
            }
            working-space {
              primaries bt709; transfer linear; matrix rgb; range full;
            }
            output-space {
              primaries bt709; transfer bt709; matrix bt709; range limited;
            }
            basic {
              exposure 0stops; temperature 6500k; tint 0;
              highlights 0; shadows 0; fade 0%;
            }
            matrix {
              red 1 0 0; green 0 1 0; blue 0 0 1; offset 0 0 0;
            }
            hsl { range red; hue 10deg; saturation 0; lightness 0; }
            curves {
              interpolation natural;
              curve luma { point 0% 0%; point 100% 100%; }
            }
            wheels { lift 0 0 0; gamma 0 0 0; gain 0 0 0; }
            lut { resource cinematic; interpolation trilinear; }
          }
        }
      }
    }
  }
}"#;

#[test]
fn color_pipeline_parse_format_lower_and_validate() {
    let document = parse(PROJECT).unwrap();
    let formatted = format_document(&document);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    let envelope =
        lower_document(&document).unwrap_or_else(|error| panic!("{:#?}", error.as_slice()));
    let pipeline = envelope.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_ref()
        .unwrap()
        .color_pipeline
        .as_ref()
        .unwrap();
    assert_eq!(pipeline.stages.len(), 6);
    assert!(matches!(
        pipeline.stages[3],
        veac_ir::ColorStage::Curves { .. }
    ));
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn invalid_curve_channel_is_rejected() {
    let source = PROJECT.replace("curve luma", "curve alpha");
    let diagnostics = parse(&source).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_UNKNOWN_COLOR_CHANNEL"));
}

#[test]
fn unknown_color_stage_is_rejected() {
    let source = PROJECT.replace("basic {", "mystery {");
    let diagnostics = parse(&source).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_UNKNOWN_COLOR_STAGE"));
}
