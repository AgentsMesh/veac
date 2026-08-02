use super::Parser;
use crate::program::expression::ast::{Expression, ExpressionKind};
use crate::program::expression::lexer::TokenKind;
use crate::program::expression::{ExactNumber, ExpressionError, Value};

impl Parser {
    pub(super) fn primary(&mut self) -> Result<Expression, ExpressionError> {
        let token = self.advance();
        match token.kind {
            TokenKind::Number { number, unit } => {
                let value = number_value(&number, &unit, token.span.clone())?;
                self.node(ExpressionKind::Literal(value), token.span)
            }
            TokenKind::Text(value) => {
                self.node(ExpressionKind::Literal(Value::Text(value)), token.span)
            }
            TokenKind::Color(value) => {
                self.node(ExpressionKind::Literal(Value::Color(value)), token.span)
            }
            TokenKind::Bool(value) => {
                self.node(ExpressionKind::Literal(Value::Bool(value)), token.span)
            }
            TokenKind::Symbol(value) if self.at(&TokenKind::LeftParen) => {
                self.call(value, token.span.start)
            }
            TokenKind::Symbol(value) => self.node(ExpressionKind::Symbol(value), token.span),
            TokenKind::LeftParen => self.parenthesized(token.span),
            _ => Err(ExpressionError::new(
                "EXPRESSION_EXPECTED_VALUE",
                "expected a literal, symbol, function call, or parenthesized expression",
                token.span,
            )),
        }
    }

    fn parenthesized(
        &mut self,
        opening: std::ops::Range<usize>,
    ) -> Result<Expression, ExpressionError> {
        self.enter_depth(opening)?;
        let expression = self.expression()?;
        self.depth -= 1;
        if self.take(&TokenKind::RightParen).is_none() {
            return Err(self.error("EXPRESSION_EXPECTED_TOKEN", "expected `)`"));
        }
        Ok(expression)
    }

    fn call(&mut self, function: String, start: usize) -> Result<Expression, ExpressionError> {
        let opening = self
            .take(&TokenKind::LeftParen)
            .expect("call starts at a left parenthesis");
        self.enter_depth(opening)?;
        let mut arguments = Vec::new();
        if !self.at(&TokenKind::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if self.take(&TokenKind::Comma).is_none() {
                    break;
                }
            }
        }
        let closing = self
            .take(&TokenKind::RightParen)
            .ok_or_else(|| self.error("EXPRESSION_EXPECTED_TOKEN", "expected `)`"))?;
        self.depth -= 1;
        self.node(
            ExpressionKind::Call {
                function,
                arguments,
            },
            start..closing.end,
        )
    }
}

fn number_value(
    raw: &str,
    unit: &str,
    span: std::ops::Range<usize>,
) -> Result<Value, ExpressionError> {
    let number = decimal(raw).ok_or_else(|| {
        ExpressionError::new(
            "EXPRESSION_NUMBER_LITERAL",
            format!("numeric literal `{raw}{unit}` is not representable"),
            span.clone(),
        )
    })?;
    let value = match unit {
        "" => Value::Scalar(number),
        "s" => Value::Time(number),
        "ms" => Value::Time(divide(number, 1_000, &span)?),
        "us" => Value::Time(divide(number, 1_000_000, &span)?),
        "px" => Value::Length(number),
        "%" => Value::Percent(number),
        "deg" => Value::Angle(number),
        _ => {
            return Err(ExpressionError::new(
                "EXPRESSION_UNIT",
                format!("unsupported unit `{unit}`"),
                span,
            ))
        }
    };
    Ok(value)
}

fn decimal(raw: &str) -> Option<ExactNumber> {
    let (whole, fraction) = raw.split_once('.').unwrap_or((raw, ""));
    if whole.is_empty() && fraction.is_empty()
        || !whole.bytes().all(|value| value.is_ascii_digit())
        || !fraction.bytes().all(|value| value.is_ascii_digit())
    {
        return None;
    }
    let whole = if whole.is_empty() { "0" } else { whole };
    let digits = format!("{whole}{fraction}");
    let numerator = digits.parse::<i128>().ok()?;
    let denominator = 10_i128.checked_pow(u32::try_from(fraction.len()).ok()?)?;
    ExactNumber::new(numerator, denominator)
}

fn divide(
    value: ExactNumber,
    denominator: i128,
    span: &std::ops::Range<usize>,
) -> Result<ExactNumber, ExpressionError> {
    value
        .checked_div(ExactNumber::integer(denominator))
        .ok_or_else(|| {
            ExpressionError::new(
                "EXPRESSION_OVERFLOW",
                "numeric literal exceeds the exact arithmetic range",
                span.clone(),
            )
        })
}
