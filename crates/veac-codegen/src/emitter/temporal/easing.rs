use veac_plan::canonical::Interpolation;

use super::budget::Budget;
use super::error::TemporalBackendError;
use super::expression;
use super::value::{number, Expression};

pub(super) fn compile(
    progress: &Expression,
    interpolation: &Interpolation,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    match interpolation {
        Interpolation::Hold => number(budget, 0.0),
        Interpolation::Linear => Ok(progress.clone()),
        Interpolation::EaseIn => {
            let two = number(budget, 2.0)?;
            expression::call2(budget, "pow", progress, &two)
        }
        Interpolation::EaseOut => ease_out(progress, budget),
        Interpolation::EaseInOut => smoothstep(progress, budget),
        Interpolation::Spring { decay, .. } => spring(progress, *decay, interpolation, budget),
        Interpolation::CubicBezier { x1, y1, x2, y2 } => {
            cubic_bezier(progress, *x1, *y1, *x2, *y2, budget)
        }
    }
}

fn ease_out(
    progress: &Expression,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let one = number(budget, 1.0)?;
    let two = number(budget, 2.0)?;
    let remaining = expression::infix(budget, &one, "-", progress)?;
    let square = expression::call2(budget, "pow", &remaining, &two)?;
    expression::infix(budget, &one, "-", &square)
}

fn smoothstep(
    progress: &Expression,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let two = number(budget, 2.0)?;
    let three = number(budget, 3.0)?;
    let square = expression::call2(budget, "pow", progress, &two)?;
    let twice = expression::infix(budget, &two, "*", progress)?;
    let factor = expression::infix(budget, &three, "-", &twice)?;
    expression::infix(budget, &square, "*", &factor)
}

fn spring(
    progress: &Expression,
    decay: f64,
    interpolation: &Interpolation,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let value = interpolation
        .spring_coefficients()
        .ok_or_else(|| budget.contract("spring coefficients are invalid"))?;
    let decay = number(budget, decay)?;
    let frequency = number(budget, value.angular_frequency)?;
    let equilibrium = number(budget, value.equilibrium)?;
    let sine = number(budget, value.sine)?;
    budget.expression(&[
        "(",
        &equilibrium,
        ")+exp(-",
        &decay,
        "*(",
        progress,
        "))*(-(",
        &equilibrium,
        ")*cos(",
        &frequency,
        "*(",
        progress,
        "))+(",
        &sine,
        ")*sin(",
        &frequency,
        "*(",
        progress,
        ")))",
    ])
}

fn cubic_bezier(
    progress: &Expression,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let parameter: Expression = "ld(0)".into();
    let x = cubic(&parameter, x1, x2, budget)?;
    let root = budget.expression(&["root((", &x, ")-(", progress, ")\\,1)"])?;
    let solved: Expression = "ld(1)".into();
    let y = cubic(&solved, y1, y2, budget)?;
    budget.expression(&["st(1\\,", &root, ");", &y])
}

fn cubic(
    parameter: &Expression,
    first: f64,
    second: f64,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let first = number(budget, first)?;
    let second = number(budget, second)?;
    budget.expression(&[
        "3*(1-(", parameter, "))*(1-(", parameter, "))*(", parameter, ")*", &first, "+3*(1-(",
        parameter, "))*(", parameter, ")*(", parameter, ")*", &second, "+(", parameter, ")*(",
        parameter, ")*(", parameter, ")",
    ])
}
