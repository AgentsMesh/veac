use super::support::*;

const UNUSED_SOURCE: &str = r#"
project unused {
  settings { timebase 1/600; canvas 32px by 24px; frame-rate 10fps; sample-rate 48000hz; }
  entry sequence main;
  resource video offline {
    locator local { path "missing.mp4"; }
    streams { video auto; audio disabled; }
  }
  sequence main {
    layer visual base {
      item background { source generated solid { color #000000ff; } record { at 0s; duration 200ms; } }
    }
  }
  output video main {
    sequence main; file-name "unused.mp4";
    encoding { container mp4; video { codec h264; pixel-format yuv420p; } audio none; captions discard; }
  }
}
"#;

const SOLO_SOURCE: &str = r#"
project solo-folding {
  settings { timebase 1/600; canvas 32px by 24px; frame-rate 10fps; sample-rate 48000hz; }
  entry sequence main;
  resource video offline {
    locator local { path "missing.mp4"; }
    streams { video auto; audio disabled; }
  }
  sequence main {
    layer visual chosen {
      item background { source generated solid { color #000000ff; } record { at 0s; duration 200ms; } }
    }
    layer video suppressed {
      item offline-1 {
        source media resource offline;
        record { at 0s; duration 200ms; }
        mapping linear { from 0s; to 200ms; }
      }
    }
  }
  output video main {
    sequence main; file-name "solo.mp4";
    encoding { container mp4; video { codec h264; pixel-format yuv420p; } audio none; captions discard; }
  }
}
"#;

const MULTI_OUTPUT_SOURCE: &str = r#"
project selected-output {
  settings { timebase 1/600; canvas 32px by 24px; frame-rate 10fps; sample-rate 48000hz; }
  entry sequence main;
  resource video offline {
    locator local { path "missing.mp4"; }
    streams { video auto; audio disabled; }
  }
  sequence main {
    layer visual base {
      item background { source generated solid { color #000000ff; } record { at 0s; duration 200ms; } }
    }
  }
  sequence offline-sequence {
    layer video media {
      item offline-1 {
        source media resource offline;
        record { at 0s; duration 200ms; }
        mapping linear { from 0s; to 200ms; }
      }
    }
  }
  output video main {
    sequence main; file-name "main.mp4";
    encoding { container mp4; video { codec h264; pixel-format yuv420p; } audio none; captions discard; }
  }
  output video offline {
    sequence offline-sequence; file-name "offline.mp4";
    encoding { container mp4; video { codec h264; pixel-format yuv420p; } audio none; captions discard; }
  }
}
"#;

#[test]
fn unused_missing_and_remote_materials_do_not_block_planning() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, UNUSED_SOURCE);
    veac()
        .args(["plan", project.to_str().unwrap()])
        .assert()
        .success();

    let mut envelope = read_project(&project);
    envelope.project.materials[0].source = veac_ir::MaterialSource::Remote {
        uri: "https://example.test/offline.mp4".into(),
    };
    write_project(&project, &envelope);
    veac()
        .args(["plan", project.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn solo_folding_ignores_suppressed_offline_tracks() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, SOLO_SOURCE);
    let mut envelope = read_project(&project);
    envelope.project.sequences[0].tracks[0].state.solo = true;
    write_project(&project, &envelope);
    veac()
        .args(["plan", project.to_str().unwrap()])
        .assert()
        .success();

    let mut envelope = read_project(&project);
    envelope.project.sequences[0].tracks[0].state.solo = false;
    write_project(&project, &envelope);
    veac()
        .args(["plan", project.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("PATH_UNAVAILABLE"));
}

#[test]
fn selected_output_hydrates_only_its_reachable_sequence_graph() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, MULTI_OUTPUT_SOURCE);
    veac()
        .args(["plan", project.to_str().unwrap(), "--config", "out_main"])
        .assert()
        .success();
    veac()
        .args(["plan", project.to_str().unwrap(), "--config", "out_offline"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("PATH_UNAVAILABLE"));
    veac()
        .args(["plan", project.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("RENDER_CONFIG_REQUIRED"));
}

fn read_project(path: &std::path::Path) -> veac_ir::ProjectEnvelope {
    veac_ir::decode_canonical_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn write_project(path: &std::path::Path, envelope: &veac_ir::ProjectEnvelope) {
    std::fs::write(path, veac_ir::canonical_json(envelope).unwrap()).unwrap();
}
