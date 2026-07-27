use std::path::PathBuf;

use serde_json::json;
use tempfile::{tempdir, TempDir};

use crate::arguments::TemplateCommand;

const SOURCE: &str = r#"
project template-cli {
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
fn template_command_emits_an_applicable_edit_batch() {
    let temp = tempdir().unwrap();
    let (project_file, request_file) = fixture(&temp);
    let output = temp.path().join("batch.json");
    run(project_file.clone(), request_file, Some(output.clone())).unwrap();
    let batch = veac_ir::decode_edit_batch_json(&std::fs::read_to_string(output).unwrap()).unwrap();
    let project = crate::canonical::load(&project_file).unwrap();
    let outcome = veac_ir::apply_edit_batch(&project, &batch);
    let veac_ir::EditOutcome::Applied { project, .. } = outcome else {
        panic!("template proposal must apply atomically");
    };
    let clip = &project.project.sequences[0].tracks[0].clips[0];
    assert!(!clip.template_editable_text);
    assert!(matches!(
        &clip.source,
        veac_ir::ClipSource::Text { text, .. } if text == "After"
    ));
}

#[test]
fn template_command_rejects_unknown_request_fields_before_writing() {
    let temp = tempdir().unwrap();
    let (project, request) = fixture(&temp);
    let mut value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&request).unwrap()).unwrap();
    value["unknown"] = json!(true);
    std::fs::write(&request, serde_json::to_vec(&value).unwrap()).unwrap();
    let output = temp.path().join("batch.json");
    let error = run(project, request, Some(output.clone())).unwrap_err();
    assert!(error.to_string().contains("WORKFLOW_JSON"));
    assert!(!output.exists());
}

#[test]
fn replacement_local_materials_are_protected_relative_to_the_project() {
    let temp = tempdir().unwrap();
    let (project, _) = fixture(&temp);
    let envelope = crate::canonical::load(&project).unwrap();
    let clip_id = envelope.project.sequences[0].tracks[0].clips[0].id.clone();
    let material = envelope.project.materials[0].clone();
    let mut request = veac_template::TemplateFillRequest {
        schema: veac_template::TEMPLATE_FILL_SCHEMA_ID.to_owned(),
        schema_version: veac_template::TEMPLATE_FILL_SCHEMA_VERSION,
        operation_id: veac_ir::OperationId::new("op_path_guard").unwrap(),
        base_revision: 0,
        media_bindings: vec![veac_template::MediaBinding { clip_id, material }],
        text_bindings: vec![],
    };
    assert_eq!(
        crate::commands::replacement_material_paths(&request, &project),
        vec![temp.path().join("clip.mp4")]
    );
    request.media_bindings[0].material.source = veac_ir::MaterialSource::Remote {
        uri: "https://example.test/replacement.mp4".to_owned(),
    };
    assert!(crate::commands::replacement_material_paths(&request, &project).is_empty());
}

fn fixture(temp: &TempDir) -> (PathBuf, PathBuf) {
    let project = super::support::canonical_project(temp, SOURCE);
    let envelope = crate::canonical::load(&project).unwrap();
    let clip_id = envelope.project.sequences[0].tracks[0].clips[0]
        .id
        .to_string();
    let request = temp.path().join("request.json");
    let value = json!({
        "schema": "https://veac.dev/schemas/template-fill-request",
        "schema_version": 1,
        "operation_id": "op_template_cli",
        "base_revision": 0,
        "media_bindings": [],
        "text_bindings": [{"clip_id": clip_id, "text": "After"}]
    });
    std::fs::write(&request, serde_json::to_vec(&value).unwrap()).unwrap();
    (project, request)
}

fn run(project: PathBuf, request: PathBuf, output: Option<PathBuf>) -> crate::CliResult {
    crate::commands::template(
        TemplateCommand::Propose {
            project,
            request,
            output,
        },
        &super::support::FakeEnvironment::success(),
    )
}
