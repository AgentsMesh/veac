use std::fs;

use tempfile::tempdir;
use veac_lang::program::{build_path, build_source, prepare_source};

#[path = "program_functions/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "executable_authored_temporal/support.rs"]
mod temporal_support;

#[test]
fn entry_function_reads_a_forward_declared_static_constant() {
    let source = support::project_with("const time duration = 750ms;", "duration");
    let built = build_source(&source).unwrap();
    assert_eq!(support::result_duration(&built), "750ms");
    assert!(prepare_source(&source)
        .unwrap()
        .entry_function()
        .body()
        .inputs()
        .iter()
        .all(|input| input.name() != "duration"));
}

#[test]
fn constant_dependency_through_a_function_is_resolved_before_execution() {
    let source = support::project_with(
        "fn selected() -> time { later }\n\
         const time duration = selected();\n\
         const time later = 1250ms;",
        "duration",
    );
    let built = build_source(&source).unwrap();
    assert_eq!(support::result_duration(&built), "1250ms");
}

#[test]
fn constant_function_dependency_cycle_fails_with_the_typed_cycle_diagnostic() {
    let source = support::project_with(
        "fn selected() -> time { duration }\nconst time duration = selected();",
        "1s",
    );
    let error = build_source(&source).unwrap_err().as_slice()[0].clone();
    assert_eq!(error.code, "PROGRAM_CONST_CYCLE");
    assert!(error.message.contains("duration -> duration"));
}

#[test]
fn exported_module_function_keeps_its_private_static_constant() {
    let temp = tempdir().unwrap();
    fs::write(
        temp.path().join("timing.veac"),
        "module {\n  const time duration = 1500ms;\n  export fn selected() -> time { duration }\n}\n",
    )
    .unwrap();
    let entry = support::project_with("import \"./timing.veac\" as timing;", "timing.selected()");
    let path = temp.path().join("main.veac");
    fs::write(&path, entry).unwrap();

    let built = build_path(&path).unwrap();
    assert_eq!(support::result_duration(&built), "1500ms");
}

#[test]
fn temporal_function_embeds_a_static_constant_before_residualization() {
    let source = temporal_support::MAIN.replace(
        "fn pulse(value: scalar) -> scalar { clamp(value * 2.0, 0.0, 1.0) }",
        "const scalar gain = 0.5;\nfn pulse(value: scalar) -> scalar { value * gain }",
    );
    let built = build_source(&source).unwrap();
    assert_eq!(
        temporal_support::evaluate(
            built.envelope(),
            veac_ir::TemporalValue::Scalar { value: 0.5 }
        ),
        veac_ir::TemporalValue::Scalar { value: 0.25 }
    );
}
