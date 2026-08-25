use std::ops::Range;

use super::{
    FunctionEffect, PrimitiveType, TypeConstructor, ValueType, ValueTypeError,
    MAX_VALUE_TYPE_ARITY, MAX_VALUE_TYPE_DEPTH,
};
use crate::DomainType;

mod cursor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueTypeParseError {
    code: &'static str,
    message: String,
    span: Range<usize>,
}

impl ValueType {
    pub fn parse(source: &str) -> Result<Self, ValueTypeParseError> {
        Parser::new(source).parse()
    }
}

impl ValueTypeParseError {
    pub fn code(&self) -> &'static str {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }

    fn syntax(message: impl Into<String>, span: Range<usize>) -> Self {
        Self {
            code: "VALUE_TYPE_PARSE",
            message: message.into(),
            span,
        }
    }

    fn construction(error: ValueTypeError, span: Range<usize>) -> Self {
        Self {
            code: error.code(),
            message: error.message().to_owned(),
            span,
        }
    }
}

pub(super) struct Parser<'a> {
    pub(super) source: &'a str,
    pub(super) cursor: usize,
    pub(super) depth: usize,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            cursor: 0,
            depth: 0,
        }
    }

    fn parse(mut self) -> Result<ValueType, ValueTypeParseError> {
        let (value, _) = self.value()?;
        self.whitespace();
        if self.cursor != self.source.len() {
            return Err(self.error("unexpected trailing type syntax"));
        }
        Ok(value)
    }

    fn value(&mut self) -> Result<(ValueType, Range<usize>), ValueTypeParseError> {
        self.whitespace();
        let start = self.cursor;
        self.depth += 1;
        if self.depth > MAX_VALUE_TYPE_DEPTH {
            return Err(ValueTypeParseError {
                code: "VALUE_TYPE_DEPTH",
                message: format!("type exceeds the {MAX_VALUE_TYPE_DEPTH} level depth limit"),
                span: start..start,
            });
        }
        let value = if self.take('(') {
            self.tuple(start)?
        } else {
            let word = self.word()?;
            if let Some(primitive) = PrimitiveType::parse(word) {
                ValueType::primitive(primitive)
            } else if let Some(domain) = DomainType::parse(word) {
                ValueType::domain(domain)
            } else {
                match TypeConstructor::parse(word) {
                    Some(TypeConstructor::List) => self.list(start)?,
                    Some(TypeConstructor::Map) => self.map(start)?,
                    Some(TypeConstructor::Range) => self.range(start)?,
                    Some(TypeConstructor::Function) => self.function(start)?,
                    None => return Err(self.error(format!("unknown value type `{word}`"))),
                }
            }
        };
        self.depth -= 1;
        Ok((value, start..self.cursor))
    }

    fn list(&mut self, start: usize) -> Result<ValueType, ValueTypeParseError> {
        self.expect('<')?;
        let (element, _) = self.value()?;
        self.expect('>')?;
        ValueType::list(element).map_err(|error| self.construct(error, start))
    }

    fn range(&mut self, _start: usize) -> Result<ValueType, ValueTypeParseError> {
        self.expect('<')?;
        let (element, element_span) = self.value()?;
        self.expect('>')?;
        ValueType::range(element)
            .map_err(|error| ValueTypeParseError::construction(error, element_span))
    }

    fn map(&mut self, _start: usize) -> Result<ValueType, ValueTypeParseError> {
        self.expect('<')?;
        let (key, key_span) = self.value()?;
        self.expect(',')?;
        let (value, _) = self.value()?;
        self.expect('>')?;
        ValueType::map(key, value)
            .map_err(|error| ValueTypeParseError::construction(error, key_span))
    }

    fn tuple(&mut self, start: usize) -> Result<ValueType, ValueTypeParseError> {
        let mut elements = Vec::new();
        if !self.peek(')') {
            loop {
                elements.push(self.value()?.0);
                self.check_arity(elements.len(), start)?;
                if !self.take(',') {
                    break;
                }
            }
        }
        self.expect(')')?;
        ValueType::tuple(elements).map_err(|error| self.construct(error, start))
    }

    fn function(&mut self, start: usize) -> Result<ValueType, ValueTypeParseError> {
        self.expect('(')?;
        let mut parameters = Vec::new();
        if !self.peek(')') {
            loop {
                parameters.push(self.value()?.0);
                self.check_arity(parameters.len(), start)?;
                if !self.take(',') {
                    break;
                }
            }
        }
        self.expect(')')?;
        self.whitespace();
        if !self.source[self.cursor..].starts_with("->") {
            return Err(self.error("expected `->` in function type"));
        }
        self.cursor += 2;
        let result = self.value()?.0;
        let effect = self.effect()?;
        ValueType::function(parameters, result, effect)
            .map_err(|error| self.construct(error, start))
    }

    fn effect(&mut self) -> Result<FunctionEffect, ValueTypeParseError> {
        self.whitespace();
        let keyword = self.word()?;
        if keyword != "effect" {
            return Err(self.error("expected `effect` in function type"));
        }
        self.whitespace();
        let value = self.word()?;
        FunctionEffect::parse(value)
            .ok_or_else(|| self.error("function effect must be `pure`, `local`, `emit`, or `any`"))
    }
}
