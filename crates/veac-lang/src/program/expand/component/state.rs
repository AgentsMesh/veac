use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;
use crate::program::model::ComponentKey;

use super::super::budget;

const MAX_COMPONENT_DEPTH: usize = 64;
const MAX_COMPONENT_NODES: usize = 16_384;
const MAX_PARAMETER_EVALUATIONS: usize = 65_536;

#[derive(Default)]
pub(crate) struct State {
    active: Vec<ComponentKey>,
    nodes: usize,
    parameter_evaluations: usize,
    retained_frame_bytes: usize,
}

impl State {
    pub(super) fn enter(
        &mut self,
        key: &ComponentKey,
        path: &str,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if let Some(start) = self.active.iter().position(|value| value == key) {
            let mut chain = self.active[start..].iter().map(label).collect::<Vec<_>>();
            chain.push(label(key));
            return Err(Diagnostic::new(
                "PROGRAM_COMPONENT_CYCLE",
                path,
                format!("component composition cycle: {}", chain.join(" -> ")),
                span,
            ));
        }
        if self.active.len() >= MAX_COMPONENT_DEPTH {
            return Err(Diagnostic::new(
                "PROGRAM_COMPONENT_DEPTH",
                path,
                format!("component composition exceeds {MAX_COMPONENT_DEPTH} levels"),
                span,
            ));
        }
        if self.nodes >= MAX_COMPONENT_NODES {
            return Err(Diagnostic::new(
                "PROGRAM_COMPONENT_BUDGET",
                path,
                format!("component expansion exceeds {MAX_COMPONENT_NODES} instances"),
                span,
            ));
        }
        self.nodes += 1;
        self.active.push(key.clone());
        Ok(())
    }

    pub(super) fn leave(&mut self) {
        self.active.pop();
    }

    pub(in crate::program::expand) fn nodes(&self) -> usize {
        self.nodes
    }

    pub(super) fn charge_parameter_work(
        &mut self,
        path: &str,
        span: Span,
        count: usize,
    ) -> Result<(), Diagnostic> {
        let next = self
            .parameter_evaluations
            .checked_add(count)
            .ok_or_else(|| parameter_budget(path, span))?;
        if next > MAX_PARAMETER_EVALUATIONS {
            return Err(parameter_budget(path, span));
        }
        self.parameter_evaluations = next;
        Ok(())
    }

    pub(super) fn retain_frame(&mut self, path: &str, bytes: usize) -> Result<(), Diagnostic> {
        let next = budget::checked_source_add(path, self.retained_frame_bytes, bytes)?;
        budget::ensure_source(path, next, budget::MAX_EXPANDED_BYTES)?;
        self.retained_frame_bytes = next;
        Ok(())
    }

    pub(super) fn remaining_frame_bytes(&self) -> usize {
        budget::MAX_EXPANDED_BYTES - self.retained_frame_bytes
    }

    pub(super) fn retained_frame_bytes(&self) -> usize {
        self.retained_frame_bytes
    }

    pub(super) fn release_frame(&mut self, bytes: usize) {
        self.retained_frame_bytes = self
            .retained_frame_bytes
            .checked_sub(bytes)
            .expect("retained component-frame accounting must balance");
    }
}

fn label(key: &ComponentKey) -> String {
    format!("{}::{}", key.path, key.name)
}

fn parameter_budget(path: &str, span: Span) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_PARAMETER_BUDGET",
        path,
        format!("component expansion exceeds {MAX_PARAMETER_EVALUATIONS} parameter evaluations"),
        span,
    )
}

#[cfg(test)]
mod tests;
