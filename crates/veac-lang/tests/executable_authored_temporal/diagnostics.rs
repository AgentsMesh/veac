use veac_lang::program::{build_source, prepare_source};

use super::support::MAIN;

#[test]
fn duplicate_authored_sink_fails_during_prepare() {
    let declaration = "animate visual-opacity on clip(@demo, @main, @visual, @first) { progress }";
    let source = MAIN.replacen("fn main", &format!("{declaration}\nfn main"), 1);
    let error = prepare_source(&source).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_TEMPORAL_SINK_DUPLICATE");
}

#[test]
fn authored_target_must_exist_in_the_emitted_static_graph() {
    let source = MAIN.replace("@first)", "@missing)");
    let error = build_source(&source).unwrap_err();
    assert!(error.as_slice()[0]
        .message
        .contains("EXECUTABLE_TEMPORAL_SINK_MISSING"));
}

#[test]
fn invalid_property_and_target_arity_fail_closed() {
    let invalid = MAIN.replace("visual-opacity", "unknown-property");
    assert_eq!(
        prepare_source(&invalid).unwrap_err().as_slice()[0].code,
        "PROGRAM_TEMPORAL_PROPERTY"
    );
    let short = MAIN.replace(
        "clip(@demo, @main, @visual, @first)",
        "clip(@demo, @main, @first)",
    );
    assert_eq!(
        prepare_source(&short).unwrap_err().as_slice()[0].code,
        "PROGRAM_EXPECTED_TOKEN"
    );
}

#[test]
fn module_cannot_own_temporal_declarations() {
    let module = "module { animate visual-opacity on clip(@p, @s, @l, @i) { progress } }";
    assert_eq!(
        prepare_source(module).unwrap_err().as_slice()[0].code,
        "PROGRAM_TEMPORAL_ENTRY_ONLY"
    );
}
