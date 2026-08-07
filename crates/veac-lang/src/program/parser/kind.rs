use super::Parser;
use crate::program::diagnostic::Diagnostic;

mod value_type;
use crate::authoring::Span;
use crate::program::TypeSyntax;

pub(super) fn value(parser: &mut Parser<'_>) -> Result<TypeSyntax, Diagnostic> {
    value_type::value(parser)
}

pub(super) fn value_with_span(parser: &mut Parser<'_>) -> Result<(TypeSyntax, Span), Diagnostic> {
    value_type::value_with_span(parser)
}
