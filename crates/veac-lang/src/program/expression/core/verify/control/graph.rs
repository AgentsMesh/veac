use std::collections::BTreeSet;

use super::super::error;
use crate::program::expression::core::{CoreProgram, CoreTerminator};
use crate::program::expression::ExpressionError;

pub(super) fn successors(program: &CoreProgram) -> Result<Vec<Vec<usize>>, ExpressionError> {
    program
        .blocks
        .iter()
        .map(|block| {
            let targets = match &block.terminator {
                CoreTerminator::Return { .. } => Vec::new(),
                CoreTerminator::Jump { target, .. } => vec![*target],
                CoreTerminator::Branch {
                    then_target,
                    else_target,
                    span,
                    ..
                } => {
                    for target in [then_target, else_target] {
                        let index = target.index().filter(|index| *index < program.blocks.len());
                        if index.is_some_and(|index| !program.blocks[index].parameters.is_empty()) {
                            return Err(error(
                                "branch targets cannot have block parameters",
                                span.clone(),
                            ));
                        }
                    }
                    vec![*then_target, *else_target]
                }
                CoreTerminator::Match { arms, .. } => arms.iter().map(|arm| arm.target()).collect(),
                CoreTerminator::ForEach(value) => vec![value.continuation()],
            };
            targets
                .into_iter()
                .map(|target| {
                    target
                        .index()
                        .filter(|index| *index < program.blocks.len())
                        .ok_or_else(|| {
                            error(
                                "terminator targets an unknown block",
                                block.terminator.span().clone(),
                            )
                        })
                })
                .collect()
        })
        .collect()
}

pub(super) fn reachable(successors: &[Vec<usize>], entry: usize) -> Vec<bool> {
    let mut seen = vec![false; successors.len()];
    let mut pending = vec![entry];
    while let Some(block) = pending.pop() {
        if !seen[block] {
            seen[block] = true;
            pending.extend(successors[block].iter().copied());
        }
    }
    seen
}

pub(super) fn reject_cycles(successors: &[Vec<usize>]) -> Result<(), ExpressionError> {
    let mut marks = vec![0u8; successors.len()];
    fn visit(
        block: usize,
        successors: &[Vec<usize>],
        marks: &mut [u8],
    ) -> Result<(), ExpressionError> {
        match marks[block] {
            1 => return Err(error("Core control flow must be acyclic", 0..0)),
            2 => return Ok(()),
            _ => {}
        }
        marks[block] = 1;
        for target in &successors[block] {
            visit(*target, successors, marks)?;
        }
        marks[block] = 2;
        Ok(())
    }
    visit(0, successors, &mut marks)
}

pub(super) fn predecessors(successors: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let mut output = vec![Vec::new(); successors.len()];
    for (source, targets) in successors.iter().enumerate() {
        for target in targets {
            output[*target].push(source);
        }
    }
    output
}

pub(super) fn dominators(predecessors: &[Vec<usize>]) -> Vec<BTreeSet<usize>> {
    let all = (0..predecessors.len()).collect::<BTreeSet<_>>();
    let mut output = vec![all; predecessors.len()];
    output[0] = BTreeSet::from([0]);
    loop {
        let mut changed = false;
        for block in 1..predecessors.len() {
            let mut next = predecessors[block]
                .iter()
                .map(|predecessor| output[*predecessor].clone())
                .reduce(|left, right| left.intersection(&right).copied().collect())
                .unwrap_or_default();
            next.insert(block);
            if next != output[block] {
                output[block] = next;
                changed = true;
            }
        }
        if !changed {
            return output;
        }
    }
}
