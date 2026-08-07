use crate::program::expression::ast::{BinaryOperator, UnaryOperator};
use crate::program::expression::core::{
    ArithmeticOperator, ComparisonOperator, CoreInstructionKind, CoreUnaryOperator,
    EqualityOperator, ValueId,
};

pub(super) fn unary(operator: UnaryOperator) -> CoreUnaryOperator {
    match operator {
        UnaryOperator::Positive => CoreUnaryOperator::Positive,
        UnaryOperator::Negative => CoreUnaryOperator::Negative,
        UnaryOperator::Not => CoreUnaryOperator::Not,
    }
}

pub(super) fn binary(
    operator: BinaryOperator,
    left: ValueId,
    right: ValueId,
) -> CoreInstructionKind {
    use BinaryOperator as Surface;
    match operator {
        Surface::Add => arithmetic(ArithmeticOperator::Add, left, right),
        Surface::Subtract => arithmetic(ArithmeticOperator::Subtract, left, right),
        Surface::Multiply => arithmetic(ArithmeticOperator::Multiply, left, right),
        Surface::Divide => arithmetic(ArithmeticOperator::Divide, left, right),
        Surface::Less => compare(ComparisonOperator::Less, left, right),
        Surface::LessEqual => compare(ComparisonOperator::LessEqual, left, right),
        Surface::Greater => compare(ComparisonOperator::Greater, left, right),
        Surface::GreaterEqual => compare(ComparisonOperator::GreaterEqual, left, right),
        Surface::Equal => equal(EqualityOperator::Equal, left, right),
        Surface::NotEqual => equal(EqualityOperator::NotEqual, left, right),
        Surface::LogicalAnd | Surface::LogicalOr => unreachable!("logical operators branch"),
    }
}

fn arithmetic(operator: ArithmeticOperator, left: ValueId, right: ValueId) -> CoreInstructionKind {
    CoreInstructionKind::Arithmetic {
        operator,
        left,
        right,
    }
}

fn compare(operator: ComparisonOperator, left: ValueId, right: ValueId) -> CoreInstructionKind {
    CoreInstructionKind::Compare {
        operator,
        left,
        right,
    }
}

fn equal(operator: EqualityOperator, left: ValueId, right: ValueId) -> CoreInstructionKind {
    CoreInstructionKind::Equal {
        operator,
        left,
        right,
    }
}
