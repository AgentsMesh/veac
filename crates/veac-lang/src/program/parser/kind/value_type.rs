use std::sync::Arc;

use super::super::Parser;
use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;
use crate::program::expression::{
    FunctionEffect, PrimitiveType, MAX_VALUE_TYPE_ARITY, MAX_VALUE_TYPE_DEPTH,
};
use crate::program::token::TokenKind;
use crate::program::{TypeSyntax, TypeSyntaxKind};
use crate::vocabulary::control_uses::{expression, structural_type as controls};

pub(super) fn value(parser: &mut Parser<'_>) -> Result<TypeSyntax, Diagnostic> {
    parse(parser, 1)
}

pub(super) fn value_with_span(parser: &mut Parser<'_>) -> Result<(TypeSyntax, Span), Diagnostic> {
    let value = value(parser)?;
    let span = value.span();
    Ok((value, span))
}

fn parse(parser: &mut Parser<'_>, depth: usize) -> Result<TypeSyntax, Diagnostic> {
    if depth > MAX_VALUE_TYPE_DEPTH {
        return Err(parser.error(
            "PROGRAM_TYPE_DEPTH",
            format!("type exceeds the {MAX_VALUE_TYPE_DEPTH} level depth limit"),
            parser.current().span,
        ));
    }
    if parser.at(&TokenKind::LeftParen) {
        return sequence(parser, depth, false);
    }
    if parser.at_control(controls::LIST_TYPE_CONSTRUCTOR) {
        return unary(parser, depth, TypeSyntaxKind::List);
    }
    if parser.at_control(controls::RANGE_TYPE_CONSTRUCTOR) {
        return unary(parser, depth, TypeSyntaxKind::Range);
    }
    if parser.at_control(controls::MAP_TYPE_CONSTRUCTOR) {
        return map(parser, depth);
    }
    if parser.at_control(controls::FUNCTION_TYPE_CONSTRUCTOR) {
        return sequence(parser, depth, true);
    }
    named(parser)
}

fn named(parser: &mut Parser<'_>) -> Result<TypeSyntax, Diagnostic> {
    let (name, span) = parser.word("value type")?;
    let kind = if let Some(value) = PrimitiveType::parse(&name) {
        TypeSyntaxKind::Primitive(value)
    } else if crate::name::is_qualified_name(&name) {
        TypeSyntaxKind::Named(name.into())
    } else {
        return Err(parser.error(
            "PROGRAM_TYPE_NAME",
            "nominal type must be a valid qualified name",
            span,
        ));
    };
    Ok(TypeSyntax::new(kind, span))
}

fn unary(
    parser: &mut Parser<'_>,
    depth: usize,
    wrap: impl FnOnce(Arc<TypeSyntax>) -> TypeSyntaxKind,
) -> Result<TypeSyntax, Diagnostic> {
    let start = parser.advance().span;
    parser.expect(TokenKind::Less, "`<`")?;
    let element = Arc::new(parse(parser, depth + 1)?);
    let end = parser.expect(TokenKind::Greater, "`>`")?;
    Ok(TypeSyntax::new(wrap(element), start.join(end)))
}

fn map(parser: &mut Parser<'_>, depth: usize) -> Result<TypeSyntax, Diagnostic> {
    let start = parser.advance().span;
    parser.expect(TokenKind::Less, "`<`")?;
    let key = Arc::new(parse(parser, depth + 1)?);
    parser.expect(TokenKind::Comma, "`,`")?;
    let value = Arc::new(parse(parser, depth + 1)?);
    let end = parser.expect(TokenKind::Greater, "`>`")?;
    Ok(TypeSyntax::new(
        TypeSyntaxKind::Map { key, value },
        start.join(end),
    ))
}

fn sequence(
    parser: &mut Parser<'_>,
    depth: usize,
    function: bool,
) -> Result<TypeSyntax, Diagnostic> {
    let start = parser.advance().span;
    if function {
        parser.expect(TokenKind::LeftParen, "`(`")?;
    }
    let mut values = Vec::new();
    while !parser.at(&TokenKind::RightParen) {
        values.push(parse(parser, depth + 1)?);
        check_arity(parser, values.len())?;
        if !parser.at(&TokenKind::Comma) {
            break;
        }
        parser.advance();
    }
    let right = parser.expect(TokenKind::RightParen, "`)`")?;
    if !function {
        return Ok(TypeSyntax::new(
            TypeSyntaxKind::Tuple(values.into()),
            start.join(right),
        ));
    }
    parser.expect(TokenKind::Arrow, "`->`")?;
    let result = Arc::new(parse(parser, depth + 1)?);
    let (effect_word, effect_span) = parser.word("`effect`")?;
    if !expression::FUNCTION_EFFECT.matches(&effect_word) {
        return Err(parser.error("PROGRAM_FUNCTION_EFFECT", "expected `effect`", effect_span));
    }
    let (effect, effect_span) = parser.word("function effect contract")?;
    let effect = FunctionEffect::parse(&effect).ok_or_else(|| {
        parser.error(
            "PROGRAM_FUNCTION_EFFECT",
            "function effect must be `pure`, `local`, `emit`, or `any`",
            effect_span,
        )
    })?;
    let span = start.join(effect_span);
    Ok(TypeSyntax::new(
        TypeSyntaxKind::Function {
            parameters: values.into(),
            result,
            effect,
        },
        span,
    ))
}

fn check_arity(parser: &Parser<'_>, actual: usize) -> Result<(), Diagnostic> {
    if actual <= MAX_VALUE_TYPE_ARITY {
        Ok(())
    } else {
        Err(parser.error(
            "PROGRAM_TYPE_ARITY",
            format!("type exceeds the {MAX_VALUE_TYPE_ARITY} element arity limit"),
            parser.current().span,
        ))
    }
}
