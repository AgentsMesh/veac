use super::lex_with_limit;

#[test]
fn lexer_stops_allocating_tokens_at_its_configured_budget() {
    let errors = lex_with_limit("large.veac", "one two three", 2).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, "PROGRAM_TOKEN_LIMIT");
}

#[test]
fn lexer_bounds_diagnostics_from_invalid_characters() {
    let errors = super::lex("invalid.veac", &"!".repeat(1_000)).unwrap_err();
    assert_eq!(errors.len(), 256);
    assert_eq!(errors.last().unwrap().code, "PROGRAM_DIAGNOSTIC_LIMIT");
}
