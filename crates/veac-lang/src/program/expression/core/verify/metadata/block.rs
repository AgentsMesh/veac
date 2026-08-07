use super::super::control::ControlFlow;
use super::super::definitions::Definitions;
use super::super::error;
use crate::program::expression::core::{CoreProgram, CoreTerminator, CoreValueMetadata};
use crate::program::expression::{ExpressionError, ValueTypeKind};

#[cfg(test)]
mod tests;

pub(crate) fn argument(
    argument: &CoreValueMetadata,
    parameter: &CoreValueMetadata,
    span: std::ops::Range<usize>,
) -> Result<(), ExpressionError> {
    let compatible = parameter.effect.covers(argument.effect)
        && parameter.shape_stage >= argument.shape_stage
        && parameter.leaf_stage >= argument.leaf_stage
        && parameter
            .shape_dependencies
            .contains(&argument.shape_dependencies)
        && parameter
            .leaf_dependencies
            .contains(&argument.leaf_dependencies);
    compatible
        .then_some(())
        .ok_or_else(|| error("block parameter metadata does not cover its argument", span))
}

pub(crate) fn parameters(
    program: &CoreProgram,
    definitions: &Definitions,
    control: &ControlFlow,
    _registry: &crate::program::TypeRegistry,
) -> Result<(), ExpressionError> {
    for (target_index, target) in program.blocks.iter().enumerate() {
        for (parameter_index, parameter) in target.parameters.iter().enumerate() {
            let incoming = jumps(program, target.id, parameter_index);
            let loop_results = for_each_results(program, target.id, parameter_index);
            let conditions = control.converging_conditions(program, target_index);
            let payload = match_payload(program, target.id);
            let parameter_type = program.value_type(parameter.type_id).ok_or_else(|| {
                error(
                    "block parameter must have a value type",
                    parameter.span.clone(),
                )
            })?;
            let mut expected = match payload {
                Some((scrutinee, variant)) => definitions
                    .metadata(scrutinee)
                    .enum_field(
                        variant,
                        crate::program::FieldIndex::from_position(parameter_index).ok_or_else(
                            || error("match field index overflows", parameter.span.clone()),
                        )?,
                        parameter_type,
                    )
                    .ok_or_else(|| {
                        error(
                            "match payload lacks a closed field metadata contract",
                            parameter.span.clone(),
                        )
                    })?,
                None if incoming.is_empty() && conditions.is_empty() && loop_results.len() == 1 => {
                    loop_results[0].clone()
                }
                None => CoreValueMetadata::combine(
                    incoming
                        .iter()
                        .chain(&conditions)
                        .map(|id| definitions.metadata(*id))
                        .chain(loop_results.iter().copied()),
                ),
            };
            if payload.is_none()
                && matches!(
                    parameter_type.kind(),
                    ValueTypeKind::Nominal(_) | ValueTypeKind::Function { .. }
                )
            {
                let values = incoming
                    .iter()
                    .map(|id| definitions.metadata(*id))
                    .collect::<Vec<_>>();
                expected.join_contract_from(&values);
            }
            if parameter.metadata != expected {
                return Err(error(
                    "block parameter metadata is not the exact control/data dependency union",
                    parameter.span.clone(),
                ));
            }
        }
    }
    Ok(())
}

fn for_each_results(
    program: &CoreProgram,
    target: crate::program::expression::BlockId,
    parameter: usize,
) -> Vec<&CoreValueMetadata> {
    if parameter != 0 {
        return Vec::new();
    }
    program
        .blocks
        .iter()
        .filter_map(|source| match &source.terminator {
            CoreTerminator::ForEach(value) if value.continuation() == target => {
                Some(value.result_metadata())
            }
            _ => None,
        })
        .collect()
}

fn jumps(
    program: &CoreProgram,
    target: crate::program::expression::BlockId,
    parameter: usize,
) -> Vec<crate::program::expression::ValueId> {
    program
        .blocks
        .iter()
        .filter_map(|source| {
            let CoreTerminator::Jump {
                target: destination,
                arguments,
                ..
            } = &source.terminator
            else {
                return None;
            };
            (*destination == target)
                .then(|| arguments.get(parameter).copied())
                .flatten()
        })
        .collect()
}

fn match_payload(
    program: &CoreProgram,
    target: crate::program::expression::BlockId,
) -> Option<(
    crate::program::expression::ValueId,
    crate::program::VariantIndex,
)> {
    program.blocks.iter().find_map(|block| {
        let CoreTerminator::Match {
            scrutinee, arms, ..
        } = &block.terminator
        else {
            return None;
        };
        arms.iter()
            .find(|arm| arm.target() == target)
            .map(|arm| (*scrutinee, arm.variant()))
    })
}
