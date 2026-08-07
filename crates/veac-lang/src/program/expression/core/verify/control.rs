use std::collections::BTreeSet;

use super::error;
use crate::program::expression::core::{CoreProgram, CoreTerminator};
use crate::program::expression::{ExpressionError, MAX_EXPRESSION_NODES};

mod graph;

pub(super) struct ControlFlow {
    dominators: Vec<BTreeSet<usize>>,
    reachable_from: Vec<BTreeSet<usize>>,
    predecessors: Vec<Vec<usize>>,
}

impl ControlFlow {
    pub(super) fn analyze(program: &CoreProgram) -> Result<Self, ExpressionError> {
        if program.blocks.is_empty() || program.entry.index() != Some(0) {
            return Err(error("Core entry must be block 0", 0..0));
        }
        if !program.blocks[0].parameters.is_empty() {
            return Err(error("Core entry block cannot have parameters", 0..0));
        }
        if program.blocks.len() > MAX_EXPRESSION_NODES.saturating_mul(4) {
            return Err(error("Core program contains too many blocks", 0..0));
        }
        let successors = graph::successors(program)?;
        let entry_reachable = graph::reachable(&successors, 0);
        if entry_reachable.iter().any(|value| !value) {
            return Err(error("every Core block must be reachable", 0..0));
        }
        graph::reject_cycles(&successors)?;
        let predecessors = graph::predecessors(&successors);
        let reachable_from = (0..successors.len())
            .map(|entry| {
                graph::reachable(&successors, entry)
                    .into_iter()
                    .enumerate()
                    .filter_map(|(index, seen)| seen.then_some(index))
                    .collect()
            })
            .collect();
        Ok(Self {
            dominators: graph::dominators(&predecessors),
            reachable_from,
            predecessors,
        })
    }

    pub(super) fn dominates(&self, definition: usize, usage: usize) -> bool {
        self.dominators[usage].contains(&definition)
    }

    pub(super) fn exclusive_predecessor(&self, target: usize, source: usize) -> bool {
        self.predecessors
            .get(target)
            .is_some_and(|values| values.as_slice() == [source])
    }

    pub(super) fn converging_conditions(
        &self,
        program: &CoreProgram,
        target: usize,
    ) -> Vec<crate::program::expression::core::ValueId> {
        program
            .blocks
            .iter()
            .enumerate()
            .filter_map(|(source, block)| {
                let (condition, targets) = match &block.terminator {
                    CoreTerminator::Branch {
                        condition,
                        then_target,
                        else_target,
                        ..
                    } => (*condition, vec![then_target.index()?, else_target.index()?]),
                    CoreTerminator::Match {
                        scrutinee, arms, ..
                    } => (
                        *scrutinee,
                        arms.iter()
                            .map(|arm| arm.target().index())
                            .collect::<Option<Vec<_>>>()?,
                    ),
                    _ => return None,
                };
                let all_reach_target = targets
                    .iter()
                    .all(|entry| self.reachable_from[*entry].contains(&target));
                let earlier_common = (0..program.blocks.len()).any(|candidate| {
                    candidate != target
                        && targets
                            .iter()
                            .all(|entry| self.reachable_from[*entry].contains(&candidate))
                        && self.reachable_from[candidate].contains(&target)
                });
                (self.dominates(source, target) && all_reach_target && !earlier_common)
                    .then_some(condition)
            })
            .collect()
    }
}
