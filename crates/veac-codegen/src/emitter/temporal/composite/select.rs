use std::collections::BTreeMap;

use super::super::budget::{Budget, MAX_TEXT_CHOICES};
use super::super::error::TemporalBackendError;
use super::super::expression;
use super::super::value::{
    number, CompiledValue as Value, Expression, LengthExpression, TextChoice,
};

pub(in crate::emitter::temporal) fn compile(
    condition: &Value,
    when_true: &Value,
    when_false: &Value,
    budget: &mut Budget<'_>,
) -> Result<Value, TemporalBackendError> {
    let Value::Boolean(condition) = condition else {
        return Err(budget.contract("select condition is not boolean"));
    };
    use Value::*;
    Ok(match (when_true, when_false) {
        (Boolean(a), Boolean(b)) => Boolean(choice(condition, a, b, budget)?),
        (Integer(a), Integer(b)) => Integer(choice(condition, a, b, budget)?),
        (Scalar(a), Scalar(b)) => Scalar(choice(condition, a, b, budget)?),
        (Time(a), Time(b)) => Time(choice(condition, a, b, budget)?),
        (Length(a), Length(b)) => Length(length_choice(condition, a, b, budget)?),
        (Angle(a), Angle(b)) => Angle(choice(condition, a, b, budget)?),
        (Vec2(ax, ay), Vec2(bx, by)) => Vec2(
            choice(condition, ax, bx, budget)?,
            choice(condition, ay, by, budget)?,
        ),
        (Point(ax, ay), Point(bx, by)) => Point(
            length_choice(condition, ax, bx, budget)?,
            length_choice(condition, ay, by, budget)?,
        ),
        (Rect(a), Rect(b)) => Rect(components(condition, a, b, budget)?),
        (Color(a), Color(b)) => Color(components(condition, a, b, budget)?),
        (Text(a), Text(b)) => Text(text_choices(condition, a, b, budget)?),
        _ => return Err(budget.contract("select branches are incompatible")),
    })
}

fn choice(
    condition: &Expression,
    a: &Expression,
    b: &Expression,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    expression::select(budget, condition, a, b)
}

fn components(
    condition: &Expression,
    a: &[Expression; 4],
    b: &[Expression; 4],
    budget: &mut Budget<'_>,
) -> Result<[Expression; 4], TemporalBackendError> {
    let values = a
        .iter()
        .zip(b)
        .map(|(a, b)| choice(condition, a, b, budget))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(values.try_into().expect("four components"))
}

fn length_choice(
    condition: &Expression,
    a: &LengthExpression,
    b: &LengthExpression,
    budget: &mut Budget<'_>,
) -> Result<LengthExpression, TemporalBackendError> {
    if a.kind != b.kind {
        return Err(budget.unsupported("dynamic length unit selection is unsupported"));
    }
    Ok(LengthExpression {
        value: choice(condition, &a.value, &b.value, budget)?,
        kind: a.kind,
    })
}

fn text_choices(
    condition: &Expression,
    a: &[TextChoice],
    b: &[TextChoice],
    budget: &mut Budget<'_>,
) -> Result<Vec<TextChoice>, TemporalBackendError> {
    let zero = number(budget, 0.0)?;
    let inverse = expression::call2(budget, "eq", condition, &zero)?;
    let mut grouped: BTreeMap<&str, Vec<Expression>> = BTreeMap::new();
    for (gate, choices) in [(condition, a), (&inverse, b)] {
        for item in choices {
            grouped
                .entry(&item.value)
                .or_default()
                .push(expression::infix(budget, gate, "*", &item.condition)?);
        }
    }
    if grouped.len() > MAX_TEXT_CHOICES {
        return Err(budget.unsupported("dynamic text choice limit exceeded"));
    }
    grouped
        .into_iter()
        .map(|(value, conditions)| {
            let condition = conditions
                .into_iter()
                .try_fold(zero.clone(), |left, right| {
                    expression::infix(budget, &left, "+", &right)
                })?;
            Ok(TextChoice {
                condition,
                value: value.into(),
            })
        })
        .collect()
}
