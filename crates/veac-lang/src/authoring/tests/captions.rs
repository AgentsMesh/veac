use crate::authoring::{format_document, lower_document, parse};

const PROJECT: &str = r#"project captions {
  settings {
    timebase 1/1000;
    canvas 1280px by 720px;
    frame-rate 30fps;
    sample-rate 48000hz;
  }
  entry sequence main;

  sequence main {
    layer caption subtitles {
      item opening {
        source caption {
          content "Welcome to VEAC";
          speaker "Narrator";
          style {
            font family "Inter";
            size 48px;
            fill #ffffffff;
            background { color #000000aa; padding 12px; }
          }
          layout {
            box-width 1100px;
            box-height 180px;
            wrap word;
            overflow clip;
            horizontal-align center;
            vertical-align bottom;
            writing-mode horizontal-tb;
            orientation upright;
          }
        }
        record { at 0s; duration 2s; }
      }
    }
  }

  delivery transcript {
    sequence main;
    artifact caption-sidecar transcript {
      target file "captions.vtt";
      source caption-tracks { track subtitles; }
      encode web-vtt;
    }
  }
}"#;

#[test]
fn caption_source_round_trips_into_sidecar_ready_ir() {
    let document = parse(PROJECT).unwrap();
    let formatted = format_document(&document);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    let envelope = lower_document(&document).unwrap();
    let source = &envelope.project.sequences[0].tracks[0].clips[0].source;
    let veac_ir::ClipSource::Caption {
        text,
        speaker,
        style,
    } = source
    else {
        panic!("caption source expected")
    };
    assert_eq!(text, "Welcome to VEAC");
    assert_eq!(speaker.as_deref(), Some("Narrator"));
    assert_eq!(style.size_pixels, 48.0);
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn caption_source_is_confined_to_caption_layers() {
    let invalid = PROJECT.replace("layer caption subtitles", "layer visual subtitles");
    let diagnostics = parse(&invalid).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_SOURCE_LAYER"));
}

#[test]
fn caption_speaker_is_a_bounded_single_line_value() {
    let invalid = PROJECT.replace("speaker \"Narrator\"", "speaker \" \"");
    let diagnostics = parse(&invalid).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_CAPTION_SPEAKER"));
}
