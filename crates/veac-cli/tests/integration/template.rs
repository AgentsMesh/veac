use std::path::{Path, PathBuf};

use serde_json::json;

use super::support::*;

const SOURCE: &str = r#"
project template-cli-e2e {
  settings {
    timebase 1/1000; canvas 1280px by 720px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main;
  resource video unused {
    locator local { path "clip.mp4"; }
    streams { video auto; audio disabled; }
  }
  sequence main {
    layer visual titles {
      item title {
        source text { content "Before"; }
        record { at 0s; duration 1s; }
        template-slot text;
      }
    }
  }
}
"#;

#[test]
fn template_propose_emits_only_an_applicable_edit_batch() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, SOURCE);
    let request = request(&temp, &project);
    let output = veac()
        .args(["template", "propose", path(&project), path(&request)])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let batch: veac_ir::EditBatch = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(batch.operation_id.to_string(), "op_template_cli_e2e");
    let batch_file = temp.path().join("batch.json");
    std::fs::write(&batch_file, output.stdout).unwrap();
    veac()
        .args(["edit", path(&project), path(&batch_file)])
        .assert()
        .success();
    let envelope = read_project(&project);
    let clip = &envelope.project.sequences[0].tracks[0].clips[0];
    assert!(!clip.template_editable_text);
    assert!(matches!(
        &clip.source,
        veac_ir::ClipSource::Text { text, .. } if text == "After"
    ));

    veac()
        .args(["schema", "--contract", "template-fill-request"])
        .assert()
        .success()
        .stdout(predicate::str::contains("media_bindings"));
}

#[test]
fn template_propose_rejects_invalid_requests_without_output() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, SOURCE);
    let request = request(&temp, &project);
    let mut value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&request).unwrap()).unwrap();
    value["unknown"] = json!(true);
    std::fs::write(&request, serde_json::to_vec(&value).unwrap()).unwrap();
    let output = temp.path().join("batch.json");
    veac()
        .args([
            "template",
            "propose",
            path(&project),
            path(&request),
            "-o",
            path(&output),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("WORKFLOW_JSON"));
    assert!(!output.exists());
}

#[test]
fn template_propose_never_overwrites_project_request_or_material() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, SOURCE);
    let request = request(&temp, &project);
    let material = temp.path().join("clip.mp4");
    std::fs::write(&material, "material sentinel").unwrap();
    let protected = [project.clone(), request.clone(), material.clone()];
    let original: Vec<_> = protected
        .iter()
        .map(|file| std::fs::read(file).unwrap())
        .collect();
    for destination in &protected {
        veac()
            .args([
                "template",
                "propose",
                path(&project),
                path(&request),
                "-o",
                path(destination),
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains("OUTPUT_OVERWRITES_INPUT"));
    }
    for (file, expected) in protected.iter().zip(original) {
        assert_eq!(std::fs::read(file).unwrap(), expected);
    }
}

fn request(temp: &TempDir, project: &Path) -> PathBuf {
    let envelope = read_project(project);
    let clip_id = envelope.project.sequences[0].tracks[0].clips[0]
        .id
        .to_string();
    let file = temp.path().join("request.json");
    let request = json!({
        "schema": "https://veac.dev/schemas/template-fill-request",
        "schema_version": 1,
        "operation_id": "op_template_cli_e2e",
        "base_revision": 0,
        "media_bindings": [],
        "text_bindings": [{"clip_id": clip_id, "text": "After"}]
    });
    std::fs::write(&file, serde_json::to_vec(&request).unwrap()).unwrap();
    file
}

fn read_project(path: &Path) -> veac_ir::ProjectEnvelope {
    veac_ir::decode_canonical_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn path(path: &Path) -> &str {
    path.to_str().unwrap()
}
