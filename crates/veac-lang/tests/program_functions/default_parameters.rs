use std::fs;

use tempfile::tempdir;
use veac_lang::program::{
    apply_executable_source_edit_path, build_path, build_source, prepare_source,
};
use veac_lang::source_edit::{
    ExpressionSite, ExpressionSource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

use super::support::{project_with, project_with_durations, result_duration};

#[test]
fn positional_and_named_calls_fill_default_slots() {
    let declarations = r#"fn duration(
  base: time,
  padding: time = 250ms,
  multiplier: scalar = 2.0,
) -> time { (base + padding) * multiplier }"#;
    let source = project_with_durations(
        declarations,
        &[
            "duration(1s)",
            "duration(1s, 500ms)",
            "duration(base: 1s, multiplier: 3.0)",
        ],
    );
    let built = build_source(&source).unwrap();
    assert_eq!(
        (0..3)
            .map(|index| super::support::item_duration(&built, 0, 0, index))
            .collect::<Vec<_>>(),
        ["2500ms", "3s", "3750ms"]
    );
}

#[test]
fn defaults_use_the_declaring_module_environment() {
    let module = r#"module {
  const time padding = 400ms;
  fn local() -> time { padding }
  export fn duration(value: time = local()) -> time { value }
}"#;
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("timing.veac"), module).unwrap();
    let entry = project_with(
        "import \"./timing.veac\" as timing;\nconst time padding = 9s;",
        "timing.duration()",
    );
    let path = temp.path().join("main.veac");
    fs::write(&path, entry).unwrap();
    assert_eq!(result_duration(&build_path(&path).unwrap()), "400ms");
}

#[test]
fn invalid_default_contracts_fail_statically() {
    for (declaration, expected) in [
        (
            "fn broken(optional: time = 1s, required: time) -> time { required }",
            "PROGRAM_REQUIRED_PARAMETER_AFTER_DEFAULT",
        ),
        (
            "fn broken(value: time = 2px) -> time { value }",
            "PROGRAM_FUNCTION_EXPRESSION",
        ),
        (
            "fn broken(value: time = graph_sequence(identifier(\"x\"))) -> time { value }",
            "PROGRAM_FUNCTION_EXPRESSION",
        ),
    ] {
        let source = project_with(declaration, "1s");
        let error = prepare_source(&source).unwrap_err().as_slice()[0].clone();
        assert_eq!(error.code, expected, "{}", error.message);
    }
}

#[test]
fn mutable_parameter_defaults_report_the_effect_contract() {
    let source = project_with(
        "fn broken(value: time = { var result = 1s; set result = 2s; result }) -> time { value }",
        "1s",
    );
    let error = prepare_source(&source).unwrap_err().as_slice()[0].clone();
    assert_eq!(error.code, "PROGRAM_FUNCTION_EXPRESSION");
    assert!(
        error
            .message
            .contains("EXPRESSION_PARAMETER_DEFAULT_EFFECT"),
        "{}",
        error.message
    );
}

#[test]
fn source_index_edits_a_parameter_default_and_reexecutes() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("main.veac");
    let source = project_with(
        "fn duration(value: time = 1s) -> time { value }",
        "duration()",
    );
    fs::write(&path, &source).unwrap();
    let built = build_path(&path).unwrap();
    let target = SourceNodeRef::function("main.veac", "duration");
    let site = ExpressionSite::ParameterDefault {
        parameter: "value".to_owned(),
    };
    let index = built.source_index().unwrap();
    assert_eq!(index.expression(&target, &site).unwrap().source, "1s");
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_default_parameter").unwrap(),
        built.source_revision().unwrap(),
    );
    batch.operations.push(SourceEditOperation::SetExpression {
        target,
        site,
        expression: ExpressionSource {
            source: "2500ms".to_owned(),
        },
    });
    let preview = apply_executable_source_edit_path(&path, &batch).unwrap();
    assert_eq!(result_duration(&preview.built), "2500ms");
    assert!(preview.source().unwrap().contains("value: time = 2500ms"));
}
