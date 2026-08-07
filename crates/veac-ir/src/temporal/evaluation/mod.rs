mod budget;
mod curve;
mod evaluator;
mod input;
mod node;
mod operation;
#[cfg(test)]
pub(super) mod test_support;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    validate_temporal_program, TemporalEvaluationError, TemporalInputId, TemporalProgram,
    TemporalValue, MAX_TEMPORAL_NODES, MAX_TEMPORAL_VALUE_BYTES,
};
use budget::Budget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemporalEvaluationLimits {
    pub max_steps: usize,
    pub max_value_bytes: usize,
}

impl Default for TemporalEvaluationLimits {
    fn default() -> Self {
        Self {
            max_steps: MAX_TEMPORAL_NODES,
            max_value_bytes: MAX_TEMPORAL_VALUE_BYTES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TemporalEvaluationInput {
    pub input_id: TemporalInputId,
    pub value: TemporalValue,
}

pub fn evaluate_temporal_program(
    program: &TemporalProgram,
    inputs: &[TemporalEvaluationInput],
    limits: TemporalEvaluationLimits,
) -> Result<TemporalValue, TemporalEvaluationError> {
    if let Err(errors) = validate_temporal_program(program) {
        let first = &errors.diagnostics()[0];
        return Err(TemporalEvaluationError::new(
            "TEMPORAL_EVALUATION_PROGRAM",
            first.pointer.clone(),
            format!("{}: {}", first.code, first.message),
        ));
    }
    let mut budget = Budget::new(limits);
    let inputs = input::prepare(program, inputs, &mut budget)?;
    evaluator::evaluate(program, &inputs, &mut budget)
}
