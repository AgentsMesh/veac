use crate::{
    TemporalEvaluationError, TemporalNodeId, TemporalNodeKind, TemporalProgram, TemporalValue,
};

use super::{budget::Budget, input::PreparedInputs, node};

macro_rules! contract {
    ($pointer:expr) => {
        TemporalEvaluationError::new(
            "TEMPORAL_EVALUATION_CONTRACT",
            $pointer,
            "verified temporal graph is incomplete",
        )
    };
}

#[derive(Clone, Copy)]
enum Work {
    Enter(TemporalNodeId),
    Select(TemporalNodeId),
    Finish(TemporalNodeId),
}

pub(super) fn evaluate(
    program: &TemporalProgram,
    inputs: &PreparedInputs<'_>,
    budget: &mut Budget,
) -> Result<TemporalValue, TemporalEvaluationError> {
    let mut values = vec![None; program.nodes.len()];
    let mut work = vec![Work::Enter(program.result)];
    while let Some(item) = work.pop() {
        match item {
            Work::Enter(id) => enter(program, &values, &mut work, budget, id)?,
            Work::Select(id) => select(program, &values, &mut work, id)?,
            Work::Finish(id) => finish(program, inputs, &mut values, budget, id)?,
        }
    }
    let index = index(program.result, "/program/result")?;
    values[index].take().ok_or(contract!("/program/result"))
}

fn enter(
    program: &TemporalProgram,
    values: &[Option<TemporalValue>],
    work: &mut Vec<Work>,
    budget: &mut Budget,
    id: TemporalNodeId,
) -> Result<(), TemporalEvaluationError> {
    let index = index(id, "/program/nodes")?;
    if matches!(values.get(index), Some(Some(_))) {
        return Ok(());
    }
    let node = program
        .nodes
        .get(index)
        .ok_or(contract!("/program/nodes"))?;
    let pointer = format!("/program/nodes/{index}");
    budget.step(&pointer)?;
    if let TemporalNodeKind::Select { condition, .. } = &node.kind {
        work.push(Work::Select(id));
        work.push(Work::Enter(*condition));
    } else {
        work.push(Work::Finish(id));
        push_dependencies(&node.kind, work);
    }
    Ok(())
}

fn select(
    program: &TemporalProgram,
    values: &[Option<TemporalValue>],
    work: &mut Vec<Work>,
    id: TemporalNodeId,
) -> Result<(), TemporalEvaluationError> {
    let index = index(id, "/program/nodes")?;
    let TemporalNodeKind::Select {
        condition,
        when_true,
        when_false,
    } = &program.nodes[index].kind
    else {
        return Err(contract!("/program/nodes"));
    };
    let selected = match value(values, *condition, &format!("/program/nodes/{index}"))? {
        TemporalValue::Boolean { value: true } => *when_true,
        TemporalValue::Boolean { value: false } => *when_false,
        _ => return Err(contract!(format!("/program/nodes/{index}"))),
    };
    work.push(Work::Finish(id));
    work.push(Work::Enter(selected));
    Ok(())
}

fn finish(
    program: &TemporalProgram,
    inputs: &PreparedInputs<'_>,
    values: &mut [Option<TemporalValue>],
    budget: &mut Budget,
    id: TemporalNodeId,
) -> Result<(), TemporalEvaluationError> {
    let index = index(id, "/program/nodes")?;
    if values[index].is_some() {
        return Ok(());
    }
    let pointer = format!("/program/nodes/{index}");
    if let TemporalNodeKind::Input { input_id } = &program.nodes[index].kind {
        let input = inputs
            .get(&input_id.get())
            .copied()
            .ok_or(contract!(&pointer))?;
        budget.value(input, &pointer)?;
        values[index] = Some(input.clone());
        return Ok(());
    }
    let value = node::evaluate(&program.nodes[index].kind, values, inputs, &pointer)?;
    budget.value(&value, &pointer)?;
    values[index] = Some(value);
    Ok(())
}

fn push_dependencies(kind: &TemporalNodeKind, work: &mut Vec<Work>) {
    use TemporalNodeKind::*;
    match kind {
        Literal { .. } | Input { .. } | Select { .. } => {}
        Unary { operand, .. } => work.push(Work::Enter(*operand)),
        Binary { left, right, .. } | Compare { left, right, .. } => {
            work.push(Work::Enter(*right));
            work.push(Work::Enter(*left));
        }
        CurveSample { input, .. } => work.push(Work::Enter(*input)),
        ComposeVec2 { x, y } | ComposePoint { x, y } => {
            work.push(Work::Enter(*y));
            work.push(Work::Enter(*x));
        }
        ProjectVec2 { value, .. }
        | ProjectPoint { value, .. }
        | ProjectRect { value, .. }
        | ProjectColor { value, .. } => work.push(Work::Enter(*value)),
        ComposeRect {
            x,
            y,
            width,
            height,
        } => push_four(work, *x, *y, *width, *height),
        ComposeColor {
            red,
            green,
            blue,
            alpha,
        } => push_four(work, *red, *green, *blue, *alpha),
    }
}

fn push_four(
    work: &mut Vec<Work>,
    a: TemporalNodeId,
    b: TemporalNodeId,
    c: TemporalNodeId,
    d: TemporalNodeId,
) {
    for id in [d, c, b, a] {
        work.push(Work::Enter(id));
    }
}

fn value<'a>(
    values: &'a [Option<TemporalValue>],
    id: TemporalNodeId,
    pointer: &str,
) -> Result<&'a TemporalValue, TemporalEvaluationError> {
    match values.get(index(id, pointer)?) {
        Some(Some(value)) => Ok(value),
        _ => Err(contract!(pointer)),
    }
}

fn index(id: TemporalNodeId, pointer: &str) -> Result<usize, TemporalEvaluationError> {
    match usize::try_from(id.get()) {
        Ok(index) => Ok(index),
        Err(_) => Err(contract!(pointer)),
    }
}
