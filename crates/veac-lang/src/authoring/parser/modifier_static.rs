use super::parameter::{point as parse_point, vector as parse_vector};
use super::Parser;

pub(super) fn vector(
    parser: &mut Parser,
    entry: &crate::authoring::SemanticEntry,
) -> Option<crate::authoring::VectorDecl> {
    match parse_vector(parser, entry)? {
        crate::authoring::ParameterDecl::Constant(value) => Some(value),
        crate::authoring::ParameterDecl::Curve { .. } => {
            parser.error(
                "AUTHORING_STATIC_VALUE",
                "this vector cannot be animated".to_owned(),
                entry.span,
            );
            None
        }
    }
}

pub(super) fn point(
    parser: &mut Parser,
    entry: &crate::authoring::SemanticEntry,
) -> Option<crate::authoring::PointDecl> {
    match parse_point(parser, entry)? {
        crate::authoring::ParameterDecl::Constant(value) => Some(value),
        crate::authoring::ParameterDecl::Curve { .. } => {
            parser.error(
                "AUTHORING_STATIC_VALUE",
                "this point cannot be animated".to_owned(),
                entry.span,
            );
            None
        }
    }
}
