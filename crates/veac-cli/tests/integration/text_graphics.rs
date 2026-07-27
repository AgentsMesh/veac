use super::support::*;

const SOURCE: &str = r##"
project agent-graphics {
  settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main;
  sequence main {
    layer visual layers {
      item backdrop {
        source generated gradient linear {
          from 0% 0%; to 100% 100%;
          stop 0% #112233ff; stop 100% #aabbccff;
        }
        record { at 0s; duration 1s; }
      }
      item mark {
        source generated shape {
          geometry rounded-rectangle { bounds 0% 0% 100% 100%; radius 8px; }
          fill solid #00ff00ff;
          stroke 2px solid #ffffffff;
        }
        record { at 1s; duration 1s; }
      }
      item words {
        source text {
          content "Hello";
          style { font family "Inter"; size 32px; fill #ffffffff; }
          layout { box-width 300px; box-height 80px; wrap word; overflow clip; }
        }
        record { at 2s; duration 1s; }
      }
    }
  }
}
"##;

#[test]
fn cli_compile_and_format_preserve_agent_authored_text_and_graphics() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, SOURCE);
    let output = temp.path().join("project.json");
    veac()
        .args([
            "compile",
            source.to_str().unwrap(),
            "--emit-ir",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();
    let json: serde_json::Value = serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    let clips = &json["project"]["sequences"][0]["tracks"][0]["clips"];
    assert_eq!(clips[0]["source"]["generator"]["type"], "gradient");
    assert_eq!(
        clips[1]["source"]["generator"]["shape"]["geometry"]["type"],
        "rounded_rectangle"
    );
    assert_eq!(clips[2]["source"]["style"]["layout"]["wrap"], "word");

    veac()
        .args(["fmt", source.to_str().unwrap()])
        .assert()
        .success();
    veac()
        .args(["fmt", source.to_str().unwrap(), "--check"])
        .assert()
        .success();
    let formatted = std::fs::read_to_string(source).unwrap();
    assert!(formatted.contains("source generated gradient linear {"));
    assert!(formatted.contains("record {"));
}
