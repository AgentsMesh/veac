use std::{ops::Range, sync::Arc};

use super::Parser;
use crate::authoring::Span;
use crate::program::expression::ast::TypeAnnotation;
use crate::program::expression::lexer::TokenKind;
use crate::program::expression::{
    ExpressionError, FunctionEffect, PrimitiveType, TypeConstructor, MAX_VALUE_TYPE_ARITY,
    MAX_VALUE_TYPE_DEPTH,
};
use crate::program::{TypeSyntax, TypeSyntaxKind};

mod effect;

impl Parser {
    pub(super) fn type_annotation(&mut self) -> Result<TypeAnnotation, ExpressionError> {
        let start = self.current().span.start;
        let syntax = self.value_type(1)?;
        Ok(TypeAnnotation {
            syntax,
            span: start..self.tokens[self.cursor.saturating_sub(1)].span.end,
        })
    }

    fn value_type(&mut self, depth: usize) -> Result<TypeSyntax, ExpressionError> {
        if depth > MAX_VALUE_TYPE_DEPTH {
            return Err(self.type_error(
                format!("type exceeds the {MAX_VALUE_TYPE_DEPTH} level depth limit"),
                self.current().span.clone(),
            ));
        }
        if self.take(&TokenKind::LeftParen).is_some() {
            return self.tuple_type(depth);
        }
        let token = self.advance();
        let name = match token.kind {
            TokenKind::Symbol(name) => name,
            TokenKind::Fn => TypeConstructor::Function.as_str().to_owned(),
            _ => return Err(self.type_error("expected a value type", token.span)),
        };
        let span = token.span.clone();
        if let Some(primitive) = PrimitiveType::parse(&name) {
            return Ok(TypeSyntax::new(
                TypeSyntaxKind::Primitive(primitive),
                authored(span),
            ));
        }
        match TypeConstructor::parse(&name) {
            Some(TypeConstructor::List) => self.list_type(depth),
            Some(TypeConstructor::Range) => self.range_type(depth),
            Some(TypeConstructor::Map) => self.map_type(depth),
            Some(TypeConstructor::Function) => self.function_type(depth),
            None => self.named_type(name, span),
        }
    }

    fn list_type(&mut self, depth: usize) -> Result<TypeSyntax, ExpressionError> {
        let start = self.tokens[self.cursor - 1].span.start;
        self.expect(&TokenKind::Less, "<")?;
        let element = Arc::new(self.value_type(depth + 1)?);
        let closing = self.expect(&TokenKind::Greater, ">")?;
        Ok(TypeSyntax::new(
            TypeSyntaxKind::List(element),
            authored(start..closing.end),
        ))
    }

    fn range_type(&mut self, depth: usize) -> Result<TypeSyntax, ExpressionError> {
        let start = self.tokens[self.cursor - 1].span.start;
        self.expect(&TokenKind::Less, "<")?;
        let element = Arc::new(self.value_type(depth + 1)?);
        let closing = self.expect(&TokenKind::Greater, ">")?;
        Ok(TypeSyntax::new(
            TypeSyntaxKind::Range(element),
            authored(start..closing.end),
        ))
    }

    fn map_type(&mut self, depth: usize) -> Result<TypeSyntax, ExpressionError> {
        let start = self.tokens[self.cursor - 1].span.start;
        self.expect(&TokenKind::Less, "<")?;
        let key = Arc::new(self.value_type(depth + 1)?);
        self.expect(&TokenKind::Comma, ",")?;
        let value = Arc::new(self.value_type(depth + 1)?);
        let closing = self.expect(&TokenKind::Greater, ">")?;
        Ok(TypeSyntax::new(
            TypeSyntaxKind::Map { key, value },
            authored(start..closing.end),
        ))
    }

    fn tuple_type(&mut self, depth: usize) -> Result<TypeSyntax, ExpressionError> {
        let start = self.tokens[self.cursor - 1].span.start;
        let values = self.type_list(depth, &TokenKind::RightParen)?;
        let closing = self.expect(&TokenKind::RightParen, ")")?;
        Ok(TypeSyntax::new(
            TypeSyntaxKind::Tuple(values.into()),
            authored(start..closing.end),
        ))
    }

    fn function_type(&mut self, depth: usize) -> Result<TypeSyntax, ExpressionError> {
        let start = self.tokens[self.cursor - 1].span.start;
        self.expect(&TokenKind::LeftParen, "(")?;
        let parameters = self.type_list(depth, &TokenKind::RightParen)?;
        self.expect(&TokenKind::RightParen, ")")?;
        self.expect(&TokenKind::Arrow, "->")?;
        let result = Arc::new(self.value_type(depth + 1)?);
        let (effect, end) = self.function_effect()?;
        Ok(TypeSyntax::new(
            TypeSyntaxKind::Function {
                parameters: parameters.into(),
                result,
                effect,
            },
            Span { start, end },
        ))
    }

    pub(super) fn function_effect(&mut self) -> Result<(FunctionEffect, usize), ExpressionError> {
        effect::parse(self)
    }

    fn type_list(
        &mut self,
        depth: usize,
        closing: &TokenKind,
    ) -> Result<Vec<TypeSyntax>, ExpressionError> {
        let mut values = Vec::new();
        while !self.at(closing) {
            values.push(self.value_type(depth + 1)?);
            if values.len() > MAX_VALUE_TYPE_ARITY {
                return Err(self.type_error(
                    format!("type exceeds the {MAX_VALUE_TYPE_ARITY} element arity limit"),
                    self.current().span.clone(),
                ));
            }
            if self.take(&TokenKind::Comma).is_none() {
                break;
            }
            if self.at(closing) {
                return Err(self.type_error(
                    "trailing type separators are not supported",
                    self.current().span.clone(),
                ));
            }
        }
        Ok(values)
    }

    fn type_error(&self, message: impl Into<String>, span: Range<usize>) -> ExpressionError {
        ExpressionError::new("EXPRESSION_TYPE_SYNTAX", message, span)
    }

    fn named_type(
        &mut self,
        first: String,
        start: Range<usize>,
    ) -> Result<TypeSyntax, ExpressionError> {
        let mut name = first;
        let mut end = start.end;
        while self.take(&TokenKind::Dot).is_some() {
            let token = self.advance();
            let TokenKind::Symbol(segment) = token.kind else {
                return Err(self.type_error("expected a nominal type name segment", token.span));
            };
            name.push('.');
            name.push_str(&segment);
            end = token.span.end;
        }
        if !crate::name::is_qualified_name(&name) {
            return Err(self.type_error("invalid nominal type name", start.start..end));
        }
        Ok(TypeSyntax::new(
            TypeSyntaxKind::Named(name.into()),
            Span {
                start: start.start,
                end,
            },
        ))
    }
}

fn authored(span: Range<usize>) -> Span {
    Span {
        start: span.start,
        end: span.end,
    }
}
