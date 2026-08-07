mod node;

use veac_plan::canonical::TemporalBindingId;
use veac_plan::ResolvedRenderPlan;

use super::budget::Budget;
use super::error::TemporalBackendError;
use super::input;
use super::value::CompiledValue;
use crate::emitter::process_owner::ProcessOwner;

pub(in crate::emitter) fn compile_binding(
    plan: &ResolvedRenderPlan,
    binding_id: &TemporalBindingId,
    owner: ProcessOwner<'_>,
    local_clock: &str,
) -> Result<CompiledValue, TemporalBackendError> {
    let mut budget = Budget::new(binding_id);
    let (binding, program) = plan.temporal_program_for(binding_id).ok_or_else(|| {
        TemporalBackendError::new(
            "TEMPORAL_BACKEND_CONTRACT",
            binding_id,
            "temporal binding or program is absent from the render plan",
        )
    })?;
    let inputs = input::compile(plan, owner, binding, program, local_clock, &mut budget)?;
    let mut values = Vec::with_capacity(program.nodes.len());
    for temporal_node in &program.nodes {
        values.push(node::compile(temporal_node, &values, &inputs, &mut budget)?);
    }
    values
        .get(program.result.get() as usize)
        .cloned()
        .ok_or_else(|| budget.contract("temporal result node is absent"))
}
