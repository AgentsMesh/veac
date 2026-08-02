use std::collections::BTreeSet;

use super::{instance, kind, Parser};
use crate::program::diagnostic::Diagnostic;
use crate::program::model::{ComponentDecl, ExpressionBinding, ParameterDecl, SlotDecl};
use crate::program::token::TokenKind;

pub(super) fn parse(parser: &mut Parser<'_>, exported: bool) -> Result<ComponentDecl, Diagnostic> {
    let start = parser.expect_word("component")?;
    parser.expect_word("sequence")?;
    let (name, _) = parser.identifier("component name")?;
    parser.expect(TokenKind::LeftBrace, "`{`")?;
    let mut parameters = Vec::new();
    let mut slots = Vec::new();
    let mut instances = Vec::new();
    while !parser.at_word("body") {
        if parser.at_word("param") {
            parameters.push(parameter(parser)?);
        } else if parser.at_word("slot") {
            slots.push(slot(parser)?);
        } else if parser.at_word("instance") {
            instances.push(instance::local(parser)?);
        } else {
            return Err(parser.error(
                "PROGRAM_COMPONENT_MEMBER",
                "component members must be param, slot, instance, or body",
                parser.current().span,
            ));
        }
    }
    parser.expect_word("body")?;
    let body = parser.raw_block()?;
    let end = parser.expect(TokenKind::RightBrace, "`}`")?;
    unique_members(parser, &parameters, &slots, &instances)?;
    Ok(ComponentDecl {
        name,
        parameters,
        slots,
        instances,
        body,
        exported,
        span: start.join(end),
    })
}

fn parameter(parser: &mut Parser<'_>) -> Result<ParameterDecl, Diagnostic> {
    let start = parser.expect_word("param")?;
    let value_type = kind::value(parser)?;
    let (name, _) = parser.identifier("parameter name")?;
    let (default, end) = if parser.at_word("default") {
        parser.advance();
        let (source, span) = parser.expression_until_semicolon()?;
        (Some(ExpressionBinding { source, span }), span)
    } else {
        (None, parser.expect(TokenKind::Semicolon, "`;`")?)
    };
    Ok(ParameterDecl {
        name,
        value_type,
        default,
        span: start.join(end),
    })
}

fn slot(parser: &mut Parser<'_>) -> Result<SlotDecl, Diagnostic> {
    let start = parser.expect_word("slot")?;
    let kind = kind::slot(parser)?;
    let (name, _) = parser.identifier("slot name")?;
    let end = parser.expect(TokenKind::Semicolon, "`;`")?;
    Ok(SlotDecl {
        name,
        kind,
        span: start.join(end),
    })
}

fn unique_members(
    parser: &Parser<'_>,
    parameters: &[ParameterDecl],
    slots: &[SlotDecl],
    instances: &[crate::program::model::InstanceDecl],
) -> Result<(), Diagnostic> {
    let mut names = BTreeSet::new();
    for parameter in parameters {
        if !names.insert(&parameter.name) {
            return Err(parser.error(
                "PROGRAM_DUPLICATE_PARAMETER",
                format!("parameter `{}` is declared more than once", parameter.name),
                parameter.span,
            ));
        }
    }
    names.clear();
    for slot in slots {
        if !names.insert(&slot.name) {
            return Err(parser.error(
                "PROGRAM_DUPLICATE_SLOT",
                format!("slot `{}` is declared more than once", slot.name),
                slot.span,
            ));
        }
    }
    names.clear();
    for instance in instances {
        if !names.insert(&instance.id) {
            return Err(parser.error(
                "PROGRAM_DUPLICATE_LOCAL_INSTANCE",
                format!(
                    "local instance `@{}` is declared more than once",
                    instance.id
                ),
                instance.span,
            ));
        }
    }
    Ok(())
}
