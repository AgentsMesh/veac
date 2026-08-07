use std::ops::Range;

use super::super::error;
use crate::program::expression::core::{CoreInstruction, CoreProgram, CoreTerminator, ValueId};
use crate::program::expression::ExpressionError;

#[derive(Clone, Copy)]
pub(super) struct InstructionUse<'a> {
    pub(super) instruction: &'a CoreInstruction,
    pub(super) block: usize,
}

#[derive(Clone)]
enum Consumer<'a> {
    Instruction(InstructionUse<'a>),
    Escape(Range<usize>),
}

pub(super) struct Consumers<'a> {
    values: Vec<Vec<Consumer<'a>>>,
    definition_blocks: Vec<Option<usize>>,
}

impl<'a> Consumers<'a> {
    pub(super) fn collect(program: &'a CoreProgram) -> Self {
        let mut output = Self {
            values: (0..program.value_count()).map(|_| Vec::new()).collect(),
            definition_blocks: vec![None; program.value_count()],
        };
        for (block_index, block) in program.blocks.iter().enumerate() {
            for parameter in &block.parameters {
                output.define(parameter.id, block_index);
            }
            for instruction in &block.instructions {
                output.define(instruction.id, block_index);
                let usage = Consumer::Instruction(InstructionUse {
                    instruction,
                    block: block_index,
                });
                for operand in instruction.kind.operands() {
                    output.push(operand, usage.clone());
                }
            }
            output.terminator(&block.terminator);
        }
        output
    }

    pub(super) fn single(&self, id: ValueId) -> Result<InstructionUse<'a>, ExpressionError> {
        let consumers = &self.values[id.index().expect("verified token ID")];
        if consumers.len() != 1 {
            return Err(error("map token must have exactly one consuming use", 0..0));
        }
        match &consumers[0] {
            Consumer::Instruction(usage) => Ok(*usage),
            Consumer::Escape(span) => Err(error(
                "internal map token cannot escape through a terminator",
                span.clone(),
            )),
        }
    }

    pub(super) fn definition_block(&self, id: ValueId) -> usize {
        self.definition_blocks[id.index().expect("verified token ID")]
            .expect("verified token definition")
    }

    fn define(&mut self, id: ValueId, block: usize) {
        if let Some(slot) = id
            .index()
            .and_then(|index| self.definition_blocks.get_mut(index))
        {
            *slot = Some(block);
        }
    }

    fn push(&mut self, id: ValueId, consumer: Consumer<'a>) {
        if let Some(consumers) = id.index().and_then(|index| self.values.get_mut(index)) {
            consumers.push(consumer);
        }
    }

    fn terminator(&mut self, terminator: &CoreTerminator) {
        for value in terminator.operands() {
            self.push(value, Consumer::Escape(terminator.span().clone()));
        }
    }
}
