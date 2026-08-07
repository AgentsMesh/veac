use std::collections::BTreeMap;

use veac_plan::canonical::{TemporalBinding, TemporalProgram};
use veac_plan::ResolvedRenderPlan;

use super::budget::Budget;
use super::clock;
use super::error::TemporalBackendError;
use super::value::CompiledValue;
use crate::emitter::process_owner::ProcessOwner;

pub(super) fn compile(
    plan: &ResolvedRenderPlan,
    owner: ProcessOwner<'_>,
    binding: &TemporalBinding,
    program: &TemporalProgram,
    local_clock: &str,
    budget: &mut Budget<'_>,
) -> Result<BTreeMap<u32, CompiledValue>, TemporalBackendError> {
    let mut values = BTreeMap::new();
    for value in &binding.clocks {
        values.insert(
            value.input_id.get(),
            clock::compile(plan, owner, value, local_clock, budget)?,
        );
    }
    for value in &binding.parameters {
        values.insert(
            value.input_id.get(),
            CompiledValue::literal(&value.value, budget)?,
        );
    }
    if values.len() != program.inputs.len() {
        return Err(budget.contract("temporal input bindings are incomplete"));
    }
    Ok(values)
}
