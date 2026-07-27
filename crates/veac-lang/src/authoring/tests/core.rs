use super::project;
use crate::authoring::{parse, LayerKind, ResourceKind, ResourceLocator};

#[test]
fn parses_project_settings_resources_and_timeline() {
    let source = project(
        r#"
  // resources remain typed at declaration and reference sites
  resource video host-video {
    locator local { path "host.mov"; }
    streams { video auto; audio disabled; }
  }
  resource audio host-voice {
    locator remote {
      uri "https://cdn.test/voice.wav";
      identity sha256 "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    }
    streams { video disabled; audio auto; }
  }
  resource image poster { locator local { path "poster.png"; } }
  resource font captions { locator local { path "captions.ttf"; } }
  resource lut-3d cinematic { locator local { path "cinematic.cube"; } }
  sequence main {
    layer video primary {
      order 0;
      item host {
        source media resource host-video;
        record { at 0s; duration 10s; }
      }
    }
    layer audio dialogue {
      item voice {
        source media resource host-voice;
        record { at 0s; duration 10s; }
      }
    }
    layer visual graphics {}
    layer caption subtitles {}
  }
"#,
    );
    let parsed = parse(&source).unwrap();
    assert_eq!(parsed.project.id.value, "demo");
    assert_eq!(parsed.project.entry.kind.value, "sequence");
    assert_eq!(parsed.project.entry.id.value, "main");
    assert_eq!(
        parsed.project.settings.canvas.as_ref().unwrap().0.raw,
        "1920px"
    );
    assert_eq!(
        parsed.project.settings.frame_rate.as_ref().unwrap().raw,
        "30000/1001fps"
    );
    assert_eq!(parsed.project.resources.len(), 5);
    assert_eq!(parsed.project.resources[0].kind, ResourceKind::Video);
    assert!(matches!(
        parsed.project.resources[1].locator,
        ResourceLocator::Remote { .. }
    ));
    let layers = &parsed.project.sequences[0].layers;
    assert_eq!(layers[0].kind, LayerKind::Video);
    assert_eq!(layers[2].kind, LayerKind::Visual);
    assert!(layers[0].items[0].mapping.is_none());
}

#[test]
fn accepts_integer_and_rational_frame_rates() {
    let integer = project("sequence main {}").replace("30000/1001fps", "30fps");
    assert!(parse(&integer).is_ok());
    assert!(parse(&project("sequence main {}")).is_ok());
}

#[test]
fn accepts_only_the_five_canonical_output_types() {
    let outputs = r#"
  output video video-out {
    sequence main; file-name "preview.mp4"; encoding {}
  }
  output image-sequence frames {
    sequence main; file-name "frame-%d.png"; encoding {}
  }
  output caption-sidecar captions-out {
    sequence main; file-name "captions.vtt";
    encoding { tracks { track captions; } }
  }
  output audio-stem master-stem {
    sequence main; file-name "master.wav"; encoding {}
  }
  output scope waveform {
    sequence main; file-name "waveform.png"; encoding {}
  }
"#;
    let body = format!("sequence main {{ layer caption captions {{}} }}\n{outputs}");
    let parsed = parse(&project(&body)).unwrap();
    assert_eq!(parsed.project.outputs.len(), 5);
}
