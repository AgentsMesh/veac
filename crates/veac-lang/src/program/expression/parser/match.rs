use std::collections::BTreeSet;
use std::ops::Range;

use super::Parser;
use crate::program::expression::ast::{
    Expression, ExpressionKind, MatchArm, MatchPattern, PathSegment, PatternField,
};
use crate::program::expression::lexer::TokenKind;
use crate::program::expression::ExpressionError;
use crate::program::MAX_TYPE_MEMBERS;

impl Parser {
    pub(super) fn match_expression(
        &mut self,
        start: Range<usize>,
    ) -> Result<Expression, ExpressionError> {
        self.enter_depth(start.clone())?;
        let result = self.match_contents(start.start);
        self.depth -= 1;
        result
    }

    fn match_contents(&mut self, start: usize) -> Result<Expression, ExpressionError> {
        let previous = self.construct_floor.replace(self.depth);
        let scrutinee = self.expression();
        self.construct_floor = previous;
        let scrutinee = Box::new(scrutinee?);
        self.expect(&TokenKind::LeftBrace, "{")?;
        let mut arms = Vec::new();
        while !self.at(&TokenKind::RightBrace) {
            if arms.len() >= MAX_TYPE_MEMBERS {
                return Err(self.error(
                    "EXPRESSION_MATCH_ARM_LIMIT",
                    format!("match exceeds {MAX_TYPE_MEMBERS} arms"),
                ));
            }
            let arm = self.match_arm()?;
            let wildcard = matches!(arm.pattern, MatchPattern::Wildcard { .. });
            arms.push(arm);
            let comma = self.take(&TokenKind::Comma).is_some();
            if wildcard && !self.at(&TokenKind::RightBrace) {
                return Err(self.error(
                    "EXPRESSION_MATCH_WILDCARD_ORDER",
                    "wildcard must be the final match arm",
                ));
            }
            if !comma {
                break;
            }
        }
        if arms.is_empty() {
            return Err(self.error("EXPRESSION_MATCH_ARM", "match requires at least one arm"));
        }
        let closing = self.expect(&TokenKind::RightBrace, "}")?;
        self.node(
            ExpressionKind::Match { scrutinee, arms },
            start..closing.end,
        )
    }

    fn match_arm(&mut self) -> Result<MatchArm, ExpressionError> {
        let start = self.current().span.start;
        let pattern = self.match_pattern()?;
        self.expect(&TokenKind::FatArrow, "=>")?;
        let body = self.expression()?;
        Ok(MatchArm {
            pattern,
            span: start..body.span.end,
            body,
        })
    }

    fn match_pattern(&mut self) -> Result<MatchPattern, ExpressionError> {
        let first = self.pattern_segment()?;
        if first.name == "_" {
            return Ok(MatchPattern::Wildcard { span: first.span });
        }
        let mut path = vec![first];
        while self.take(&TokenKind::Dot).is_some() {
            path.push(self.pattern_segment()?);
        }
        if path.len() < 2 {
            return Err(ExpressionError::new(
                "EXPRESSION_MATCH_PATTERN",
                "enum patterns require a type and variant path",
                path[0].span.clone(),
            ));
        }
        let fields = if self.take(&TokenKind::LeftBrace).is_some() {
            self.pattern_fields()?
        } else {
            Vec::new()
        };
        Ok(MatchPattern::Variant { path, fields })
    }

    fn pattern_segment(&mut self) -> Result<PathSegment, ExpressionError> {
        let token = self.advance();
        let TokenKind::Symbol(name) = token.kind else {
            return Err(ExpressionError::new(
                "EXPRESSION_MATCH_PATTERN",
                "expected an enum variant path or `_`",
                token.span,
            ));
        };
        Ok(PathSegment {
            name,
            span: token.span,
        })
    }

    fn pattern_fields(&mut self) -> Result<Vec<PatternField>, ExpressionError> {
        let mut fields = Vec::new();
        let mut names = BTreeSet::new();
        let mut bindings = BTreeSet::new();
        while !self.at(&TokenKind::RightBrace) {
            if fields.len() >= MAX_TYPE_MEMBERS {
                return Err(self.error(
                    "EXPRESSION_MATCH_FIELD_LIMIT",
                    format!("match pattern exceeds {MAX_TYPE_MEMBERS} fields"),
                ));
            }
            let field = self.pattern_segment()?;
            let binding = if self.take(&TokenKind::Colon).is_some() {
                self.pattern_segment()?
            } else {
                field.clone()
            };
            if !names.insert(field.name.clone()) {
                return Err(duplicate("field", &field));
            }
            if !bindings.insert(binding.name.clone()) {
                return Err(duplicate("binding", &binding));
            }
            fields.push(PatternField {
                field: field.name,
                field_span: field.span,
                binding: binding.name,
                binding_span: binding.span,
            });
            if self.take(&TokenKind::Comma).is_none() {
                break;
            }
            if self.at(&TokenKind::RightBrace) {
                break;
            }
        }
        self.expect(&TokenKind::RightBrace, "}")?;
        Ok(fields)
    }
}

fn duplicate(kind: &str, value: &PathSegment) -> ExpressionError {
    ExpressionError::new(
        "EXPRESSION_DUPLICATE_PATTERN_NAME",
        format!("pattern {kind} `{}` is authored more than once", value.name),
        value.span.clone(),
    )
}

#[cfg(test)]
#[path = "match/tests.rs"]
mod tests;
