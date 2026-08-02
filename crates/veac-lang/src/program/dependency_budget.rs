use crate::authoring::Span;

use super::diagnostic::Diagnostic;

pub(crate) const MAX_DEPENDENCY_DEPTH: usize = 64;
pub(crate) const MAX_DEPENDENCY_NODES: usize = 16_384;

#[derive(Default)]
pub(crate) struct DependencyBudget {
    active: usize,
    nodes: usize,
}

impl DependencyBudget {
    pub(crate) fn enter(
        &mut self,
        path: &str,
        dependency: &str,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if self.active >= MAX_DEPENDENCY_DEPTH {
            return Err(Diagnostic::new(
                "PROGRAM_DEPENDENCY_DEPTH",
                path,
                format!(
                    "{dependency} dependency graph exceeds {MAX_DEPENDENCY_DEPTH} active levels"
                ),
                span,
            ));
        }
        if self.nodes >= MAX_DEPENDENCY_NODES {
            return Err(Diagnostic::new(
                "PROGRAM_DEPENDENCY_BUDGET",
                path,
                format!(
                    "{dependency} dependency graph exceeds {MAX_DEPENDENCY_NODES} resolved nodes"
                ),
                span,
            ));
        }
        self.active += 1;
        self.nodes += 1;
        Ok(())
    }

    pub(crate) fn leave(&mut self) {
        debug_assert!(self.active > 0);
        self.active -= 1;
    }
}

#[cfg(test)]
#[path = "dependency_budget/tests.rs"]
mod tests;
