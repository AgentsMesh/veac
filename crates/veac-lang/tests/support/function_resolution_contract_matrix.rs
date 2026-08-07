use crate::program::prepare_source;

const MAIN: &str = r#"
fn main(context: Context) -> Project {
  let timeline = sequence(identifier("main"), "函数诊断",
    sequence_settings(canvas(16px, 16px), frame_rate(1, 1), 8000));
  project(identifier("functions"), project_settings(60))
    .with_sequence(timeline).entry(timeline)
}"#;

fn error(declarations: &str) -> &'static str {
    let source = format!("{declarations}\n{MAIN}");
    prepare_source(&source).unwrap_err().as_slice()[0].code
}

#[test]
fn function_compilation_maps_body_return_cycle_and_signature_diagnostics() {
    let cases = [
        (
            "fn broken() -> int { missing }",
            "PROGRAM_FUNCTION_EXPRESSION",
        ),
        ("fn wrong() -> int { true }", "PROGRAM_FUNCTION_RETURN_TYPE"),
        (
            "fn first() -> int { second() } fn second() -> int { first() }",
            "PROGRAM_FUNCTION_CYCLE",
        ),
        (
            "fn duplicate(value: int, value: int) -> int { value }",
            "PROGRAM_DUPLICATE_PARAMETER",
        ),
        (
            "fn unknown(value: Missing) -> int { 0 }",
            "PROGRAM_UNKNOWN_TYPE",
        ),
        (
            "struct Holder {} impl Holder @holder { fn broken(self) -> int { missing } }",
            "PROGRAM_FUNCTION_EXPRESSION",
        ),
    ];
    for (source, code) in cases {
        assert_eq!(error(source), code, "{source}");
    }
}

#[test]
fn function_parameter_budget_reports_the_first_excess_parameter() {
    let parameters = (0..=64)
        .map(|index| format!("value{index}: int"))
        .collect::<Vec<_>>()
        .join(", ");
    assert_eq!(
        error(&format!("fn large({parameters}) -> int {{ 0 }}")),
        "PROGRAM_FUNCTION_PARAMETER_LIMIT"
    );
}
