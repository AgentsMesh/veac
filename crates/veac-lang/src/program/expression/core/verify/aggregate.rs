use super::collection::operand_type;
use super::{error, type_table};
use crate::program::expression::core::{
    CoreInstruction, CoreInstructionKind, CoreProgram, ValueId,
};
use crate::program::expression::{
    CollectionOperation, ExpressionError, PrimitiveType, ValueType, ValueTypeKind,
};

use super::definitions::Definitions;

mod effect;
#[cfg(test)]
mod tests;

pub(super) fn verify(
    instruction: &CoreInstruction,
    program: &CoreProgram,
    definitions: &Definitions,
    _registry: &crate::program::TypeRegistry,
) -> Result<(), ExpressionError> {
    let CoreInstructionKind::Collection {
        operation,
        iterable,
        initial,
        callable,
    } = &instruction.kind
    else {
        unreachable!("aggregate verifier receives a Collection instruction")
    };
    let element = iterable_element(
        operand_type(program, definitions, *iterable, instruction)?,
        instruction,
    )?;
    let callable_type = operand_type(program, definitions, *callable, instruction)?;
    let ValueTypeKind::Function {
        parameters, result, ..
    } = callable_type.kind()
    else {
        return Err(error(
            "aggregate callback must have a function type",
            instruction.span.clone(),
        ));
    };
    let expected = match *operation {
        CollectionOperation::Map => {
            require_no_initial(*initial, instruction)?;
            require_signature(parameters, std::slice::from_ref(&element), instruction)?;
            match ValueType::list(result.clone()) {
                Ok(value) => value,
                Err(_) => {
                    return Err(error(
                        "map result type is invalid",
                        instruction.span.clone(),
                    ))
                }
            }
        }
        CollectionOperation::Filter => {
            require_no_initial(*initial, instruction)?;
            require_signature(parameters, std::slice::from_ref(&element), instruction)?;
            if result.as_primitive() != Some(PrimitiveType::Boolean) {
                return Err(error(
                    "filter callback must return bool",
                    instruction.span.clone(),
                ));
            }
            match ValueType::list(element) {
                Ok(value) => value,
                Err(_) => {
                    return Err(error(
                        "filter result type is invalid",
                        instruction.span.clone(),
                    ))
                }
            }
        }
        CollectionOperation::Fold => {
            let Some(initial) = *initial else {
                return Err(error(
                    "fold instruction requires an initial value",
                    instruction.span.clone(),
                ));
            };
            let accumulator = operand_type(program, definitions, initial, instruction)?.clone();
            require_signature(parameters, &[accumulator.clone(), element], instruction)?;
            if result != &accumulator {
                return Err(error(
                    "fold callback result must match its accumulator",
                    instruction.span.clone(),
                ));
            }
            accumulator
        }
    };
    let actual = type_table::value(program, instruction.type_id, instruction.span.clone())?;
    effect::verify_callback(
        *operation,
        *callable,
        parameters.len(),
        definitions,
        instruction,
    )?;
    if actual == &expected {
        Ok(())
    } else {
        Err(error(
            format!("aggregate declares {actual}, expected {expected}"),
            instruction.span.clone(),
        ))
    }
}

fn iterable_element(
    value: &ValueType,
    instruction: &CoreInstruction,
) -> Result<ValueType, ExpressionError> {
    match value.kind() {
        ValueTypeKind::List(value) | ValueTypeKind::Range(value) => Ok(value.clone()),
        ValueTypeKind::Map { key, value } => {
            match ValueType::tuple(vec![ValueType::primitive(key.primitive()), value.clone()]) {
                Ok(value) => Ok(value),
                Err(_) => Err(error(
                    "map iterable entry type is invalid",
                    instruction.span.clone(),
                )),
            }
        }
        _ => Err(error(
            "aggregate input must be list, range, or map",
            instruction.span.clone(),
        )),
    }
}

fn require_no_initial(
    initial: Option<ValueId>,
    instruction: &CoreInstruction,
) -> Result<(), ExpressionError> {
    if initial.is_none() {
        Ok(())
    } else {
        Err(error(
            "map/filter instruction cannot carry an initial value",
            instruction.span.clone(),
        ))
    }
}

fn require_signature(
    actual: &[ValueType],
    expected: &[ValueType],
    instruction: &CoreInstruction,
) -> Result<(), ExpressionError> {
    if actual == expected {
        Ok(())
    } else {
        Err(error(
            "aggregate callback parameter signature does not match its input",
            instruction.span.clone(),
        ))
    }
}
