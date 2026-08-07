use super::definitions::Definitions;
use super::{error, type_table};
use crate::program::expression::core::{
    CoreInstruction, CoreInstructionKind, CoreProgram, ValueId,
};
use crate::program::expression::{ExpressionError, ValueType, ValueTypeKind};

mod map;

pub(super) fn verify(
    instruction: &CoreInstruction,
    program: &CoreProgram,
    definitions: &Definitions,
) -> Result<(), ExpressionError> {
    match &instruction.kind {
        CoreInstructionKind::List { elements } => list(instruction, elements, program, definitions),
        CoreInstructionKind::Tuple { elements } => {
            tuple(instruction, elements, program, definitions)
        }
        CoreInstructionKind::MapBegin { .. } => map::begin(instruction, program),
        CoreInstructionKind::MapKey { builder, key, .. } => {
            map::key(instruction, *builder, *key, program, definitions)
        }
        CoreInstructionKind::MapValue { pending, value } => {
            map::value(instruction, *pending, *value, program, definitions)
        }
        CoreInstructionKind::MapFinish { builder } => {
            map::finish(instruction, *builder, program, definitions)
        }
        _ => unreachable!("collection verifier receives collection instructions"),
    }
}

fn list(
    instruction: &CoreInstruction,
    elements: &[ValueId],
    program: &CoreProgram,
    definitions: &Definitions,
) -> Result<(), ExpressionError> {
    let actual = type_table::value(program, instruction.type_id, instruction.span.clone())?;
    let ValueTypeKind::List(element) = actual.kind() else {
        return Err(error(
            "list instruction must declare a list type",
            instruction.span.clone(),
        ));
    };
    require_elements(
        elements,
        std::iter::repeat(element),
        program,
        definitions,
        instruction,
    )
}

fn tuple(
    instruction: &CoreInstruction,
    elements: &[ValueId],
    program: &CoreProgram,
    definitions: &Definitions,
) -> Result<(), ExpressionError> {
    let actual = type_table::value(program, instruction.type_id, instruction.span.clone())?;
    let ValueTypeKind::Tuple(expected) = actual.kind() else {
        return Err(error(
            "tuple instruction must declare a tuple type",
            instruction.span.clone(),
        ));
    };
    if elements.len() != expected.len() {
        return Err(error(
            "tuple instruction arity does not match its type",
            instruction.span.clone(),
        ));
    }
    require_elements(elements, expected.iter(), program, definitions, instruction)
}

fn require_elements<'a>(
    elements: &[ValueId],
    expected: impl Iterator<Item = &'a ValueType>,
    program: &CoreProgram,
    definitions: &Definitions,
    instruction: &CoreInstruction,
) -> Result<(), ExpressionError> {
    for (element, expected) in elements.iter().zip(expected) {
        let actual = operand_type(program, definitions, *element, instruction)?;
        if actual != expected {
            return Err(error(
                "collection element type does not match",
                instruction.span.clone(),
            ));
        }
    }
    Ok(())
}

pub(super) fn operand_type<'a>(
    program: &'a CoreProgram,
    definitions: &Definitions,
    id: ValueId,
    instruction: &CoreInstruction,
) -> Result<&'a ValueType, ExpressionError> {
    type_table::value(program, definitions.type_id(id), instruction.span.clone())
}
