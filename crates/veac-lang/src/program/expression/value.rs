use std::fmt;

use super::ExactNumber;
mod render;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Scalar,
    Time,
    Length,
    Percent,
    Angle,
    Text,
    Color,
    Boolean,
    Identifier,
}

pub type ValueKind = ValueType;

impl ValueType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Scalar => "scalar",
            Self::Time => "time",
            Self::Length => "length",
            Self::Percent => "percent",
            Self::Angle => "angle",
            Self::Text => "text",
            Self::Color => "color",
            Self::Boolean => "bool",
            Self::Identifier => "identifier",
        }
    }

    pub(crate) fn is_numeric(self) -> bool {
        matches!(
            self,
            Self::Scalar | Self::Time | Self::Length | Self::Percent | Self::Angle
        )
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Scalar(ExactNumber),
    Time(ExactNumber),
    Length(ExactNumber),
    Percent(ExactNumber),
    Angle(ExactNumber),
    Text(String),
    Color(String),
    Bool(bool),
    Identifier(String),
}

impl Value {
    pub fn kind(&self) -> ValueKind {
        match self {
            Self::Scalar(_) => ValueKind::Scalar,
            Self::Time(_) => ValueKind::Time,
            Self::Length(_) => ValueKind::Length,
            Self::Percent(_) => ValueKind::Percent,
            Self::Angle(_) => ValueKind::Angle,
            Self::Text(_) => ValueKind::Text,
            Self::Color(_) => ValueKind::Color,
            Self::Bool(_) => ValueKind::Boolean,
            Self::Identifier(_) => ValueKind::Identifier,
        }
    }

    pub(crate) fn retained_bytes(&self) -> usize {
        match self {
            Self::Text(value) | Self::Color(value) | Self::Identifier(value) => value.len(),
            _ => 0,
        }
    }

    pub(crate) fn numeric(&self) -> Option<ExactNumber> {
        match self {
            Self::Scalar(value)
            | Self::Time(value)
            | Self::Length(value)
            | Self::Percent(value)
            | Self::Angle(value) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn from_numeric(kind: ValueKind, value: ExactNumber) -> Option<Self> {
        Some(match kind {
            ValueKind::Scalar => Self::Scalar(value),
            ValueKind::Time => Self::Time(value),
            ValueKind::Length => Self::Length(value),
            ValueKind::Percent => Self::Percent(value),
            ValueKind::Angle => Self::Angle(value),
            _ => return None,
        })
    }
}
