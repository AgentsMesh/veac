use super::definitions::Definitions;
use super::{error, type_table};
use crate::program::expression::core::{
    CoreInstruction, CoreInstructionKind, CoreProgram, ValueId,
};
use crate::program::expression::{ExpressionError, PrimitiveType, ValueTypeKind};

pub(super) fn verify(
    instruction: &CoreInstruction,
    program: &CoreProgram,
    definitions: &Definitions,
) -> Result<(), ExpressionError> {
    let CoreInstructionKind::Range { start, end, step } = &instruction.kind else {
        unreachable!("range verifier receives Range instructions")
    };
    let declared = type_table::value(program, instruction.type_id, instruction.span.clone())?;
    if !matches!(
        declared.kind(),
        ValueTypeKind::Range(element)
            if element.as_primitive() == Some(PrimitiveType::Integer)
    ) {
        return Err(error(
            "Range instruction must declare range<int>",
            instruction.span.clone(),
        ));
    }
    for operand in [Some(*start), Some(*end), *step].into_iter().flatten() {
        require_integer(operand, instruction, program, definitions)?;
    }
    Ok(())
}

fn require_integer(
    operand: ValueId,
    instruction: &CoreInstruction,
    program: &CoreProgram,
    definitions: &Definitions,
) -> Result<(), ExpressionError> {
    let actual = type_table::value(
        program,
        definitions.type_id(operand),
        instruction.span.clone(),
    )?;
    (actual.as_primitive() == Some(PrimitiveType::Integer))
        .then_some(())
        .ok_or_else(|| {
            error(
                "Range operands must have type int",
                instruction.span.clone(),
            )
        })
}
