use super::*;

#[test]
fn aggregate_expression_payload_accepts_exact_boundary_and_rejects_next_byte() {
    let expression = full_expression();
    let count = MAX_SOURCE_EDIT_EXPRESSION_PAYLOAD_BYTES / expression.len();
    assert_eq!(
        count * expression.len(),
        MAX_SOURCE_EDIT_EXPRESSION_PAYLOAD_BYTES
    );
    let mut value = batch();
    value.operations = (0..count).map(|_| operation(&expression)).collect();
    assert_eq!(validate_source_edit_contract(&value), Ok(()));

    value
        .preconditions
        .push(SourcePrecondition::ExpressionEquals {
            target: target(),
            site: ExpressionSite::ItemRecordDuration,
            expression: ExpressionSource {
                source: "1".to_owned(),
            },
        });
    assert!(matches!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::ExpressionPayloadTooLarge { .. })
    ));

    let SourceEditOperation::SetExpression { expression, .. } =
        value.operations.last_mut().unwrap();
    expression.source = invalid_full_expression();
    assert!(matches!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::InvalidExpression(_))
    ));
}

fn full_expression() -> String {
    format!(
        "\"{}\"",
        "x".repeat(MAX_SOURCE_EDIT_SINGLE_EXPRESSION_BYTES - 2)
    )
}

fn invalid_full_expression() -> String {
    let mut value = "1 1".to_owned();
    value.push_str(&" ".repeat(MAX_SOURCE_EDIT_SINGLE_EXPRESSION_BYTES - value.len()));
    value
}
