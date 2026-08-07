use crate::program::expression::{referenced_symbols, MAX_CALL_ARGUMENTS};

fn code(source: &str) -> &'static str {
    referenced_symbols(source).unwrap_err().code()
}

#[test]
fn temporal_attachment_reports_each_closed_header_error() {
    assert_eq!(
        code("animate unknown on clip(owner) { progress }"),
        "EXPRESSION_TEMPORAL_PROPERTY"
    );
    assert_eq!(
        code("animate 1 on clip(owner) { progress }"),
        "EXPRESSION_TEMPORAL_PROPERTY"
    );
    assert_eq!(
        code("animate visual-opacity clip(owner) { progress }"),
        "EXPRESSION_EXPECTED_TOKEN"
    );
    assert_eq!(
        code("animate visual-opacity on unknown(owner) { progress }"),
        "EXPRESSION_TEMPORAL_TARGET"
    );
    assert_eq!(
        code("animate visual-opacity on 1(owner) { progress }"),
        "EXPRESSION_TEMPORAL_TARGET"
    );
}

#[test]
fn temporal_target_argument_delimiters_and_limits_fail_at_the_header() {
    for source in [
        "animate visual-opacity on clip owner) { progress }",
        "animate visual-opacity on clip(owner,) { progress }",
        "animate visual-opacity on clip(owner { progress }",
        "animate visual-opacity on clip(owner) progress",
        "animate visual-opacity on clip(owner) { progress",
    ] {
        assert!(referenced_symbols(source).is_err(), "{source}");
    }
    let arguments = (0..=MAX_CALL_ARGUMENTS)
        .map(|index| format!("value{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    assert_eq!(
        code(&format!(
            "animate visual-opacity on clip({arguments}) {{ progress }}"
        )),
        "EXPRESSION_CALL_ARGUMENT_LIMIT"
    );
}
