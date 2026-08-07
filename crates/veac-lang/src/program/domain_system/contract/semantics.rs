use super::{DomainInstructionKind, DomainOperationSemantics, DomainRuntimeAction};
use crate::program::expression::Effect;

impl DomainOperationSemantics {
    pub(in crate::program::domain_system) const fn new(
        instruction: DomainInstructionKind,
        runtime_action: DomainRuntimeAction,
        effect: Effect,
    ) -> Self {
        Self {
            instruction,
            runtime_action,
            effect,
        }
    }

    pub(in crate::program::domain_system) const fn description() -> Self {
        Self::new(
            DomainInstructionKind::DomainConstruct,
            DomainRuntimeAction::Description,
            Effect::Pure,
        )
    }

    pub(in crate::program::domain_system) const fn graph(
        runtime_action: DomainRuntimeAction,
    ) -> Self {
        Self::new(
            DomainInstructionKind::GraphEmit,
            runtime_action,
            Effect::GraphEmit,
        )
    }
}
