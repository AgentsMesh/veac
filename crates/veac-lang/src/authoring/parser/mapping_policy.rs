use crate::authoring::SourceOutOfRangeDecl;

use super::Parser;

pub(super) fn value(parser: &mut Parser) -> Option<SourceOutOfRangeDecl> {
    let value = parser.identifier("source out-of-range policy")?;
    match value.value.as_str() {
        "strict" => Some(SourceOutOfRangeDecl::Strict),
        "hold-first" => Some(SourceOutOfRangeDecl::HoldFirst),
        "hold-last" => Some(SourceOutOfRangeDecl::HoldLast),
        "hold-both" => Some(SourceOutOfRangeDecl::HoldBoth),
        _ => {
            parser.error(
                "AUTHORING_MAPPING_OUTSIDE",
                "mapping outside must be strict, hold-first, hold-last, or hold-both".to_owned(),
                value.span,
            );
            None
        }
    }
}
