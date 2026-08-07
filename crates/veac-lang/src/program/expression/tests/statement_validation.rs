use crate::program::expression::validate_function_statement;

#[test]
fn standalone_statement_parser_accepts_one_complete_statement() {
    for source in [
        "let value = 1;",
        "var value: int = 1;",
        "set value = value + 1;",
    ] {
        validate_function_statement(source).unwrap();
    }
}

#[test]
fn standalone_statement_parser_rejects_non_statement_or_extra_input() {
    for source in [
        "let value = 1",
        "value + 1",
        "let value = 1; let other = 2;",
        "set value = 2; trailing",
    ] {
        assert!(validate_function_statement(source).is_err(), "{source}");
    }
}
