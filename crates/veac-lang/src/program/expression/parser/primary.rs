use super::Parser;
use crate::program::expression::ast::{Expression, ExpressionKind};
use crate::program::expression::lexer::TokenKind;
use crate::program::expression::{ExactNumber, ExpressionError, UnitSuffix, Value};

impl Parser {
    pub(super) fn signed_min_literal(
        &mut self,
        sign: std::ops::Range<usize>,
    ) -> Result<Option<Expression>, ExpressionError> {
        let is_minimum = matches!(
            &self.current().kind,
            TokenKind::Number { number, unit }
                if number == "9223372036854775808" && unit.is_empty()
        );
        if !is_minimum {
            return Ok(None);
        }
        let number = self.advance();
        self.node(
            ExpressionKind::Literal(Value::Integer(i64::MIN)),
            sign.start..number.span.end,
        )
        .map(Some)
    }

    pub(super) fn primary(&mut self) -> Result<Expression, ExpressionError> {
        let token = self.advance();
        match token.kind {
            TokenKind::Number { number, unit } => {
                let value = number_value(&number, &unit, token.span.clone())?;
                self.node(ExpressionKind::Literal(value), token.span)
            }
            TokenKind::Text(value) => self.node(
                ExpressionKind::Literal(Value::Text(value.into())),
                token.span,
            ),
            TokenKind::Color(value) => self.node(
                ExpressionKind::Literal(Value::Color(value.into())),
                token.span,
            ),
            TokenKind::Bool(value) => {
                self.node(ExpressionKind::Literal(Value::Bool(value)), token.span)
            }
            TokenKind::Symbol(value) => self.node(ExpressionKind::Symbol(value), token.span),
            TokenKind::LeftParen => self.parenthesized(token.span),
            TokenKind::LeftBracket => self.list(token.span),
            TokenKind::MapStart => self.map(token.span),
            TokenKind::LeftBrace => {
                let (block, span) = self.block(token.span)?;
                self.node(ExpressionKind::Block(block), span)
            }
            TokenKind::If => self.conditional(token.span),
            TokenKind::Fn => self.closure(token.span),
            TokenKind::For => self.iteration(token.span),
            TokenKind::Match => self.match_expression(token.span),
            TokenKind::Animate => self.temporal_attachment(token.span),
            _ => Err(ExpressionError::new(
                "EXPRESSION_EXPECTED_VALUE",
                "expected a value, block, conditional, closure, or function call",
                token.span,
            )),
        }
    }

    fn parenthesized(
        &mut self,
        opening: std::ops::Range<usize>,
    ) -> Result<Expression, ExpressionError> {
        self.enter_depth(opening.clone())?;
        let expression = self.parenthesized_contents(opening);
        self.depth -= 1;
        expression
    }
}

fn number_value(
    raw: &str,
    unit: &str,
    span: std::ops::Range<usize>,
) -> Result<Value, ExpressionError> {
    if unit.is_empty() && !raw.contains('.') {
        return raw.parse::<i64>().map(Value::Integer).map_err(|_| {
            ExpressionError::new(
                "EXPRESSION_NUMBER_LITERAL",
                format!("integer literal `{raw}` is outside the signed 64-bit range"),
                span,
            )
        });
    }
    let number = decimal(raw).ok_or_else(|| {
        ExpressionError::new(
            "EXPRESSION_NUMBER_LITERAL",
            format!("numeric literal `{raw}{unit}` is not representable"),
            span.clone(),
        )
    })?;
    let value = match UnitSuffix::parse(unit) {
        None if unit.is_empty() => Value::Scalar(number),
        Some(UnitSuffix::Seconds) => Value::Time(number),
        Some(UnitSuffix::Milliseconds) => Value::Time(divide(number, 1_000, &span)?),
        Some(UnitSuffix::Microseconds) => Value::Time(divide(number, 1_000_000, &span)?),
        Some(UnitSuffix::Pixels) => Value::Length(number),
        Some(UnitSuffix::Percent) => Value::Percent(number),
        Some(UnitSuffix::Degrees) => Value::Angle(number),
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
    let fraction = fraction.trim_end_matches('0');
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
