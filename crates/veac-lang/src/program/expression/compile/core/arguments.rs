use super::Builder;
use crate::program::expression::core::ValueId;
use crate::program::expression::hir::TypedCallArgument;
use crate::program::expression::hir::TypedDefaultArgument;
use crate::program::expression::{CoreCallTarget, CoreInstructionKind};

impl Builder<'_> {
    pub(super) fn call_arguments(
        &mut self,
        arguments: &[TypedCallArgument],
        defaults: &[TypedDefaultArgument],
        span: std::ops::Range<usize>,
    ) -> Vec<ValueId> {
        let mut ordered = vec![None; arguments.len() + defaults.len()];
        for argument in arguments {
            let value = self.node(&argument.value);
            let target = ordered
                .get_mut(argument.slot)
                .expect("typed call argument slot is in bounds");
            debug_assert!(target.is_none(), "typed call argument slots are unique");
            *target = Some(value);
        }
        for argument in defaults {
            let value = self.emit(
                CoreInstructionKind::Call {
                    target: CoreCallTarget::User(argument.target),
                    arguments: Vec::new(),
                },
                &argument.value_type,
                span.clone(),
            );
            let target = ordered
                .get_mut(argument.slot)
                .expect("typed default argument slot is in bounds");
            debug_assert!(target.is_none(), "typed call argument slots are unique");
            *target = Some(value);
        }
        ordered
            .into_iter()
            .map(|value| value.expect("typed call arguments fill every slot"))
            .collect()
    }

    pub(super) fn complete_arguments(&mut self, arguments: &[TypedCallArgument]) -> Vec<ValueId> {
        self.call_arguments(arguments, &[], 0..0)
    }
}
