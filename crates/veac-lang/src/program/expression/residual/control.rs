use super::convert;
use super::effect;
use super::evaluator::Evaluator;
use super::{ResidualRuntimeValue, ResidualizationError};
use crate::program::expression::{BlockId, CoreTerminator, Value};
use veac_ir::{TemporalNodeKind, TemporalType};

impl Evaluator<'_> {
    pub(super) fn block(
        &mut self,
        id: BlockId,
        mut slots: Vec<Option<ResidualRuntimeValue>>,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        let block = id
            .index()
            .and_then(|index| self.core().blocks().get(index))
            .cloned()
            .ok_or_else(|| {
                ResidualizationError::new(
                    "RESIDUAL_CORE_CONTRACT",
                    format!("unknown Core block {}", id.value()),
                    0..0,
                )
            })?;
        for instruction in block.instructions() {
            effect::instruction(instruction)?;
        }
        for instruction in block.instructions() {
            let value = self.instruction(instruction, &slots)?;
            slots[instruction.id().index().expect("verified value ID")] = Some(value);
        }
        self.builder
            .budget
            .step(block.terminator().span().clone())?;
        match block.terminator() {
            CoreTerminator::Return { value, span } => self.value(&slots, *value, span.clone()),
            CoreTerminator::Jump {
                target,
                arguments,
                span,
            } => {
                self.bind_parameters(*target, arguments, &mut slots, span.clone())?;
                self.block(*target, slots)
            }
            CoreTerminator::Branch {
                condition,
                then_target,
                else_target,
                span,
            } => self.branch(
                self.value(&slots, *condition, span.clone())?,
                [*then_target, *else_target],
                slots,
                span.clone(),
            ),
            CoreTerminator::Match {
                scrutinee,
                arms,
                span,
            } => {
                let value = self.value(&slots, *scrutinee, span.clone())?;
                let ResidualRuntimeValue::Concrete(Value::Enum(value)) = value else {
                    return Err(ResidualizationError::new(
                        "RESIDUAL_TEMPORAL_MATCH_UNSUPPORTED",
                        "match selection must be concrete at Build stage",
                        span.clone(),
                    ));
                };
                let arm = arms.get(value.variant().index()).ok_or_else(|| {
                    ResidualizationError::new(
                        "RESIDUAL_CORE_CONTRACT",
                        "selected match arm is unavailable",
                        span.clone(),
                    )
                })?;
                self.bind_concrete_fields(arm.target(), value.fields(), &mut slots)?;
                self.block(arm.target(), slots)
            }
            CoreTerminator::ForEach(value) => self.for_each(value, slots),
        }
    }

    fn branch(
        &mut self,
        condition: ResidualRuntimeValue,
        targets: [BlockId; 2],
        slots: Vec<Option<ResidualRuntimeValue>>,
        span: std::ops::Range<usize>,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        match condition {
            ResidualRuntimeValue::Concrete(Value::Bool(value)) => {
                self.block(targets[usize::from(!value)], slots)
            }
            ResidualRuntimeValue::Concrete(_) => Err(ResidualizationError::new(
                "RESIDUAL_CORE_CONTRACT",
                "Core branch condition is not bool",
                span,
            )),
            condition @ ResidualRuntimeValue::Residual(_) => {
                if !effect::branches_are_pure(self.core(), targets) {
                    return Err(ResidualizationError::new(
                        "RESIDUAL_TEMPORAL_BRANCH_EFFECT",
                        "Temporal branch requires two provably Pure arms",
                        span,
                    ));
                }
                let when_true = self.block(targets[0], slots.clone())?;
                let when_false = self.block(targets[1], slots)?;
                self.select(condition, when_true, when_false, span)
            }
            ResidualRuntimeValue::TemporalConstant(_) | ResidualRuntimeValue::Sequence { .. } => {
                Err(ResidualizationError::new(
                    "RESIDUAL_CORE_CONTRACT",
                    "Core branch condition is not bool",
                    span,
                ))
            }
        }
    }

    pub(super) fn select(
        &mut self,
        condition: ResidualRuntimeValue,
        when_true: ResidualRuntimeValue,
        when_false: ResidualRuntimeValue,
        span: std::ops::Range<usize>,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        let result_type = convert::runtime_type(&when_true, span.clone())?;
        if convert::runtime_type(&when_false, span.clone())? != result_type {
            return Err(ResidualizationError::new(
                "RESIDUAL_CORE_CONTRACT",
                "Temporal branch arms have different result types",
                span,
            ));
        }
        let condition = self.builder.as_node(condition, span.clone())?;
        if condition.value_type != TemporalType::Boolean {
            return Err(ResidualizationError::new(
                "RESIDUAL_CORE_CONTRACT",
                "Temporal branch condition is not bool",
                span,
            ));
        }
        let when_true = self.builder.as_node(when_true, span.clone())?;
        let when_false = self.builder.as_node(when_false, span.clone())?;
        self.builder
            .node(
                result_type,
                TemporalNodeKind::Select {
                    condition: condition.node_id,
                    when_true: when_true.node_id,
                    when_false: when_false.node_id,
                },
                span,
            )
            .map(ResidualRuntimeValue::Residual)
    }

    fn bind_parameters(
        &self,
        target: BlockId,
        arguments: &[crate::program::expression::ValueId],
        slots: &mut [Option<ResidualRuntimeValue>],
        span: std::ops::Range<usize>,
    ) -> Result<(), ResidualizationError> {
        let parameters = self.core().blocks()[target.index().expect("verified block")].parameters();
        for (parameter, argument) in parameters.iter().zip(arguments) {
            slots[parameter.id().index().expect("verified value ID")] =
                Some(self.value(slots, *argument, span.clone())?);
        }
        Ok(())
    }

    fn bind_concrete_fields(
        &self,
        target: BlockId,
        fields: &[Value],
        slots: &mut [Option<ResidualRuntimeValue>],
    ) -> Result<(), ResidualizationError> {
        let parameters = self.core().blocks()[target.index().expect("verified block")].parameters();
        for (parameter, field) in parameters.iter().zip(fields) {
            slots[parameter.id().index().expect("verified value ID")] =
                Some(ResidualRuntimeValue::Concrete(field.clone()));
        }
        Ok(())
    }
}
