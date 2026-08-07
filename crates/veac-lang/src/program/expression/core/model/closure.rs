use std::ops::Range;

use super::super::{ClosureDefinitionId, CoreDigest, FunctionId, FunctionSummary};
use super::{CoreCallTarget, CoreInstructionKind, CoreProgram};
use crate::program::expression::{FunctionEffect, Stage, ValueType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreClosureDefinition {
    pub(crate) id: ClosureDefinitionId,
    pub(crate) parameter_types: Vec<ValueType>,
    pub(crate) parameter_stages: Vec<Stage>,
    pub(crate) capture_types: Vec<ValueType>,
    pub(crate) effect: FunctionEffect,
    pub(crate) non_escaping: bool,
    pub(crate) body: Box<CoreProgram>,
    pub(crate) summary: FunctionSummary,
    pub(crate) digest: CoreDigest,
    pub(crate) span: Range<usize>,
}

impl CoreClosureDefinition {
    pub fn id(&self) -> ClosureDefinitionId {
        self.id
    }

    pub fn parameter_types(&self) -> &[ValueType] {
        &self.parameter_types
    }

    pub fn parameter_stages(&self) -> &[Stage] {
        &self.parameter_stages
    }

    pub fn capture_types(&self) -> &[ValueType] {
        &self.capture_types
    }

    pub fn effect(&self) -> FunctionEffect {
        self.effect
    }

    pub fn non_escaping(&self) -> bool {
        self.non_escaping
    }

    pub fn body(&self) -> &CoreProgram {
        &self.body
    }

    pub fn summary(&self) -> &FunctionSummary {
        &self.summary
    }

    pub fn digest(&self) -> CoreDigest {
        self.digest
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }
}

impl CoreProgram {
    pub(crate) fn called_functions(&self) -> Vec<FunctionId> {
        let mut functions = self
            .blocks
            .iter()
            .flat_map(|block| &block.instructions)
            .filter_map(|instruction| match &instruction.kind {
                CoreInstructionKind::Call {
                    target: CoreCallTarget::User(id),
                    ..
                } => Some(*id),
                _ => None,
            })
            .collect::<Vec<_>>();
        for definition in &self.closure_definitions {
            functions.extend(definition.body.called_functions());
        }
        functions
    }
}
