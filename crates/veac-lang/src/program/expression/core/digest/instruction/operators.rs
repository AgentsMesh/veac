use crate::program::expression::{
    ArithmeticOperator, CollectionOperation, ComparisonOperator, CoreTemporalComposeOperation,
    CoreTemporalProjectOperation, CoreUnaryOperator, EqualityOperator,
};

pub(super) fn unary(value: CoreUnaryOperator) -> u8 {
    match value {
        CoreUnaryOperator::Positive => 0,
        CoreUnaryOperator::Negative => 1,
        CoreUnaryOperator::Not => 2,
    }
}

pub(super) fn arithmetic(value: ArithmeticOperator) -> u8 {
    match value {
        ArithmeticOperator::Add => 0,
        ArithmeticOperator::Subtract => 1,
        ArithmeticOperator::Multiply => 2,
        ArithmeticOperator::Divide => 3,
    }
}

pub(super) fn comparison(value: ComparisonOperator) -> u8 {
    match value {
        ComparisonOperator::Less => 0,
        ComparisonOperator::LessEqual => 1,
        ComparisonOperator::Greater => 2,
        ComparisonOperator::GreaterEqual => 3,
    }
}

pub(super) fn equality(value: EqualityOperator) -> u8 {
    match value {
        EqualityOperator::Equal => 0,
        EqualityOperator::NotEqual => 1,
    }
}

pub(super) fn collection(value: CollectionOperation) -> u8 {
    match value {
        CollectionOperation::Map => 0,
        CollectionOperation::Filter => 1,
        CollectionOperation::Fold => 2,
    }
}

pub(super) fn compose(value: CoreTemporalComposeOperation) -> u8 {
    match value {
        CoreTemporalComposeOperation::Vec2 => 0,
        CoreTemporalComposeOperation::Point => 1,
        CoreTemporalComposeOperation::Rect => 2,
        CoreTemporalComposeOperation::Color => 3,
    }
}

pub(super) fn project(value: CoreTemporalProjectOperation) -> u8 {
    use CoreTemporalProjectOperation::*;
    match value {
        Vec2X => 0,
        Vec2Y => 1,
        PointX => 2,
        PointY => 3,
        RectX => 4,
        RectY => 5,
        RectWidth => 6,
        RectHeight => 7,
        ColorRed => 8,
        ColorGreen => 9,
        ColorBlue => 10,
        ColorAlpha => 11,
    }
}
