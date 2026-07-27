use tempfile::tempdir;

use super::support::{canonical_project, source_file, FakeEnvironment, GENERATED_SOURCE};
use crate::Cli;

const TEMPLATE_SOURCE: &str = r#"
project dispatch-template {
  settings {
    timebase 1/1000;
    canvas 32px by 24px;
    frame-rate 10fps;
    sample-rate 48000hz;
  }
  entry sequence main;
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

fn dispatch(args: &[&str], environment: &FakeEnvironment) {
    let cli = Cli::try_parse_from(args).unwrap();
    crate::execute_with_environment(cli, environment).unwrap();
}

#[test]
fn dispatcher_reaches_every_non_compile_command_variant() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let document = veac_lang::authoring::parse(GENERATED_SOURCE).unwrap();
    let formatted = veac_lang::authoring::format_document(&document);
    let source = source_file(&temp, &formatted);
    let media = temp.path().join("probe.bin");
    let output = temp.path().join("deliveries");
    let manifest = temp.path().join("build.json");
    let package = temp.path().join("package");
    let template_temp = tempdir().unwrap();
    let template_project = canonical_project(&template_temp, TEMPLATE_SOURCE);
    let template_request = template_temp.path().join("template-request.json");
    let template_batch = template_temp.path().join("template-batch.json");
    let template = crate::canonical::load(&template_project).unwrap();
    let clip_id = template.project.sequences[0].tracks[0].clips[0]
        .id
        .to_string();
    std::fs::write(
        &template_request,
        serde_json::to_vec(&serde_json::json!({
            "schema": veac_template::TEMPLATE_FILL_SCHEMA_ID,
            "schema_version": veac_template::TEMPLATE_FILL_SCHEMA_VERSION,
            "operation_id": "op_dispatch_template",
            "base_revision": 0,
            "media_bindings": [],
            "text_bindings": [{"clip_id": clip_id, "text": "After"}]
        }))
        .unwrap(),
    )
    .unwrap();
    std::fs::write(&media, "fixture").unwrap();
    std::fs::create_dir(&output).unwrap();
    let environment = FakeEnvironment::success();

    dispatch(
        &["veac", "fmt", source.to_str().unwrap(), "--check"],
        &environment,
    );
    dispatch(&["veac", "schema"], &environment);
    dispatch(
        &[
            "veac",
            "template",
            "propose",
            template_project.to_str().unwrap(),
            template_request.to_str().unwrap(),
            "--output",
            template_batch.to_str().unwrap(),
        ],
        &environment,
    );
    dispatch(&["veac", "plan", project.to_str().unwrap()], &environment);
    dispatch(
        &[
            "veac",
            "manifest",
            project.to_str().unwrap(),
            "--output",
            manifest.to_str().unwrap(),
        ],
        &environment,
    );
    dispatch(
        &[
            "veac",
            "package",
            project.to_str().unwrap(),
            "--destination",
            package.to_str().unwrap(),
        ],
        &environment,
    );
    dispatch(
        &[
            "veac",
            "render",
            project.to_str().unwrap(),
            "--destination",
            output.to_str().unwrap(),
        ],
        &environment,
    );
    dispatch(&["veac", "probe", media.to_str().unwrap()], &environment);
    assert_eq!(environment.executed.borrow().len(), 1);
}
