use super::slot::public;
use super::Evaluator;
use crate::program::expression::core::{CoreInstruction, ValueId};
use crate::program::expression::{ExpressionError, TemporalAttachmentKind, Value};

impl Evaluator<'_> {
    pub(super) fn temporal_attachment(
        &mut self,
        kind: u8,
        owner: ValueId,
        selectors: &[ValueId],
        animation: ValueId,
        instruction: &CoreInstruction,
        values: &[Option<super::slot::RuntimeValue>],
    ) -> Result<Value, ExpressionError> {
        let kind = TemporalAttachmentKind::from_opcode(kind).ok_or_else(|| {
            ExpressionError::new(
                "EXPRESSION_RUNTIME_CONTRACT",
                "verified temporal attachment kind is unknown",
                instruction.span(),
            )
        })?;
        let owner = public(values, owner, instruction)?;
        let selectors = selectors
            .iter()
            .map(|value| public(values, *value, instruction))
            .collect::<Result<Vec<_>, _>>()?;
        let Value::Closure(animation) = public(values, animation, instruction)? else {
            return Err(ExpressionError::new(
                "EXPRESSION_RUNTIME_CONTRACT",
                "verified temporal attachment animation is not a closure",
                instruction.span(),
            ));
        };
        let instance = self.origin(instruction.span());
        self.domain.attach_temporal(
            kind,
            owner,
            selectors,
            animation,
            instruction.span(),
            instance,
        )
    }
}
