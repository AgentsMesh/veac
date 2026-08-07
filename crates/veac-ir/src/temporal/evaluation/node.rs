use crate::{TemporalEvaluationError, TemporalNodeId, TemporalNodeKind, TemporalValue};

use super::{curve, input::PreparedInputs, operation};

pub(super) fn evaluate(
    kind: &TemporalNodeKind,
    values: &[Option<TemporalValue>],
    inputs: &PreparedInputs<'_>,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    use TemporalNodeKind::*;
    match kind {
        Literal { value } => Ok(value.clone()),
        Input { input_id } => match inputs.get(&input_id.get()) {
            Some(value) => Ok((*value).clone()),
            None => Err(TemporalEvaluationError::new(
                "TEMPORAL_EVALUATION_INPUT",
                pointer,
                "input value is missing",
            )),
        },
        Unary {
            operation: op,
            operand,
        } => operation::unary(*op, get(values, *operand, pointer)?, pointer),
        Binary {
            operation: op,
            left,
            right,
        } => operation::binary(
            *op,
            get(values, *left, pointer)?,
            get(values, *right, pointer)?,
            pointer,
        ),
        Compare {
            operation: op,
            left,
            right,
        } => operation::compare(
            *op,
            get(values, *left, pointer)?,
            get(values, *right, pointer)?,
            pointer,
        ),
        Select {
            condition,
            when_true,
            when_false,
        } => match get(values, *condition, pointer)? {
            TemporalValue::Boolean { value: true } => Ok(get(values, *when_true, pointer)?.clone()),
            TemporalValue::Boolean { value: false } => {
                Ok(get(values, *when_false, pointer)?.clone())
            }
            _ => Err(contract(pointer)),
        },
        CurveSample { input, keys } => curve::sample(get(values, *input, pointer)?, keys, pointer),
        ComposeVec2 { x, y } => operation::compose_vec2(
            get(values, *x, pointer)?,
            get(values, *y, pointer)?,
            pointer,
        ),
        ProjectVec2 { value, axis } => {
            operation::project_vec2(get(values, *value, pointer)?, *axis, pointer)
        }
        ComposePoint { x, y } => operation::compose_point(
            get(values, *x, pointer)?,
            get(values, *y, pointer)?,
            pointer,
        ),
        ProjectPoint { value, axis } => {
            operation::project_point(get(values, *value, pointer)?, *axis, pointer)
        }
        ComposeRect {
            x,
            y,
            width,
            height,
        } => operation::compose_rect(
            get(values, *x, pointer)?,
            get(values, *y, pointer)?,
            get(values, *width, pointer)?,
            get(values, *height, pointer)?,
            pointer,
        ),
        ProjectRect { value, field } => {
            operation::project_rect(get(values, *value, pointer)?, *field, pointer)
        }
        ComposeColor {
            red,
            green,
            blue,
            alpha,
        } => operation::compose_color(
            get(values, *red, pointer)?,
            get(values, *green, pointer)?,
            get(values, *blue, pointer)?,
            get(values, *alpha, pointer)?,
            pointer,
        ),
        ProjectColor { value, channel } => {
            operation::project_color(get(values, *value, pointer)?, *channel, pointer)
        }
    }
}

fn get<'a>(
    values: &'a [Option<TemporalValue>],
    id: TemporalNodeId,
    pointer: &str,
) -> Result<&'a TemporalValue, TemporalEvaluationError> {
    let index = match usize::try_from(id.get()) {
        Ok(index) => index,
        Err(_) => return Err(contract(pointer)),
    };
    match values.get(index).and_then(Option::as_ref) {
        Some(value) => Ok(value),
        None => Err(contract(pointer)),
    }
}

fn contract(pointer: &str) -> TemporalEvaluationError {
    TemporalEvaluationError::new(
        "TEMPORAL_EVALUATION_CONTRACT",
        pointer,
        "verified node has incompatible values",
    )
}
