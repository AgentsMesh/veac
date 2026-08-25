use super::{
    DomainInstructionKind, DomainOperationSemantics, DomainRuntimeAction, TemporalLoweringOpcode,
};
use veac_lang_model::{Effect, Stage};

impl DomainOperationSemantics {
    pub(crate) const fn new(
        instruction: DomainInstructionKind,
        runtime_action: DomainRuntimeAction,
        effect: Effect,
        max_stage: Stage,
        temporal_lowering: Option<TemporalLoweringOpcode>,
    ) -> Self {
        Self {
            instruction,
            runtime_action,
            effect,
            max_stage,
            temporal_lowering,
        }
    }
}
