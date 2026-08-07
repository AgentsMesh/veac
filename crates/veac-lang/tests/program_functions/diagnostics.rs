use veac_lang::program::prepare_source;

use super::support::project_with;

#[test]
fn unused_invalid_function_still_fails_compilation() {
    let source = project_with(
        "fn valid(value: scalar) -> scalar { value }\n\
         fn broken(value: scalar) -> scalar { missing + value }\n\
         const time duration = 1s;",
        "duration",
    );
    let diagnostics = prepare_source(&source).unwrap_err();
    assert_eq!(
        diagnostics.as_slice()[0].code,
        "PROGRAM_FUNCTION_EXPRESSION"
    );
    assert!(diagnostics.as_slice()[0]
        .message
        .contains("unknown symbol `missing`"));
}

#[test]
fn direct_and_indirect_recursion_have_stable_cycle_diagnostics() {
    for declarations in [
        "fn again(value: scalar) -> scalar { again(value) }",
        "fn first(value: scalar) -> scalar { second(value) }\n\
         fn second(value: scalar) -> scalar { first(value) }",
    ] {
        let source = project_with(declarations, "1s");
        let diagnostics = prepare_source(&source).unwrap_err();
        assert_eq!(diagnostics.as_slice()[0].code, "PROGRAM_FUNCTION_CYCLE");
        assert!(diagnostics.as_slice()[0]
            .message
            .contains("function call cycle"));
    }
}
