use std::ops::Range;

use super::control::ControlFlow;
use super::error;
use crate::program::expression::core::{
    CoreProgram, CoreTypeId, CoreValueMetadata, InputId, ValueId,
};
use crate::program::expression::{ExpressionError, MAX_EXPRESSION_DEPTH, MAX_EXPRESSION_NODES};

#[cfg(test)]
mod tests;

#[derive(Clone)]
struct Definition {
    block: usize,
    position: Option<usize>,
    type_id: CoreTypeId,
    metadata: CoreValueMetadata,
    operands: Vec<ValueId>,
    span: Range<usize>,
}

pub(super) struct Definitions {
    values: Vec<Definition>,
    inputs: Vec<CoreTypeId>,
}

impl Definitions {
    pub(super) fn collect(program: &CoreProgram) -> Result<Self, ExpressionError> {
        let inputs = program
            .inputs
            .iter()
            .enumerate()
            .map(|(index, input)| {
                valid_span(&input.span)?;
                if input.id.index() != Some(index) || input.name.is_empty() {
                    return Err(error(
                        "input IDs must be dense and named",
                        input.span.clone(),
                    ));
                }
                Ok(input.type_id)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if inputs.len() > MAX_EXPRESSION_NODES {
            return Err(error(
                "Core program contains too many declared inputs",
                0..0,
            ));
        }
        let count = program.value_count();
        if count > MAX_EXPRESSION_NODES {
            return Err(ExpressionError::new(
                "EXPRESSION_NODE_LIMIT",
                format!("Core program exceeds the {MAX_EXPRESSION_NODES} value limit"),
                0..0,
            ));
        }
        let mut slots = vec![None; count];
        for (block_index, block) in program.blocks.iter().enumerate() {
            if block.id.index() != Some(block_index) {
                return Err(error("block IDs must be dense", 0..0));
            }
            for parameter in &block.parameters {
                valid_span(&parameter.span)?;
                insert(
                    &mut slots,
                    parameter.id,
                    Definition {
                        block: block_index,
                        position: None,
                        type_id: parameter.type_id,
                        metadata: parameter.metadata.clone(),
                        operands: Vec::new(),
                        span: parameter.span.clone(),
                    },
                )?;
            }
            for (position, instruction) in block.instructions.iter().enumerate() {
                valid_span(&instruction.span)?;
                insert(
                    &mut slots,
                    instruction.id,
                    Definition {
                        block: block_index,
                        position: Some(position),
                        type_id: instruction.type_id,
                        metadata: instruction.metadata.clone(),
                        operands: instruction.kind.operands().collect(),
                        span: instruction.span.clone(),
                    },
                )?;
            }
            valid_span(match &block.terminator {
                crate::program::expression::core::CoreTerminator::Return { span, .. }
                | crate::program::expression::core::CoreTerminator::Jump { span, .. }
                | crate::program::expression::core::CoreTerminator::Branch { span, .. }
                | crate::program::expression::core::CoreTerminator::Match { span, .. } => span,
                crate::program::expression::core::CoreTerminator::ForEach(value) => {
                    let loop_span = value.provenance().loop_span();
                    let binding_span = value.provenance().binding_span();
                    valid_span(binding_span)?;
                    loop_span
                }
            })?;
        }
        let values = slots
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| error("value IDs must be dense", 0..0))?;
        Ok(Self { values, inputs })
    }

    pub(super) fn type_id(&self, id: ValueId) -> CoreTypeId {
        self.values[id.index().expect("verified ValueId")].type_id
    }

    pub(super) fn input_type_id(&self, id: InputId) -> Option<CoreTypeId> {
        id.index().and_then(|index| self.inputs.get(index)).copied()
    }

    pub(super) fn metadata(&self, id: ValueId) -> &CoreValueMetadata {
        &self.values[id.index().expect("verified ValueId")].metadata
    }

    pub(super) fn combined_metadata(
        &self,
        operands: impl IntoIterator<Item = ValueId>,
    ) -> CoreValueMetadata {
        CoreValueMetadata::combine(operands.into_iter().map(|id| self.metadata(id)))
    }

    pub(super) fn verify_use(
        &self,
        id: ValueId,
        block: usize,
        position: usize,
        control: &ControlFlow,
        span: Range<usize>,
    ) -> Result<(), ExpressionError> {
        let Some(definition) = id.index().and_then(|index| self.values.get(index)) else {
            return Err(error(format!("unknown value ID {}", id.value()), span));
        };
        if definition.block == block {
            if definition
                .position
                .is_some_and(|defined| defined >= position)
            {
                return Err(error("value is used before its definition", span));
            }
        } else if !control.dominates(definition.block, block) {
            return Err(error("value definition does not dominate its use", span));
        }
        Ok(())
    }

    pub(super) fn verify_depth(&self) -> Result<(), ExpressionError> {
        let mut depths = vec![1usize; self.values.len()];
        for (index, definition) in self.values.iter().enumerate() {
            let depth = definition
                .operands
                .iter()
                .filter_map(|id| id.index().and_then(|value| depths.get(value)).copied())
                .max()
                .unwrap_or(0)
                .saturating_add(1);
            depths[index] = depth;
            if depth > MAX_EXPRESSION_DEPTH {
                return Err(ExpressionError::new(
                    "EXPRESSION_DEPTH_LIMIT",
                    format!("expression exceeds the {MAX_EXPRESSION_DEPTH} nesting limit"),
                    definition.span.clone(),
                ));
            }
        }
        Ok(())
    }
}

fn insert(
    slots: &mut [Option<Definition>],
    id: ValueId,
    definition: Definition,
) -> Result<(), ExpressionError> {
    let slot = id
        .index()
        .and_then(|index| slots.get_mut(index))
        .ok_or_else(|| error("value IDs must be dense", definition.span.clone()))?;
    if slot.replace(definition).is_some() {
        return Err(error("duplicate value ID", 0..0));
    }
    Ok(())
}

fn valid_span(span: &Range<usize>) -> Result<(), ExpressionError> {
    (span.start <= span.end)
        .then_some(())
        .ok_or_else(|| error("invalid source span", span.clone()))
}
