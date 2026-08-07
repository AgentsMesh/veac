use super::super::convert;
use super::super::evaluator::Evaluator;
use super::super::{ResidualRuntimeValue, ResidualizationError};
use crate::program::expression::{
    ArithmeticOperator, CoreInstruction, CoreInstructionKind, CoreUnaryOperator,
};
use veac_ir::{TemporalCompareOperation, TemporalNodeKind, TemporalUnaryOperation};

impl Evaluator<'_> {
    pub(super) fn unary(
        &mut self,
        operator: CoreUnaryOperator,
        operand: ResidualRuntimeValue,
        instruction: &CoreInstruction,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        if let ResidualRuntimeValue::Concrete(value) = operand {
            return crate::program::expression::runtime::residual_unary(
                operator,
                value,
                instruction.span(),
            )
            .map(ResidualRuntimeValue::Concrete)
            .map_err(ResidualizationError::expression);
        }
        if operator == CoreUnaryOperator::Positive {
            return Ok(operand);
        }
        let operand = self.builder.as_node(operand, instruction.span())?;
        let operation = match operator {
            CoreUnaryOperator::Negative => TemporalUnaryOperation::Negate,
            CoreUnaryOperator::Not => TemporalUnaryOperation::Not,
            CoreUnaryOperator::Positive => unreachable!(),
        };
        self.builder
            .node(
                self.instruction_type(instruction)?,
                TemporalNodeKind::Unary {
                    operation,
                    operand: operand.node_id,
                },
                instruction.span(),
            )
            .map(ResidualRuntimeValue::Residual)
    }

    pub(super) fn arithmetic(
        &mut self,
        operator: ArithmeticOperator,
        left: ResidualRuntimeValue,
        right: ResidualRuntimeValue,
        instruction: &CoreInstruction,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        if let (ResidualRuntimeValue::Concrete(left), ResidualRuntimeValue::Concrete(right)) =
            (&left, &right)
        {
            return crate::program::expression::runtime::residual_arithmetic(
                self.concrete_budget,
                operator,
                left.clone(),
                right.clone(),
                instruction.span(),
            )
            .map(ResidualRuntimeValue::Concrete)
            .map_err(ResidualizationError::expression);
        }
        let left = self.builder.as_node(left, instruction.span())?;
        let right = self.builder.as_node(right, instruction.span())?;
        self.builder
            .node(
                self.instruction_type(instruction)?,
                TemporalNodeKind::Binary {
                    operation: super::arithmetic(operator),
                    left: left.node_id,
                    right: right.node_id,
                },
                instruction.span(),
            )
            .map(ResidualRuntimeValue::Residual)
    }

    pub(super) fn compare(
        &mut self,
        operation: TemporalCompareOperation,
        left: ResidualRuntimeValue,
        right: ResidualRuntimeValue,
        instruction: &CoreInstruction,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        if let (ResidualRuntimeValue::Concrete(left), ResidualRuntimeValue::Concrete(right)) =
            (&left, &right)
        {
            let value = if matches!(instruction.kind(), CoreInstructionKind::Compare { .. }) {
                crate::program::expression::runtime::residual_ordering(
                    super::comparison_core(operation),
                    left.clone(),
                    right.clone(),
                    instruction.span(),
                )
            } else {
                crate::program::expression::runtime::residual_equality(
                    super::equality_core(operation),
                    left.clone(),
                    right.clone(),
                    instruction.span(),
                )
            };
            return value
                .map(ResidualRuntimeValue::Concrete)
                .map_err(ResidualizationError::expression);
        }
        let left = self.builder.as_node(left, instruction.span())?;
        let right = self.builder.as_node(right, instruction.span())?;
        self.builder
            .node(
                veac_ir::TemporalType::Boolean,
                TemporalNodeKind::Compare {
                    operation,
                    left: left.node_id,
                    right: right.node_id,
                },
                instruction.span(),
            )
            .map(ResidualRuntimeValue::Residual)
    }

    fn instruction_type(
        &self,
        instruction: &CoreInstruction,
    ) -> Result<veac_ir::TemporalType, ResidualizationError> {
        self.core()
            .value_type(instruction.type_id())
            .and_then(convert::value_type)
            .ok_or_else(|| {
                ResidualizationError::new(
                    "RESIDUAL_TYPE_UNSUPPORTED",
                    "Core result type has no closed Temporal IR representation",
                    instruction.span(),
                )
            })
    }
}
