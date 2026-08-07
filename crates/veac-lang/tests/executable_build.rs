use veac_lang::program::{build_source, prepare_source};

const SOURCE: &str = r#"
fn main(context: Context) -> Project {
    let timeline = sequence(
        identifier("main"), "主时间线",
        sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
    );
    project(identifier("integration"), project_settings(600))
        .with_sequence(timeline)
        .entry(timeline)
}
"#;

#[test]
fn public_executable_build_api_preserves_source_metadata() {
    let built = build_source(SOURCE).unwrap();
    assert_eq!(built.root_module(), "main.veac");
    assert_eq!(built.entry_function().name(), "main");
    assert_eq!(built.sources()["main.veac"], SOURCE);
    assert!(built.source_index().is_ok());
    assert_eq!(built.type_registry().len(), 0);
    assert_eq!(built.method_registry().len(), 0);
}

#[test]
fn prepared_executable_can_run_more_than_once() {
    let prepared = prepare_source(SOURCE).unwrap();
    assert!(prepared.execute().is_ok());
    assert!(prepared.execute().is_ok());
}

#[test]
fn public_signature_diagnostic_is_stable_and_source_located() {
    let source = "fn main(input: Context) -> Project { input }";
    let diagnostic = prepare_source(source).unwrap_err();
    let diagnostic = &diagnostic.as_slice()[0];
    assert_eq!(diagnostic.code, "PROGRAM_EXECUTABLE_MAIN_PARAMETER_NAME");
    assert_eq!(
        &source[diagnostic.span.start..diagnostic.span.end],
        "input: Context"
    );
}
