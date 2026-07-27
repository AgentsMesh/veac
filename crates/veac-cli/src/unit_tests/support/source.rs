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
  output video main {
    sequence main;
    file-name "render.mp4";
    encoding {
      container mp4;
      optimize-for-streaming true;
      video { codec h264; pixel-format yuv420p; }
      audio none;
      captions discard;
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
  output video main {
    sequence main;
    file-name "media-render.mp4";
    encoding {
      container mp4;
      optimize-for-streaming true;
      video { codec h264; pixel-format yuv420p; }
      audio none;
      captions discard;
    }
  }
}"#;
