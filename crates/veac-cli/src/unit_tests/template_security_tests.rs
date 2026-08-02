use serde_json::to_vec_pretty;
use tempfile::{tempdir, TempDir};
use veac_ir::{ItemId, MaterialSource, OperationId};

use crate::arguments::TemplateCommand;

use super::support::{canonical_project, identity, snapshot, FakeEnvironment};

const SOURCE: &str = r#"
project template-security {
  settings {
    timebase 1/1000; canvas 32px by 24px;
    frame-rate 10fps; sample-rate 48000hz;
  }
  entry sequence main;
  resource video hero {
    locator local { path "placeholder.mp4"; }
    streams { video auto; audio disabled; }
  }
  sequence main {
    layer video picture {
      item hero {
        source media resource hero;
        record { at 0s; duration 1s; }
        mapping linear { from 0s; to 1s; }
        template-slot media {
          accepts video; fill fit-duration; label "Hero";
        }
      }
    }
  }
}
"#;

#[test]
fn local_replacement_requires_exact_identity_and_probe_facts() {
    let temp = tempdir().unwrap();
    let (project, request) = fixture(&temp);
    let output = temp.path().join("batch.json");
    let mut forged: veac_template::TemplateFillRequest =
        serde_json::from_slice(&std::fs::read(&request).unwrap()).unwrap();
    forged.media_bindings[0]
        .material
        .probe
        .as_mut()
        .unwrap()
        .streams[0]
        .video
        .as_mut()
        .unwrap()
        .width = 640;
    std::fs::write(&request, to_vec_pretty(&forged).unwrap()).unwrap();
    let error = run(
        project,
        request,
        output.clone(),
        &FakeEnvironment::success(),
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("TEMPLATE_REPLACEMENT_PROBE_MISMATCH"));
    assert!(!output.exists());
}

#[test]
fn wrong_bytes_missing_files_and_remote_sources_fail_closed() {
    let temp = tempdir().unwrap();
    let (project, request) = fixture(&temp);
    let output = temp.path().join("batch.json");
    let mut wrong = FakeEnvironment::success();
    wrong.observed = identity(0x22);
    let error = run(project.clone(), request.clone(), output.clone(), &wrong).unwrap_err();
    assert!(error
        .to_string()
        .contains("TEMPLATE_REPLACEMENT_IDENTITY_MISMATCH"));

    let mut value: veac_template::TemplateFillRequest =
        serde_json::from_slice(&std::fs::read(&request).unwrap()).unwrap();
    value.media_bindings[0].material.source = MaterialSource::File {
        uri: "missing.mp4".to_owned(),
    };
    std::fs::write(&request, to_vec_pretty(&value).unwrap()).unwrap();
    let error = run(
        project.clone(),
        request.clone(),
        output.clone(),
        &FakeEnvironment::success(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("PATH_UNAVAILABLE"));

    value.media_bindings[0].material.source = MaterialSource::Remote {
        uri: "https://example.test/hero.mp4".to_owned(),
    };
    std::fs::write(&request, to_vec_pretty(&value).unwrap()).unwrap();
    let error = run(
        project,
        request,
        output.clone(),
        &FakeEnvironment::success(),
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("TEMPLATE_REPLACEMENT_UNMATERIALIZED"));
    assert!(!output.exists());
}

#[test]
fn invalid_requests_are_rejected_before_any_replacement_probe() {
    let temp = tempdir().unwrap();
    let (project, request) = fixture(&temp);
    let base: veac_template::TemplateFillRequest =
        serde_json::from_slice(&std::fs::read(&request).unwrap()).unwrap();
    let failed_probe = FakeEnvironment {
        fail_probe: true,
        ..FakeEnvironment::success()
    };
    let mut values = Vec::new();
    for uri in ["/tmp/outside.mp4", "../outside.mp4"] {
        let mut value = base.clone();
        value.media_bindings[0].material.source = MaterialSource::File {
            uri: uri.to_owned(),
        };
        values.push(value);
    }
    let mut duplicate = base.clone();
    duplicate
        .media_bindings
        .push(duplicate.media_bindings[0].clone());
    values.push(duplicate);
    let mut unexpected = base;
    unexpected.media_bindings[0].clip_id = ItemId::new("itm_unexpected").unwrap();
    values.push(unexpected);

    for (index, value) in values.into_iter().enumerate() {
        std::fs::write(&request, to_vec_pretty(&value).unwrap()).unwrap();
        let error = run(
            project.clone(),
            request.clone(),
            temp.path().join(format!("batch-{index}.json")),
            &failed_probe,
        )
        .unwrap_err();
        assert!(error.to_string().contains("TEMPLATE_PROPOSAL_FAILED"));
        assert!(!error.to_string().contains("FAKE_PROBE"));
    }
}

fn fixture(temp: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    let project = canonical_project(temp, SOURCE);
    let envelope = crate::canonical::load(&project).unwrap();
    let clip = &envelope.project.sequences[0].tracks[0].clips[0];
    let mut material = envelope.project.materials[0].clone();
    std::fs::write(temp.path().join("replacement.mp4"), b"fixture").unwrap();
    material.source = MaterialSource::File {
        uri: "replacement.mp4".to_owned(),
    };
    material.identity = Some(identity(0x11));
    material.probe = Some(snapshot(material.stream_intent.clone(), identity(0x11)));
    let request = veac_template::TemplateFillRequest {
        schema: veac_template::TEMPLATE_FILL_SCHEMA_ID.to_owned(),
        schema_version: veac_template::TEMPLATE_FILL_SCHEMA_VERSION,
        operation_id: OperationId::new("op_template_security").unwrap(),
        base_revision: envelope.project.revision,
        media_bindings: vec![veac_template::MediaBinding {
            clip_id: clip.id.clone(),
            material,
        }],
        text_bindings: vec![],
    };
    let path = temp.path().join("request.json");
    std::fs::write(&path, to_vec_pretty(&request).unwrap()).unwrap();
    (project, path)
}

fn run(
    project: std::path::PathBuf,
    request: std::path::PathBuf,
    output: std::path::PathBuf,
    environment: &dyn crate::environment::Environment,
) -> crate::CliResult {
    crate::commands::template(
        TemplateCommand::Propose {
            project,
            request,
            output: Some(output),
        },
        environment,
    )
}
