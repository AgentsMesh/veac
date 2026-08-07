use super::{kind, Parser};
use crate::program::diagnostic::Diagnostic;
use crate::program::lexer;
use crate::program::model::{
    EnumDecl, EnumVariantDecl, StructDecl, TypeDecl, TypeDeclKind, TypeFieldDecl,
};
use crate::program::token::TokenKind;
use crate::program::MAX_TYPE_MEMBERS;
use crate::vocabulary::control_uses::static_program as controls;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TypeDeclarationFragmentKind {
    Struct,
    StructField,
    Enum,
    EnumVariant,
    EnumVariantField,
}

pub(super) fn structure(parser: &mut Parser<'_>, exported: bool) -> Result<TypeDecl, Diagnostic> {
    let start = parser.expect_control(controls::STRUCT_DECLARATION)?;
    let (name, name_span) = parser.identifier("struct name")?;
    let (fields, end) = fields(parser)?;
    Ok(TypeDecl {
        name,
        name_span,
        kind: TypeDeclKind::Struct(StructDecl { fields }),
        exported,
        span: start.join(end),
    })
}

pub(super) fn enumeration(parser: &mut Parser<'_>, exported: bool) -> Result<TypeDecl, Diagnostic> {
    let start = parser.expect_control(controls::ENUM_DECLARATION)?;
    let (name, name_span) = parser.identifier("enum name")?;
    parser.expect(TokenKind::LeftBrace, "`{`")?;
    let mut variants = Vec::new();
    while !parser.at(&TokenKind::RightBrace) {
        if variants.len() >= MAX_TYPE_MEMBERS {
            return Err(limit(parser, "enum variant"));
        }
        variants.push(variant(parser)?);
        if !parser.at(&TokenKind::Comma) {
            break;
        }
        parser.advance();
    }
    let end = parser.expect(TokenKind::RightBrace, "`}`")?;
    Ok(TypeDecl {
        name,
        name_span,
        kind: TypeDeclKind::Enum(EnumDecl { variants }),
        exported,
        span: start.join(end),
    })
}

fn variant(parser: &mut Parser<'_>) -> Result<EnumVariantDecl, Diagnostic> {
    let (name, name_span) = parser.identifier("enum variant name")?;
    if !parser.at(&TokenKind::LeftBrace) {
        return Ok(EnumVariantDecl {
            name,
            name_span,
            fields: Vec::new(),
            span: name_span,
        });
    }
    let (fields, end) = fields(parser)?;
    Ok(EnumVariantDecl {
        name,
        name_span,
        fields,
        span: name_span.join(end),
    })
}

fn fields(
    parser: &mut Parser<'_>,
) -> Result<(Vec<TypeFieldDecl>, crate::authoring::Span), Diagnostic> {
    parser.expect(TokenKind::LeftBrace, "`{`")?;
    let mut fields = Vec::new();
    while !parser.at(&TokenKind::RightBrace) {
        if fields.len() >= MAX_TYPE_MEMBERS {
            return Err(limit(parser, "field"));
        }
        fields.push(field(parser)?);
        if !parser.at(&TokenKind::Comma) {
            break;
        }
        parser.advance();
    }
    let end = parser.expect(TokenKind::RightBrace, "`}`")?;
    Ok((fields, end))
}

fn field(parser: &mut Parser<'_>) -> Result<TypeFieldDecl, Diagnostic> {
    let (name, name_span) = parser.identifier("field name")?;
    parser.expect(TokenKind::Colon, "`:`")?;
    let type_syntax = kind::value(parser)?;
    Ok(TypeFieldDecl {
        name,
        name_span,
        span: name_span.join(type_syntax.span()),
        type_syntax,
    })
}

fn limit(parser: &Parser<'_>, kind: &str) -> Diagnostic {
    parser.error(
        "PROGRAM_TYPE_MEMBER_LIMIT",
        format!("{kind} count exceeds {MAX_TYPE_MEMBERS}"),
        parser.current().span,
    )
}

pub(crate) fn validate_fragment(
    source: &str,
    kind: TypeDeclarationFragmentKind,
) -> Result<(), String> {
    const PATH: &str = "<source-edit-declaration>";
    let tokens = lexer::lex(PATH, source).map_err(|errors| errors[0].message.clone())?;
    let mut parser = Parser::new(PATH, source, tokens);
    let parsed = match kind {
        TypeDeclarationFragmentKind::Struct => structure(&mut parser, false).map(|_| ()),
        TypeDeclarationFragmentKind::StructField
        | TypeDeclarationFragmentKind::EnumVariantField => field(&mut parser).map(|_| ()),
        TypeDeclarationFragmentKind::Enum => enumeration(&mut parser, false).map(|_| ()),
        TypeDeclarationFragmentKind::EnumVariant => variant(&mut parser).map(|_| ()),
    };
    parsed.map_err(|error| error.message)?;
    parser
        .at_eof()
        .then_some(())
        .ok_or_else(|| "replacement must contain exactly one nominal declaration".into())
}

#[cfg(test)]
#[path = "type_declaration/tests.rs"]
mod tests;
