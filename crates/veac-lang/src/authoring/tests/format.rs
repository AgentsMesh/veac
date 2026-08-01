use super::super::{format_document, parse};

const SOURCE: &str = r#"
project demo {
  settings {
    timebase 1/1000000;
    canvas 1920px by 1080px;
    frame-rate 30fps;
    sample-rate 48000hz;
  }
  entry sequence main;
  resource video host {
    locator local { path "assets/host.mp4"; }
    streams { video auto; audio auto; }
  }
  resource image remote-card {
    locator remote {
      uri "https://example.invalid/card.png";
      identity sha256 "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    }
    streams { video auto; audio disabled; }
  }
  sequence nested {
    layer visual art {
      item shape {
        source generated shape {
          geometry rectangle { bounds 0% 0% 100% 100%; }
          fill solid #20242aff;
        }
        record { at 0s; duration 2s; }
      }
    }
  }
  sequence main {
    layer video picture {
      order 0;
      item hero {
        source media resource host;
        record { at 0s; duration 4s; }
        mapping linear { from 1s; to 5s; }
        modifiers {
          transform hero-motion {
            position { x 0px; y 0px; }
            crop { x 0.1; y 0.1; width 0.8; height 0.8; }
          }
          composite hero-composite {
            opacity 1;
          }
        }
      }
      item reverse {
        source media resource host;
        record { at 4s; duration 2s; }
        mapping curve {
          key start { at 0s; source 5s; interpolation linear; }
          key end { at 2s; source 3s; interpolation ease-in-out; }
        }
      }
      item still {
        source media resource host;
        record { at 6s; duration 2s; }
        mapping freeze { source 3s; }
      }
    }
    layer visual titles {
      order 10;
      item title { source text { content "Hello VEAC"; } record { at 0s; duration 2s; } }
      item nested-card { source sequence sequence nested; record { at 2s; duration 2s; } }
    }
    relation transition intro-cut {
      endpoints { from item hero; to item reverse; }
      timing { duration 300ms; alignment centered; }
      style { dissolve; }
    }
    apply grade {
      scope composite-band {
        from layer picture;
        through layer titles;
      }
      record { at 1s; duration 3s; }
      pipeline {
        stage effect grade-fx {
          type video.color_adjust;
          parameter contrast 1.2;
        }
      }
      mix { opacity 0.9; blend normal; }
    }
  }
  annotation marker intro {
    target item hero;
    span range { at 0s; duration 500ms; }
    payload { label "Opening"; }
    provenance {
      producer "test";
      request-sha256 "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
      response-sha256 "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    }
  }
  delivery master {
    sequence main;
    raster { canvas 1920px by 1080px; frame-rate 30fps; captions burn-in; }
    artifact video master {
      target file "master.mp4";
      mux mp4 {
        video h264 { pixel-format yuv420p; }
        audio aac { sample-rate 48khz; channel-layout stereo; }
      }
    }
  }
}
"#;

#[test]
fn canonical_format_is_parseable_and_idempotent() {
    let parsed = parse(SOURCE).expect("source parses");
    let formatted = format_document(&parsed);
    let reparsed = parse(&formatted).expect("canonical source parses");

    assert_eq!(format_document(&reparsed), formatted);
    assert!(formatted.starts_with("project demo {\n  entry sequence main;"));
    assert!(formatted.contains("relation transition intro-cut"));
    assert!(formatted.contains("effect grade-fx"));
    assert!(!formatted.contains('='));
}
