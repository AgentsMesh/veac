use super::{ExactNumber, PrimitiveType, Value};

impl Value {
    pub(crate) fn numeric(&self) -> Option<ExactNumber> {
        match self {
            Self::Integer(value) => Some(ExactNumber::integer(i128::from(*value))),
            Self::Scalar(value)
            | Self::Time(value)
            | Self::Length(value)
            | Self::Percent(value)
            | Self::Angle(value) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn from_numeric(kind: PrimitiveType, value: ExactNumber) -> Option<Self> {
        Some(match kind {
            PrimitiveType::Integer => {
                if value.denominator() != 1 {
                    return None;
                }
                Self::Integer(i64::try_from(value.numerator()).ok()?)
            }
            PrimitiveType::Scalar => Self::Scalar(value),
            PrimitiveType::Time => Self::Time(value),
            PrimitiveType::Length => Self::Length(value),
            PrimitiveType::Percent => Self::Percent(value),
            PrimitiveType::Angle => Self::Angle(value),
            _ => return None,
        })
    }
}
