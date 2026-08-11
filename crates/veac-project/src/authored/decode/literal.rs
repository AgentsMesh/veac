use veac_lang::program::expression::Value;

use crate::{ProjectLiteral, ProjectRational};

use super::super::ProjectDecodeError;
use super::value::{unknown_variant, Decoder};

pub(super) fn literal(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<ProjectLiteral, ProjectDecodeError> {
    let variant = decoder.variant(value, "ProjectLiteral", path)?;
    let field = || variant.fields.get("value");
    let field_path = || variant.fields.path("value");
    match variant.name {
        "Bool" => Ok(ProjectLiteral::Bool {
            value: decoder.bool(field()?, &field_path())?,
        }),
        "Integer" => Ok(ProjectLiteral::Integer {
            value: decoder.integer(field()?, &field_path())?,
        }),
        "Scalar" => Ok(ProjectLiteral::Scalar {
            value: rational(decoder, field()?, &field_path())?,
        }),
        "Text" => Ok(ProjectLiteral::Text {
            value: decoder.text(field()?, &field_path())?,
        }),
        "Identifier" => Ok(ProjectLiteral::Identifier {
            value: decoder.identifier(field()?, &field_path())?,
        }),
        "Duration" => Ok(ProjectLiteral::Duration {
            value: rational(decoder, field()?, &field_path())?,
        }),
        name => Err(unknown_variant(path, "ProjectLiteral", name)),
    }
}

pub(super) fn rational(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<ProjectRational, ProjectDecodeError> {
    let fields = decoder.structure(value, "ProjectRational", path)?;
    Ok(ProjectRational::new(
        decoder.integer(fields.get("numerator")?, &fields.path("numerator"))?,
        decoder.u64(fields.get("denominator")?, &fields.path("denominator"))?,
    ))
}
