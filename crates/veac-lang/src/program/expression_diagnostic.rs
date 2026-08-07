use crate::authoring::Span;

use super::diagnostic::Diagnostic;
use super::expression::ExpressionError;

/// Both the fallback call site and authored function origins use file-absolute spans.
pub(super) fn runtime(
    default_code: &'static str,
    fallback_path: &str,
    fallback_absolute_span: Span,
    error: ExpressionError,
) -> Diagnostic {
    let code = match error.code() {
        "EXPRESSION_EXECUTION_LIMIT" => "PROGRAM_EXECUTION_LIMIT",
        _ => default_code,
    };
    let authored = error
        .authored_origin()
        .zip(error.authored_span())
        .map(|(origin, span)| {
            (
                origin.source_id().to_owned(),
                Span {
                    start: span.start,
                    end: span.end,
                },
            )
        });
    let mut message = error.to_string();
    if authored.is_some() {
        message.push_str(&format!(
            "; called from {fallback_path}:{}..{}",
            fallback_absolute_span.start, fallback_absolute_span.end
        ));
    }
    if code == "PROGRAM_EXECUTION_LIMIT" {
        return Diagnostic::new(code, fallback_path, message, fallback_absolute_span);
    }
    let (path, span) =
        authored.unwrap_or_else(|| (fallback_path.to_owned(), fallback_absolute_span));
    Diagnostic::new(code, path, message, span)
}
