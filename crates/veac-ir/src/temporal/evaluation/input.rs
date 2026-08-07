use std::collections::BTreeMap;

use crate::{TemporalEvaluationError, TemporalEvaluationInput, TemporalProgram, TemporalValue};

use super::Budget;

pub(super) type PreparedInputs<'a> = BTreeMap<u32, &'a TemporalValue>;

pub(super) fn prepare<'a>(
    program: &TemporalProgram,
    inputs: &'a [TemporalEvaluationInput],
    budget: &mut Budget,
) -> Result<PreparedInputs<'a>, TemporalEvaluationError> {
    let declarations: BTreeMap<_, _> = program
        .inputs
        .iter()
        .map(|input| (input.id.get(), input.value_type))
        .collect();
    let mut values = BTreeMap::new();
    for (index, input) in inputs.iter().enumerate() {
        let pointer = format!("/inputs/{index}");
        let Some(expected) = declarations.get(&input.input_id.get()) else {
            return Err(error(&pointer, "input was not declared by the program"));
        };
        if *expected != input.value.value_type() || !canonical(&input.value) {
            return Err(error(
                &pointer,
                "input value does not match its declared type",
            ));
        }
        budget.value(&input.value, &pointer)?;
        if values.insert(input.input_id.get(), &input.value).is_some() {
            return Err(error(&pointer, "input value is duplicated"));
        }
    }
    if values.len() != declarations.len() || declarations.keys().any(|id| !values.contains_key(id))
    {
        return Err(error("/inputs", "every declared input must have one value"));
    }
    Ok(values)
}

fn canonical(value: &TemporalValue) -> bool {
    match value {
        TemporalValue::Boolean { .. } | TemporalValue::Color { .. } => true,
        TemporalValue::Integer { value } => value.unsigned_abs() <= crate::MAX_SAFE_INTEGER,
        TemporalValue::Scalar { value } | TemporalValue::Angle { degrees: value } => number(*value),
        TemporalValue::Time { value } => value.is_valid(),
        TemporalValue::Length { value } => number(value.value),
        TemporalValue::Vec2 { value } => number(value.x) && number(value.y),
        TemporalValue::Point { value } => number(value.x.value) && number(value.y.value),
        TemporalValue::Rect { value } => [value.x, value.y, value.width, value.height]
            .into_iter()
            .all(number),
        TemporalValue::Text { value } => value.len() <= crate::MAX_TEMPORAL_TEXT_BYTES,
    }
}

fn number(value: f64) -> bool {
    value.is_finite() && !(value == 0.0 && value.is_sign_negative())
}

fn error(pointer: &str, message: &str) -> TemporalEvaluationError {
    TemporalEvaluationError::new("TEMPORAL_EVALUATION_INPUT", pointer, message)
}
