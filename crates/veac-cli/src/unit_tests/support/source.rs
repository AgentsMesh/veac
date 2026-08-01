pub(crate) const GENERATED_SOURCE: &str = r#"project cli-test {
  settings {
    timebase 1/1000;
    canvas 32px by 24px;
    frame-rate 10fps;
    sample-rate 48000hz;
  }
  entry sequence main;
  sequence main {
    layer visual base {
      item background {
        source generated solid { color #112233ff; }
        record { at 0s; duration 200ms; }
      }
    }
  }
  delivery main {
    sequence main;
    raster { canvas 32px by 24px; frame-rate 10fps; captions discard; }
    artifact video main {
      target file "render.mp4";
      mux mp4 {
        layout fast-start;
        video h264 { pixel-format yuv420p; alpha opaque; color-space source; rate-control crf { value 23; } gop automatic; b-frames automatic; profile automatic; level automatic; }
        audio none; passes single; accelerator auto;
      }
    }
  }
}"#;

pub(crate) const MEDIA_SOURCE: &str = r#"project media-test {
  settings {
    timebase 1/1000;
    canvas 32px by 24px;
    frame-rate 10fps;
    sample-rate 48000hz;
  }
  entry sequence main;
  resource video footage {
    locator local { path "clip.mp4"; }
    streams { video auto; audio disabled; }
  }
  sequence main {
    layer video base {
      item footage-1 {
        source media resource footage;
        record { at 0s; duration 200ms; }
        mapping linear { from 0s; to 200ms; }
      }
    }
  }
  delivery main {
    sequence main;
    raster { canvas 32px by 24px; frame-rate 10fps; captions discard; }
    artifact video main {
      target file "media-render.mp4";
      mux mp4 {
        layout fast-start;
        video h264 { pixel-format yuv420p; alpha opaque; color-space source; rate-control crf { value 23; } gop automatic; b-frames automatic; profile automatic; level automatic; }
        audio none; passes single; accelerator auto;
      }
    }
  }
}"#;
