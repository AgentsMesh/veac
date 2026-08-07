use std::collections::BTreeMap;

use veac_plan::canonical::{TemporalNode, TemporalNodeId, TemporalNodeKind};

use super::super::budget::Budget;
use super::super::error::TemporalBackendError;
use super::super::value::CompiledValue as Value;
use super::super::{compare, composite, curve, operation};

pub(super) fn compile(
    node: &TemporalNode,
    values: &[Value],
    inputs: &BTreeMap<u32, Value>,
    budget: &mut Budget<'_>,
) -> Result<Value, TemporalBackendError> {
    use TemporalNodeKind::*;
    match &node.kind {
        Literal { value } => Value::literal(value, budget),
        Input { input_id } => inputs
            .get(&input_id.get())
            .cloned()
            .ok_or_else(|| budget.contract("temporal input is absent")),
        Unary {
            operation: value,
            operand,
        } => operation::unary(*value, get(values, *operand, budget)?, budget),
        Binary {
            operation: value,
            left,
            right,
        } => operation::binary(
            *value,
            get(values, *left, budget)?,
            get(values, *right, budget)?,
            budget,
        ),
        Compare {
            operation: value,
            left,
            right,
        } => compare::compile(
            *value,
            get(values, *left, budget)?,
            get(values, *right, budget)?,
            budget,
        ),
        Select {
            condition,
            when_true,
            when_false,
        } => composite::select(
            get(values, *condition, budget)?,
            get(values, *when_true, budget)?,
            get(values, *when_false, budget)?,
            budget,
        ),
        CurveSample { input, keys } => curve::compile(get(values, *input, budget)?, keys, budget),
        ComposeVec2 { x, y } => {
            composite::compose_vec2(get(values, *x, budget)?, get(values, *y, budget)?, budget)
        }
        ProjectVec2 { value, axis } => {
            composite::project_vec2(get(values, *value, budget)?, *axis, budget)
        }
        ComposePoint { x, y } => {
            composite::compose_point(get(values, *x, budget)?, get(values, *y, budget)?, budget)
        }
        ProjectPoint { value, axis } => {
            composite::project_point(get(values, *value, budget)?, *axis, budget)
        }
        ComposeRect {
            x,
            y,
            width,
            height,
        } => composite::compose_rect(
            [
                get(values, *x, budget)?,
                get(values, *y, budget)?,
                get(values, *width, budget)?,
                get(values, *height, budget)?,
            ],
            budget,
        ),
        ProjectRect { value, field } => {
            composite::project_rect(get(values, *value, budget)?, *field, budget)
        }
        ComposeColor {
            red,
            green,
            blue,
            alpha,
        } => composite::compose_color(
            [
                get(values, *red, budget)?,
                get(values, *green, budget)?,
                get(values, *blue, budget)?,
                get(values, *alpha, budget)?,
            ],
            budget,
        ),
        ProjectColor { value, channel } => {
            composite::project_color(get(values, *value, budget)?, *channel, budget)
        }
    }
}

fn get<'a>(
    values: &'a [Value],
    id: TemporalNodeId,
    budget: &Budget<'_>,
) -> Result<&'a Value, TemporalBackendError> {
    values
        .get(id.get() as usize)
        .ok_or_else(|| budget.contract("temporal operand is absent"))
}
