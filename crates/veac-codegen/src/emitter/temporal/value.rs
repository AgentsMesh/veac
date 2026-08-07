use std::sync::Arc;

use veac_plan::canonical::{LengthUnit, TemporalValue};

use super::{budget::Budget, error::TemporalBackendError};
use crate::emitter::time;

pub(super) type Expression = Arc<str>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::emitter) enum LengthKind {
    Pixels,
    Relative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::emitter) struct LengthExpression {
    pub(in crate::emitter) value: Expression,
    pub(in crate::emitter) kind: LengthKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::emitter) struct TextChoice {
    pub(in crate::emitter) condition: Expression,
    pub(in crate::emitter) value: Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::emitter) enum CompiledValue {
    Boolean(Expression),
    Integer(Expression),
    Scalar(Expression),
    Time(Expression),
    Length(LengthExpression),
    Angle(Expression),
    Vec2(Expression, Expression),
    Point(LengthExpression, LengthExpression),
    Rect([Expression; 4]),
    Color([Expression; 4]),
    Text(Vec<TextChoice>),
}

impl CompiledValue {
    pub(super) fn literal(
        value: &TemporalValue,
        budget: &mut Budget<'_>,
    ) -> Result<Self, TemporalBackendError> {
        use TemporalValue::*;
        Ok(match value {
            Boolean { value } => Self::Boolean(number(budget, f64::from(u8::from(*value)))?),
            Integer { value } => Self::Integer(number(budget, *value as f64)?),
            Scalar { value } => Self::Scalar(number(budget, *value)?),
            Time { value } => Self::Time(number(
                budget,
                value.value as f64 / f64::from(value.timescale),
            )?),
            Length { value } => Self::Length(length(budget, *value)?),
            Angle { degrees } => Self::Angle(number(budget, *degrees)?),
            Vec2 { value } => Self::Vec2(number(budget, value.x)?, number(budget, value.y)?),
            Point { value } => Self::Point(length(budget, value.x)?, length(budget, value.y)?),
            Rect { value } => Self::Rect([
                number(budget, value.x)?,
                number(budget, value.y)?,
                number(budget, value.width)?,
                number(budget, value.height)?,
            ]),
            Color { value } => Self::Color([
                number(budget, f64::from(value.red))?,
                number(budget, f64::from(value.green))?,
                number(budget, f64::from(value.blue))?,
                number(budget, f64::from(value.alpha))?,
            ]),
            Text { value } => Self::Text(vec![TextChoice {
                condition: number(budget, 1.0)?,
                value: Arc::from(value.as_str()),
            }]),
        })
    }

    pub(super) fn numeric(&self) -> Option<&Expression> {
        match self {
            Self::Boolean(value)
            | Self::Integer(value)
            | Self::Scalar(value)
            | Self::Time(value)
            | Self::Angle(value) => Some(value),
            _ => None,
        }
    }

    pub(in crate::emitter) fn number_expression(&self) -> Option<String> {
        match self {
            Self::Scalar(value) | Self::Angle(value) => Some(value.to_string()),
            _ => None,
        }
    }

    pub(in crate::emitter) fn vector_expression(&self, index: usize) -> Option<String> {
        let Self::Vec2(x, y) = self else {
            return None;
        };
        Some(if index == 0 { x } else { y }.to_string())
    }

    pub(in crate::emitter) fn point_expression(
        &self,
        index: usize,
        extent: &str,
    ) -> Option<String> {
        let Self::Point(x, y) = self else {
            return None;
        };
        Some(if index == 0 { x } else { y }.pixels(extent))
    }

    pub(in crate::emitter) fn rect_expression(&self, index: usize) -> Option<String> {
        let Self::Rect(values) = self else {
            return None;
        };
        values.get(index).map(ToString::to_string)
    }
}

impl LengthExpression {
    fn pixels(&self, extent: &str) -> String {
        match self.kind {
            LengthKind::Pixels => self.value.to_string(),
            LengthKind::Relative => format!("({extent})*({})", self.value),
        }
    }
}

pub(super) fn number(
    budget: &mut Budget<'_>,
    value: f64,
) -> Result<Expression, TemporalBackendError> {
    budget.literal(time::number(value))
}

pub(super) fn length(
    budget: &mut Budget<'_>,
    value: veac_plan::canonical::Length,
) -> Result<LengthExpression, TemporalBackendError> {
    let (value, kind) = match value.unit {
        LengthUnit::Pixels => (value.value, LengthKind::Pixels),
        LengthUnit::Normalized => (value.value, LengthKind::Relative),
        LengthUnit::Percent => (value.value / 100.0, LengthKind::Relative),
    };
    Ok(LengthExpression {
        value: number(budget, value)?,
        kind,
    })
}
