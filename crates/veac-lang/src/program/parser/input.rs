use super::{kind, Parser};
use crate::program::diagnostic::Diagnostic;
use crate::program::model::BuildInputDecl;
use crate::program::token::TokenKind;
use crate::program::BuildInputRole;
use crate::vocabulary::control_uses::static_program as controls;

pub(super) fn parse(parser: &mut Parser<'_>) -> Result<BuildInputDecl, Diagnostic> {
    let start = parser.expect_control(controls::INPUT_DECLARATION)?;
    let role = role(parser)?;
    let (name, name_span) = parser.identifier("input name")?;
    parser.expect(TokenKind::Colon, "`:`")?;
    let type_syntax = kind::value(parser)?;
    let end = parser.expect(TokenKind::Semicolon, "`;`")?;
    Ok(BuildInputDecl {
        role,
        name,
        name_span,
        type_syntax,
        span: start.join(end),
    })
}

fn role(parser: &mut Parser<'_>) -> Result<BuildInputRole, Diagnostic> {
    let value = if parser.at_control(controls::INPUT_PARAMETER_ROLE) {
        BuildInputRole::Parameter
    } else if parser.at_control(controls::INPUT_ASSET_METADATA_ROLE) {
        BuildInputRole::AssetMetadata
    } else if parser.at_control(controls::INPUT_ANALYSIS_ROLE) {
        BuildInputRole::Analysis
    } else if parser.at_control(controls::INPUT_MATERIAL_ROLE) {
        BuildInputRole::Material
    } else {
        return Err(parser.error(
            "PROGRAM_INPUT_ROLE",
            "input role must be parameter, asset_metadata, analysis, or material",
            parser.current().span,
        ));
    };
    parser.advance();
    Ok(value)
}
