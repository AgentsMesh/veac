crate::define_syntax_tokens! {
    array
    pub enum BuiltinFunction {
        Min => "min",
        Max => "max",
        Clamp => "clamp",
        Identifier => "identifier",
        SampleCurveHold => "sample_curve_hold",
        SampleCurveLinear => "sample_curve_linear",
        SampleCurveEaseIn => "sample_curve_ease_in",
        SampleCurveEaseOut => "sample_curve_ease_out",
        SampleCurveEaseInOut => "sample_curve_ease_in_out",
        SampleCurveSpring => "sample_curve_spring",
        SampleCurveCubicBezier => "sample_curve_cubic_bezier",
    }
}

use super::{PrimitiveType, ValueType, ValueTypeKind};
use crate::program::DomainType;

impl BuiltinFunction {
    pub(crate) const fn arity(self) -> usize {
        match self {
            Self::Min | Self::Max => 2,
            Self::Clamp => 3,
            Self::Identifier => 1,
            Self::SampleCurveHold
            | Self::SampleCurveLinear
            | Self::SampleCurveEaseIn
            | Self::SampleCurveEaseOut
            | Self::SampleCurveEaseInOut => 2,
            Self::SampleCurveSpring => 5,
            Self::SampleCurveCubicBezier => 6,
        }
    }

    pub(crate) const fn is_curve(self) -> bool {
        matches!(
            self,
            Self::SampleCurveHold
                | Self::SampleCurveLinear
                | Self::SampleCurveEaseIn
                | Self::SampleCurveEaseOut
                | Self::SampleCurveEaseInOut
                | Self::SampleCurveSpring
                | Self::SampleCurveCubicBezier
        )
    }

    pub(crate) fn result_type(self, arguments: &[ValueType]) -> Result<ValueType, &'static str> {
        if arguments.len() != self.arity() {
            return Err("builtin call arity mismatch");
        }
        if self == Self::Identifier {
            return (arguments[0].as_primitive() == Some(PrimitiveType::Text))
                .then(|| PrimitiveType::Identifier.into())
                .ok_or("identifier argument must be text");
        }
        if self.is_curve() {
            return curve_type(arguments);
        }
        let first = &arguments[0];
        (first.is_numeric() && arguments.iter().all(|value| value == first))
            .then(|| first.clone())
            .ok_or("builtin numeric arguments do not match")
    }
}

fn curve_type(arguments: &[ValueType]) -> Result<ValueType, &'static str> {
    if !matches!(
        arguments[0].as_primitive(),
        Some(PrimitiveType::Scalar | PrimitiveType::Time)
    ) {
        return Err("curve input must be scalar or time");
    }
    let ValueTypeKind::List(key) = arguments[1].kind() else {
        return Err("curve keys must be a list of (position, value) tuples");
    };
    let ValueTypeKind::Tuple(fields) = key.kind() else {
        return Err("curve keys must be a list of (position, value) tuples");
    };
    if fields.len() != 2 || fields[0] != arguments[0] {
        return Err("curve key positions must match the input type");
    }
    if !curve_value_type(&fields[1]) {
        return Err("curve values must use an interpolatable Temporal type");
    }
    if arguments[2..]
        .iter()
        .any(|value| value.as_primitive() != Some(PrimitiveType::Scalar))
    {
        return Err("curve interpolation parameters must be scalar");
    }
    Ok(fields[1].clone())
}

fn curve_value_type(value: &ValueType) -> bool {
    matches!(
        value.as_primitive(),
        Some(
            PrimitiveType::Scalar
                | PrimitiveType::Percent
                | PrimitiveType::Length
                | PrimitiveType::Angle
                | PrimitiveType::Color
        )
    ) || matches!(
        value.as_domain(),
        Some(DomainType::Vector | DomainType::Point | DomainType::Rect)
    )
}
