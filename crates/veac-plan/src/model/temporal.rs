use veac_ir::{TemporalBinding, TemporalBindingId, TemporalProgram};

use super::ResolvedRenderPlan;

impl ResolvedRenderPlan {
    pub fn temporal_binding(&self, id: &TemporalBindingId) -> Option<&TemporalBinding> {
        self.temporal.bindings.iter().find(|value| value.id == *id)
    }

    pub fn temporal_program_for(
        &self,
        binding_id: &TemporalBindingId,
    ) -> Option<(&TemporalBinding, &TemporalProgram)> {
        let binding = self.temporal_binding(binding_id)?;
        let program = self
            .temporal
            .programs
            .iter()
            .find(|value| value.id == binding.program_id)?;
        Some((binding, program))
    }
}
