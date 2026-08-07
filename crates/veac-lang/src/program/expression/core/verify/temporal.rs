use super::definitions::Definitions;
use super::{error, type_table};
use crate::program::expression::{
    CoreInstruction, CoreInstructionKind, CoreProgram, CoreTemporalComposeOperation as Compose,
    CoreTemporalProjectOperation as Project, ExpressionError, PrimitiveType, ValueType,
};
use crate::program::DomainType;

pub(super) fn instruction(
    instruction: &CoreInstruction,
    program: &CoreProgram,
    definitions: &Definitions,
) -> Result<ValueType, ExpressionError> {
    match instruction.kind() {
        CoreInstructionKind::TemporalCompose {
            operation,
            operands,
        } => compose(*operation, operands, instruction, program, definitions),
        CoreInstructionKind::TemporalProject { operation, value } => {
            project(*operation, *value, instruction, program, definitions)
        }
        _ => Err(error(
            "Core instruction is not a temporal composite operation",
            instruction.span(),
        )),
    }
}

fn compose(
    operation: Compose,
    operands: &[crate::program::expression::ValueId],
    instruction: &CoreInstruction,
    program: &CoreProgram,
    definitions: &Definitions,
) -> Result<ValueType, ExpressionError> {
    if operands.len() != operation.arity() {
        return Err(error(
            "TemporalCompose operand arity does not match its closed operation",
            instruction.span(),
        ));
    }
    let (operand, result) = compose_types(operation);
    for value in operands {
        let actual = type_table::value(program, definitions.type_id(*value), instruction.span())?;
        if actual != &operand {
            return Err(error(
                format!("TemporalCompose expects {operand}, found {actual}"),
                instruction.span(),
            ));
        }
    }
    Ok(result)
}

fn project(
    operation: Project,
    value: crate::program::expression::ValueId,
    instruction: &CoreInstruction,
    program: &CoreProgram,
    definitions: &Definitions,
) -> Result<ValueType, ExpressionError> {
    let (operand, result) = project_types(operation);
    let actual = type_table::value(program, definitions.type_id(value), instruction.span())?;
    (actual == &operand).then_some(result).ok_or_else(|| {
        error(
            format!("TemporalProject expects {operand}, found {actual}"),
            instruction.span(),
        )
    })
}

fn compose_types(value: Compose) -> (ValueType, ValueType) {
    match value {
        Compose::Vec2 => (
            PrimitiveType::Scalar.into(),
            ValueType::domain(DomainType::Vector),
        ),
        Compose::Point => (
            PrimitiveType::Length.into(),
            ValueType::domain(DomainType::Point),
        ),
        Compose::Rect => (
            PrimitiveType::Scalar.into(),
            ValueType::domain(DomainType::Rect),
        ),
        Compose::Color => (PrimitiveType::Integer.into(), PrimitiveType::Color.into()),
    }
}

fn project_types(value: Project) -> (ValueType, ValueType) {
    use Project::*;
    match value {
        Vec2X | Vec2Y => (
            ValueType::domain(DomainType::Vector),
            PrimitiveType::Scalar.into(),
        ),
        PointX | PointY => (
            ValueType::domain(DomainType::Point),
            PrimitiveType::Length.into(),
        ),
        RectX | RectY | RectWidth | RectHeight => (
            ValueType::domain(DomainType::Rect),
            PrimitiveType::Scalar.into(),
        ),
        ColorRed | ColorGreen | ColorBlue | ColorAlpha => {
            (PrimitiveType::Color.into(), PrimitiveType::Integer.into())
        }
    }
}
