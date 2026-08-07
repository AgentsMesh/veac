use super::{TemporalEvaluationError, TemporalEvaluationLimits, TemporalValue};

pub(super) struct Budget {
    limits: TemporalEvaluationLimits,
    steps: usize,
    value_bytes: usize,
}

impl Budget {
    pub(super) fn new(limits: TemporalEvaluationLimits) -> Self {
        Self {
            limits,
            steps: 0,
            value_bytes: 0,
        }
    }

    pub(super) fn step(&mut self, pointer: &str) -> Result<(), TemporalEvaluationError> {
        self.steps = self.steps.saturating_add(1);
        if self.steps > self.limits.max_steps {
            Err(limit(pointer, "evaluation step"))
        } else {
            Ok(())
        }
    }

    pub(super) fn value(
        &mut self,
        value: &TemporalValue,
        pointer: &str,
    ) -> Result<(), TemporalEvaluationError> {
        self.value_bytes = self.value_bytes.saturating_add(value.logical_bytes());
        if self.value_bytes > self.limits.max_value_bytes {
            Err(limit(pointer, "evaluated value byte"))
        } else {
            Ok(())
        }
    }
}

fn limit(pointer: &str, resource: &str) -> TemporalEvaluationError {
    TemporalEvaluationError::new(
        "TEMPORAL_EVALUATION_LIMIT",
        pointer,
        format!("temporal {resource} limit exceeded"),
    )
}
