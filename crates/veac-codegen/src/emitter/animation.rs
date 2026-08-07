mod keyframes;

use veac_plan::canonical::{Animatable, Length, LengthUnit, Point, Rect, Vec2};
use veac_plan::ResolvedRenderPlan;

use super::{process_owner::ProcessOwner, temporal, time};

#[cfg(test)]
#[path = "animation_tests.rs"]
mod tests;
#[cfg(test)]
use keyframes::easing;

pub(crate) fn number(
    plan: &ResolvedRenderPlan,
    owner: ProcessOwner<'_>,
    value: &Animatable<f64>,
    clock: &str,
) -> String {
    expression(
        plan,
        owner,
        value,
        clock,
        |value| time::number(*value),
        |value| value.number_expression(),
    )
}

pub(crate) fn vec_x(
    plan: &ResolvedRenderPlan,
    owner: ProcessOwner<'_>,
    value: &Animatable<Vec2>,
    clock: &str,
) -> String {
    expression(
        plan,
        owner,
        value,
        clock,
        |value| time::number(value.x),
        |value| value.vector_expression(0),
    )
}

pub(crate) fn vec_y(
    plan: &ResolvedRenderPlan,
    owner: ProcessOwner<'_>,
    value: &Animatable<Vec2>,
    clock: &str,
) -> String {
    expression(
        plan,
        owner,
        value,
        clock,
        |value| time::number(value.y),
        |value| value.vector_expression(1),
    )
}

pub(crate) fn point_x(
    plan: &ResolvedRenderPlan,
    owner: ProcessOwner<'_>,
    value: &Animatable<Point>,
    clock: &str,
    extent: &str,
) -> String {
    expression(
        plan,
        owner,
        value,
        clock,
        |value| length(value.x, extent),
        |value| value.point_expression(0, extent),
    )
}

pub(crate) fn point_y(
    plan: &ResolvedRenderPlan,
    owner: ProcessOwner<'_>,
    value: &Animatable<Point>,
    clock: &str,
    extent: &str,
) -> String {
    expression(
        plan,
        owner,
        value,
        clock,
        |value| length(value.y, extent),
        |value| value.point_expression(1, extent),
    )
}

pub(crate) fn rect_x(
    plan: &ResolvedRenderPlan,
    owner: ProcessOwner<'_>,
    value: &Animatable<Rect>,
    clock: &str,
) -> String {
    rect(plan, owner, value, clock, 0, |value| value.x)
}

pub(crate) fn rect_y(
    plan: &ResolvedRenderPlan,
    owner: ProcessOwner<'_>,
    value: &Animatable<Rect>,
    clock: &str,
) -> String {
    rect(plan, owner, value, clock, 1, |value| value.y)
}

pub(crate) fn rect_width(
    plan: &ResolvedRenderPlan,
    owner: ProcessOwner<'_>,
    value: &Animatable<Rect>,
    clock: &str,
) -> String {
    rect(plan, owner, value, clock, 2, |value| value.width)
}

pub(crate) fn rect_height(
    plan: &ResolvedRenderPlan,
    owner: ProcessOwner<'_>,
    value: &Animatable<Rect>,
    clock: &str,
) -> String {
    rect(plan, owner, value, clock, 3, |value| value.height)
}

fn expression<T>(
    plan: &ResolvedRenderPlan,
    owner: ProcessOwner<'_>,
    value: &Animatable<T>,
    clock: &str,
    render: impl Fn(&T) -> String,
    compiled: impl Fn(&temporal::CompiledValue) -> Option<String>,
) -> String {
    match value {
        Animatable::Constant { value } => render(value),
        Animatable::Keyframes { keyframes } => keyframes::curve(keyframes, clock, render),
        Animatable::Binding { binding_id } => compiled(
            &temporal::compile_binding(plan, binding_id, owner, clock)
                .expect("preflight compiles every temporal binding"),
        )
        .expect("plan validation matches temporal binding and animation sink types"),
    }
}

fn rect(
    plan: &ResolvedRenderPlan,
    owner: ProcessOwner<'_>,
    value: &Animatable<Rect>,
    clock: &str,
    index: usize,
    field: impl Fn(&Rect) -> f64,
) -> String {
    expression(
        plan,
        owner,
        value,
        clock,
        |value| time::number(field(value)),
        |value| value.rect_expression(index),
    )
}

fn length(value: Length, extent: &str) -> String {
    match value.unit {
        LengthUnit::Pixels => time::number(value.value),
        LengthUnit::Normalized => format!("{extent}*{}", time::number(value.value)),
        LengthUnit::Percent => format!("{extent}*{}/100", time::number(value.value)),
    }
}
