use super::support::*;

const MULTI_DELIVERY_SOURCE: &str = r#"
project multi-delivery {
  settings { timebase 1/600; canvas 32px by 24px; frame-rate 10fps; sample-rate 48000hz; }
  entry sequence main;
  sequence main {
    layer visual base {
      item background {
        source generated solid { color #24a148ff; }
        record { at 0s; duration 200ms; }
      }
    }
  }
  delivery main {
    sequence main;
    raster { canvas 32px by 24px; frame-rate 10fps; captions discard; }
    artifact video master {
      target file "authored-master.mp4";
      mux mp4 {
        layout standard;
        video h264 { pixel-format yuv420p; alpha opaque; color-space source; rate-control crf { value 23; } gop automatic; b-frames automatic; profile automatic; level automatic; }
        audio none; passes single; accelerator auto;
      }
    }
    artifact image-sequence frames {
      target pattern "authored-frame-%04d.png";
      numbering from 1; encode png;
    }
  }
}
"#;

#[test]
fn real_render_emits_and_resumes_every_authored_deliverable() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, MULTI_DELIVERY_SOURCE);
    let destination = temp.path().join("deliveries");
    std::fs::create_dir(&destination).unwrap();
    veac()
        .args([
            "render",
            project.to_str().unwrap(),
            "--config",
            "out_main",
            "--destination",
            destination.to_str().unwrap(),
        ])
        .assert()
        .success();
    let master = destination.join("authored-master.mp4");
    let first_frame = destination.join("authored-frame-0001.png");
    assert_eq!(video_dimensions(&master), "32x24");
    assert_eq!(video_dimensions(&first_frame), "32x24");
    let frames = std::fs::read_dir(&destination)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("authored-frame-")
        })
        .count();
    assert_eq!(frames, 2);

    veac()
        .args([
            "render",
            project.to_str().unwrap(),
            "--config",
            "out_main",
            "--destination",
            destination.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("reused"));
}
