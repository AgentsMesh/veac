use super::effect;
use super::evaluator::Evaluator;
use super::{ResidualRuntimeValue, ResidualizationError};
use crate::program::expression::{
    ArithmeticOperator, ComparisonOperator, CoreInputIdentity, CoreInstruction,
    CoreInstructionKind, EqualityOperator,
};
use veac_ir::{TemporalBinaryOperation, TemporalCompareOperation};

mod call;
mod collection;
mod map;
mod operation;
mod range;

impl Evaluator<'_> {
    pub(super) fn instruction(
        &mut self,
        instruction: &CoreInstruction,
        slots: &[Option<ResidualRuntimeValue>],
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        self.builder.budget.step(instruction.span())?;
        effect::instruction(instruction)?;
        match instruction.kind() {
            CoreInstructionKind::Literal(value) => Ok(self.concrete(value.clone())),
            CoreInstructionKind::Input(id) => {
                let input = self.core().inputs()[id.index().expect("verified input")].clone();
                match input.identity().clone() {
                    CoreInputIdentity::Build(id) => self
                        .bindings
                        .get(&id)
                        .cloned()
                        .map(ResidualRuntimeValue::Concrete)
                        .ok_or_else(|| {
                            ResidualizationError::new(
                                "RESIDUAL_BUILD_INPUT_MISSING",
                                format!("missing Build input `{}`", input.name()),
                                input.span(),
                            )
                        }),
                    CoreInputIdentity::Temporal(identity) => self.builder.input(&input, &identity),
                }
            }
            CoreInstructionKind::Parameter(index) => {
                self.parameters.get(*index).cloned().ok_or_else(|| {
                    ResidualizationError::new(
                        "RESIDUAL_CORE_CONTRACT",
                        format!("Core parameter {index} is unavailable"),
                        instruction.span(),
                    )
                })
            }
            CoreInstructionKind::Capture(index) => {
                self.captures.get(*index).cloned().ok_or_else(|| {
                    ResidualizationError::new(
                        "RESIDUAL_CORE_CONTRACT",
                        format!("Core capture {index} is unavailable"),
                        instruction.span(),
                    )
                })
            }
            CoreInstructionKind::Closure {
                definition,
                captures,
            } => self.closure(*definition, captures, instruction, slots),
            CoreInstructionKind::Call { target, arguments } => {
                self.call(*target, arguments, instruction, slots)
            }
            CoreInstructionKind::Unary { operator, operand } => {
                let operand = self.value(slots, *operand, instruction.span())?;
                self.unary(*operator, operand, instruction)
            }
            CoreInstructionKind::Arithmetic {
                operator,
                left,
                right,
            } => {
                let left = self.value(slots, *left, instruction.span())?;
                let right = self.value(slots, *right, instruction.span())?;
                self.arithmetic(*operator, left, right, instruction)
            }
            CoreInstructionKind::Compare {
                operator,
                left,
                right,
            } => {
                let left = self.value(slots, *left, instruction.span())?;
                let right = self.value(slots, *right, instruction.span())?;
                self.compare(ordering(*operator), left, right, instruction)
            }
            CoreInstructionKind::Equal {
                operator,
                left,
                right,
            } => {
                let left = self.value(slots, *left, instruction.span())?;
                let right = self.value(slots, *right, instruction.span())?;
                self.compare(equality(*operator), left, right, instruction)
            }
            CoreInstructionKind::DomainConstruct { opcode, operands } => {
                self.domain_construct(*opcode, operands, instruction, slots)
            }
            CoreInstructionKind::TemporalCompose {
                operation,
                operands,
            } => self.temporal_compose(*operation, operands, instruction, slots),
            CoreInstructionKind::TemporalProject { operation, value } => {
                let value = self.value(slots, *value, instruction.span())?;
                self.temporal_project(*operation, value, instruction)
            }
            CoreInstructionKind::List { elements } => {
                self.sequence(elements, false, instruction, slots)
            }
            CoreInstructionKind::Tuple { elements } => {
                self.sequence(elements, true, instruction, slots)
            }
            CoreInstructionKind::Collection {
                operation,
                iterable,
                initial,
                callable,
            } => self.aggregate(
                *operation,
                *iterable,
                *initial,
                *callable,
                instruction,
                slots,
            ),
            CoreInstructionKind::Range { start, end, step } => {
                self.range(*start, *end, *step, instruction, slots)
            }
            CoreInstructionKind::MapBegin { entries } => self.map_begin(*entries, instruction),
            CoreInstructionKind::MapKey {
                builder,
                key,
                ordinal,
            } => self.map_key(*builder, *key, *ordinal, instruction, slots),
            CoreInstructionKind::MapValue { pending, value } => {
                self.map_value(*pending, *value, instruction, slots)
            }
            CoreInstructionKind::MapFinish { builder } => self.map_finish(*builder, instruction),
            value => Err(ResidualizationError::new(
                "RESIDUAL_INSTRUCTION_UNSUPPORTED",
                format!("Core instruction {value:?} is not residualizable"),
                instruction.span(),
            )),
        }
    }
}

fn arithmetic(value: ArithmeticOperator) -> TemporalBinaryOperation {
    match value {
        ArithmeticOperator::Add => TemporalBinaryOperation::Add,
        ArithmeticOperator::Subtract => TemporalBinaryOperation::Subtract,
        ArithmeticOperator::Multiply => TemporalBinaryOperation::Multiply,
        ArithmeticOperator::Divide => TemporalBinaryOperation::Divide,
    }
}

fn ordering(value: ComparisonOperator) -> TemporalCompareOperation {
    match value {
        ComparisonOperator::Less => TemporalCompareOperation::Less,
        ComparisonOperator::LessEqual => TemporalCompareOperation::LessOrEqual,
        ComparisonOperator::Greater => TemporalCompareOperation::Greater,
        ComparisonOperator::GreaterEqual => TemporalCompareOperation::GreaterOrEqual,
    }
}

fn equality(value: EqualityOperator) -> TemporalCompareOperation {
    match value {
        EqualityOperator::Equal => TemporalCompareOperation::Equal,
        EqualityOperator::NotEqual => TemporalCompareOperation::NotEqual,
    }
}

fn comparison_core(value: TemporalCompareOperation) -> ComparisonOperator {
    match value {
        TemporalCompareOperation::Less => ComparisonOperator::Less,
        TemporalCompareOperation::LessOrEqual => ComparisonOperator::LessEqual,
        TemporalCompareOperation::Greater => ComparisonOperator::Greater,
        TemporalCompareOperation::GreaterOrEqual => ComparisonOperator::GreaterEqual,
        _ => unreachable!(),
    }
}

fn equality_core(value: TemporalCompareOperation) -> EqualityOperator {
    match value {
        TemporalCompareOperation::Equal => EqualityOperator::Equal,
        TemporalCompareOperation::NotEqual => EqualityOperator::NotEqual,
        _ => unreachable!(),
    }
}
