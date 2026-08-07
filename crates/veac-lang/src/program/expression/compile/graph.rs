use std::collections::BTreeMap;
use std::ops::Range;

use super::super::core::{FunctionId, FunctionRegistry};
use super::super::{ExpressionError, MAX_FUNCTION_CALL_DEPTH};
use super::body::ResolvedBody;

mod walk;

#[derive(Debug, Clone)]
pub(super) struct CallSite {
    pub target: FunctionId,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    Visiting,
    Complete,
}

pub(super) fn order(
    imported: &FunctionRegistry,
    bodies: &[ResolvedBody],
) -> Result<Vec<usize>, ExpressionError> {
    let mut indices = BTreeMap::new();
    for (index, body) in bodies.iter().enumerate() {
        if indices.insert(body.id(), index).is_some() {
            return Err(body.decorate(ExpressionError::new(
                "EXPRESSION_FUNCTION_ID_COLLISION",
                format!("resolved function ID for `{}` is not unique", body.name()),
                body.typed().root.span.clone(),
            )));
        }
    }
    let calls = bodies
        .iter()
        .map(|body| checked_calls(imported, &indices, body))
        .collect::<Result<Vec<_>, _>>()?;
    let mut traversal = Traversal {
        bodies,
        indices: &indices,
        calls: &calls,
        marks: BTreeMap::new(),
        path: Vec::new(),
        output: Vec::with_capacity(bodies.len()),
    };
    for index in 0..bodies.len() {
        traversal.visit(index)?;
    }
    Ok(traversal.output)
}

fn checked_calls(
    imported: &FunctionRegistry,
    local: &BTreeMap<FunctionId, usize>,
    body: &ResolvedBody,
) -> Result<Vec<CallSite>, ExpressionError> {
    let calls = walk::calls(body.typed());
    for call in &calls {
        if !local.contains_key(&call.target) && imported.get(call.target).is_none() {
            return Err(body.decorate(ExpressionError::new(
                "EXPRESSION_UNCOMPILED_FUNCTION",
                format!(
                    "resolved call target {} has no verified function body",
                    call.target
                ),
                call.span.clone(),
            )));
        }
    }
    Ok(calls)
}

struct Traversal<'a> {
    bodies: &'a [ResolvedBody],
    indices: &'a BTreeMap<FunctionId, usize>,
    calls: &'a [Vec<CallSite>],
    marks: BTreeMap<FunctionId, Mark>,
    path: Vec<usize>,
    output: Vec<usize>,
}

impl Traversal<'_> {
    fn visit(&mut self, index: usize) -> Result<(), ExpressionError> {
        let body = &self.bodies[index];
        if self.marks.get(&body.id()) == Some(&Mark::Complete) {
            return Ok(());
        }
        self.marks.insert(body.id(), Mark::Visiting);
        self.path.push(index);
        for call in &self.calls[index] {
            let Some(target) = self.indices.get(&call.target).copied() else {
                continue;
            };
            if self.marks.get(&call.target) == Some(&Mark::Visiting) {
                return Err(self.cycle(target, call));
            }
            if self.path.len() >= MAX_FUNCTION_CALL_DEPTH {
                return Err(body.decorate(ExpressionError::new(
                    "EXPRESSION_CALL_DEPTH_LIMIT",
                    format!("function calls exceed the {MAX_FUNCTION_CALL_DEPTH} depth limit"),
                    call.span.clone(),
                )));
            }
            self.visit(target)?;
        }
        self.path.pop();
        self.marks.insert(body.id(), Mark::Complete);
        self.output.push(index);
        Ok(())
    }

    fn cycle(&self, target: usize, call: &CallSite) -> ExpressionError {
        let start = self
            .path
            .iter()
            .position(|index| *index == target)
            .unwrap_or(0);
        let mut names = self.path[start..]
            .iter()
            .map(|index| self.bodies[*index].name().to_owned())
            .collect::<Vec<_>>();
        names.push(self.bodies[target].name().to_owned());
        let current = &self.bodies[*self.path.last().expect("active DFS path")];
        current.decorate(ExpressionError::new(
            "EXPRESSION_FUNCTION_CYCLE",
            format!("function call cycle: {}", names.join(" -> ")),
            call.span.clone(),
        ))
    }
}

#[cfg(test)]
#[path = "graph/tests.rs"]
mod tests;
