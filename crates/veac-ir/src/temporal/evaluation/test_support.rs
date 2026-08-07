use std::collections::BTreeMap;

use crate::{TemporalEvaluationError, TemporalNodeKind, TemporalValue};

pub(in crate::temporal) fn evaluate_node(
    kind: &TemporalNodeKind,
    values: &[TemporalValue],
    inputs: &BTreeMap<u32, TemporalValue>,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    let values: Vec<_> = values.iter().cloned().map(Some).collect();
    let inputs = inputs.iter().map(|(id, value)| (*id, value)).collect();
    super::node::evaluate(kind, &values, &inputs, pointer)
}
