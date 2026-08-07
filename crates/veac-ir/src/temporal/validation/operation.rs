use crate::temporal::{
    TemporalBinaryOperation as Binary, TemporalCompareOperation as Compare, TemporalType as Type,
    TemporalUnaryOperation as Unary,
};

pub(super) fn unary(operation: Unary, operand: Type) -> Option<Type> {
    match operation {
        Unary::Not if operand == Type::Boolean => Some(Type::Boolean),
        Unary::Negate | Unary::Absolute
            if matches!(
                operand,
                Type::Integer | Type::Scalar | Type::Time | Type::Length | Type::Angle | Type::Vec2
            ) =>
        {
            Some(operand)
        }
        Unary::Floor
        | Unary::Ceil
        | Unary::Round
        | Unary::SquareRoot
        | Unary::Exponential
        | Unary::NaturalLog
            if operand == Type::Scalar =>
        {
            Some(Type::Scalar)
        }
        Unary::Sine | Unary::Cosine if operand == Type::Angle => Some(Type::Scalar),
        _ => None,
    }
}

pub(super) fn binary(operation: Binary, left: Type, right: Type) -> Option<Type> {
    match operation {
        Binary::Add | Binary::Subtract
            if left == right
                && matches!(
                    left,
                    Type::Integer
                        | Type::Scalar
                        | Type::Time
                        | Type::Length
                        | Type::Angle
                        | Type::Vec2
                ) =>
        {
            Some(left)
        }
        Binary::Multiply if left == Type::Integer && right == Type::Integer => Some(Type::Integer),
        Binary::Multiply if left == Type::Scalar && right == Type::Scalar => Some(Type::Scalar),
        Binary::Multiply if scalable(left) && right == Type::Scalar => Some(left),
        Binary::Multiply if left == Type::Scalar && scalable(right) => Some(right),
        Binary::Divide if left == Type::Integer && right == Type::Integer => Some(Type::Integer),
        Binary::Divide if left == Type::Scalar && right == Type::Scalar => Some(Type::Scalar),
        Binary::Divide if left == right && ratio(left) => Some(Type::Scalar),
        Binary::Divide if scalable(left) && right == Type::Scalar => Some(left),
        Binary::Minimum | Binary::Maximum if left == right && ordered(left) => Some(left),
        _ => None,
    }
}

pub(super) fn compare(operation: Compare, left: Type, right: Type) -> Option<Type> {
    if left != right {
        return None;
    }
    match operation {
        Compare::Equal | Compare::NotEqual => Some(Type::Boolean),
        _ if ordered(left) => Some(Type::Boolean),
        _ => None,
    }
}

fn scalable(value: Type) -> bool {
    matches!(value, Type::Time | Type::Length | Type::Angle | Type::Vec2)
}

fn ratio(value: Type) -> bool {
    matches!(value, Type::Time | Type::Length | Type::Angle)
}

fn ordered(value: Type) -> bool {
    matches!(
        value,
        Type::Integer | Type::Scalar | Type::Time | Type::Length | Type::Angle
    )
}

pub(super) fn interpolatable(value: Type) -> bool {
    matches!(
        value,
        Type::Scalar
            | Type::Length
            | Type::Angle
            | Type::Vec2
            | Type::Point
            | Type::Rect
            | Type::Color
    )
}
