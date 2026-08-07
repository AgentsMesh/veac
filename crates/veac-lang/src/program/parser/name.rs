use super::Parser;
use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;

impl Parser<'_> {
    pub(super) fn identifier(&mut self, context: &str) -> Result<(String, Span), Diagnostic> {
        let (value, span) = self.word(context)?;
        if crate::name::is_name(&value) {
            Ok((value, span))
        } else {
            Err(self.error(
                "PROGRAM_IDENTIFIER",
                format!("{context} must be {}", crate::name::NAME_CONTRACT),
                span,
            ))
        }
    }
}
