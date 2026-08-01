use tempfile::tempdir;

use super::support::{canonical_project, FakeEnvironment};
use crate::arguments::SubstitutionPolicy::{Original, Require};

const IMAGE_SOURCE: &str = r#"
project image-proxy-test {
  settings {
    timebase 1/600;
    canvas 32px by 24px;
    frame-rate 10fps;
    sample-rate 48000hz;
  }
  entry sequence main;
  resource image still { locator local { path "still.png"; } }
  sequence main {
    layer visual base {
      item still {
        source media resource still;
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
}
"#;

#[test]
fn required_proxy_policy_keeps_static_image_on_its_original_binding() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("still.png"), b"original still").unwrap();
    let project = canonical_project(&temp, IMAGE_SOURCE);
    let mut environment = FakeEnvironment::success();
    environment.observed =
        veac_runtime::asset::sha256_identity(&temp.path().join("still.png")).unwrap();
    environment.filters.insert("loop".to_owned());

    crate::commands::render(&project, None, None, None, Require, Original, &environment).unwrap();

    assert_eq!(environment.executed.borrow().len(), 1);
    assert_eq!(
        environment.consumed_inputs.borrow()[0],
        vec![b"original still".to_vec()]
    );
}
