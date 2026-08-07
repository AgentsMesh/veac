use super::{ast, lexer, parser, ExpressionError};

pub(crate) fn validate_function_body(source: &str) -> Result<(), ExpressionError> {
    let expression = parser::parse(lexer::lex(source)?)?;
    matches!(&expression.kind, ast::ExpressionKind::Block(_))
        .then_some(())
        .ok_or_else(|| {
            ExpressionError::new(
                "EXPRESSION_FUNCTION_BODY",
                "function body must be a block expression",
                expression.span,
            )
        })
}

pub(crate) fn validate_function_statement(source: &str) -> Result<(), ExpressionError> {
    parser::parse_statement(lexer::lex(source)?).map(drop)
}

pub(crate) fn validate_temporal_attachment(source: &str) -> Result<(), ExpressionError> {
    let expression = parser::parse(lexer::lex(source)?)?;
    matches!(&expression.kind, ast::ExpressionKind::TemporalAttach(_))
        .then_some(())
        .ok_or_else(|| {
            ExpressionError::new(
                "EXPRESSION_TEMPORAL_ATTACHMENT",
                "component animation declaration must be an `animate` expression",
                expression.span,
            )
        })
}
