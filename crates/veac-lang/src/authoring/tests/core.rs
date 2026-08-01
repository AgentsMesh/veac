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
fn accepts_all_nine_canonical_artifact_recipes() {
    let delivery = r#"
  delivery preview {
    sequence main;
    raster { canvas 1920px by 1080px; frame-rate 30fps; captions burn-in; }
    artifact video video-out {
      target file "preview.mp4"; mux mp4 { video h264 {} audio none; }
    }
    artifact image-sequence frames {
      target pattern "frame-%d.png"; numbering from 1; encode png;
    }
    artifact caption-sidecar captions-out {
      target file "captions.vtt"; source caption-tracks { track captions; } encode web-vtt;
    }
    artifact audio-stem master-stem {
      target file "master.wav"; source master;
      encode wav { sample-format pcm-s24le; sample-rate 48khz; channel-layout stereo; }
    }
    artifact scope waveform {
      target file "waveform.png"; analyze waveform; frame containing 0s;
      canvas 640px by 360px; encode png;
    }
    artifact audio-file podcast {
      target file "podcast.mp3"; source master;
      encode mp3 { bitrate 192kbps; sample-rate 48khz; channel-layout stereo; }
    }
    artifact animated-image loop {
      target file "loop.gif"; encode gif { playback forever; dither sierra2; }
    }
    artifact still-image cover {
      target file "cover.png"; frame containing 0s; encode png;
    }
    artifact adaptive-package stream {
      target package "stream";
      package hls {
        segment-duration 2s; audio none;
        rendition hd {
          canvas 1280px by 720px;
          encode h264 { rate-control capped { target 3mbps; max 3210kbps; buffer 6mbit; } }
        }
      }
    }
  }
"#;
    let body = format!("sequence main {{ layer caption captions {{}} }}\n{delivery}");
    let parsed = parse(&project(&body)).unwrap();
    assert_eq!(parsed.project.deliveries.len(), 1);
    assert_eq!(parsed.project.deliveries[0].artifacts.len(), 9);
}
