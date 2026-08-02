use std::path::Path;

use tempfile::tempdir;

use super::support::{
    canonical_project, render, source_file, FakeEnvironment, GENERATED_SOURCE, MEDIA_SOURCE,
};
use crate::{Cli, PlanFormat};

fn parse(args: &[&str]) -> Cli {
    Cli::try_parse_from(args).unwrap()
}

#[test]
fn compile_check_and_check_ir_dispatch_end_to_end() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, GENERATED_SOURCE);
    let ir = temp.path().join("project.json");
    let environment = FakeEnvironment::success();
    crate::execute_with_environment(
        parse(&[
            "veac",
            "compile",
            source.to_str().unwrap(),
            "--emit-ir",
            ir.to_str().unwrap(),
            "--revision",
            "9",
        ]),
        &environment,
    )
    .unwrap();
    assert_eq!(crate::canonical::load(&ir).unwrap().project.revision, 9);
    crate::execute_with_environment(
        parse(&["veac", "check", source.to_str().unwrap()]),
        &environment,
    )
    .unwrap();
    crate::execute_with_environment(
        parse(&["veac", "check-ir", ir.to_str().unwrap()]),
        &environment,
    )
    .unwrap();
    crate::execute(parse(&["veac", "check", source.to_str().unwrap()])).unwrap();
}

#[test]
fn compile_supports_stdout_and_protects_its_source() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, GENERATED_SOURCE);
    crate::commands::compile(&source, None, 0).unwrap();
    crate::commands::compile(&source, Some(Path::new("-")), 0).unwrap();
    assert!(crate::commands::compile(&source, Some(&source), 0)
        .unwrap_err()
        .to_string()
        .contains("OUTPUT_OVERWRITES_INPUT"));
    let media_source = source_file(&temp, MEDIA_SOURCE);
    assert!(
        crate::commands::compile(&media_source, Some(&temp.path().join("clip.mp4")), 0,)
            .unwrap_err()
            .to_string()
            .contains("OUTPUT_OVERWRITES_INPUT")
    );
}

#[test]
fn formatter_supports_check_stdout_and_atomic_in_place_modes() {
    let temp = tempdir().unwrap();
    let compact = GENERATED_SOURCE.replace('\n', " ");
    let source = source_file(&temp, &compact);
    assert!(crate::commands::format(&source, true, false)
        .unwrap_err()
        .to_string()
        .contains("FORMAT_REQUIRED"));
    crate::commands::format(&source, false, true).unwrap();
    crate::commands::format(&source, false, false).unwrap();
    crate::commands::format(&source, true, false).unwrap();
    crate::commands::format(&source, false, false).unwrap();
}

#[test]
fn schema_plan_probe_and_render_dispatch() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let media = temp.path().join("clip.bin");
    std::fs::write(&media, "fixture").unwrap();
    let output = temp.path().join("render.mp4");
    let environment = FakeEnvironment::success();

    crate::commands::plan(&project, None, None, PlanFormat::Json, &environment).unwrap();
    crate::commands::probe(&media, &environment).unwrap();
    render(&project, Some(temp.path()), &environment).unwrap();
    let calls = environment.executed.borrow();
    assert_eq!(calls.len(), 1);
    assert_eq!(std::fs::read(output).unwrap(), b"rendered");
}

#[test]
fn render_and_probe_propagate_environment_failures() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let media = temp.path().join("clip.bin");
    std::fs::write(&media, "fixture").unwrap();
    let failed = FakeEnvironment {
        fail_probe: true,
        fail_execute: true,
        ..FakeEnvironment::success()
    };
    assert!(crate::commands::probe(&media, &failed)
        .unwrap_err()
        .to_string()
        .contains("FAKE_PROBE"));
    assert!(render(&project, None, &failed)
        .unwrap_err()
        .to_string()
        .contains("FAKE_RENDER"));
}

#[test]
fn codegen_diagnostics_remain_machine_readable() {
    let errors =
        veac_codegen::emitter::CodegenErrors::one(veac_codegen::emitter::CodegenDiagnostic {
            kind: veac_codegen::emitter::CodegenErrorKind::InvalidPlan,
            code: "TEST_CODEGEN",
            object_id: Some("itm_broken".into()),
            location: "render-plan:object:itm_broken".into(),
            message: "invalid plan".into(),
            suggested_repair: Some("regenerate the render plan".into()),
        });
    assert_eq!(
        crate::diagnostic::codegen(errors).to_string(),
        "error[TEST_CODEGEN]: render-plan:object:itm_broken: invalid plan\n  object: itm_broken\n  help: regenerate the render plan"
    );
}
