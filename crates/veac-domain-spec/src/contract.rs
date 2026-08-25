use std::sync::Arc;

use veac_lang_model::{Effect, Stage};

use super::{DomainOperationId, DomainType, DomainValueShape};

mod semantics;
mod temporal;
pub use temporal::TemporalLoweringOpcode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandAxis {
    Topology,
    Leaf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainInstructionKind {
    DomainConstruct,
    GraphEmit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DomainRuntimeAction {
    Description,
    EntityConstructor,
    OwnedAttachment,
    NonOwningUpdate,
    ProjectEntry,
    RelationConstructor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainOperandContract {
    name: Arc<str>,
    shape: DomainValueShape,
    axis: OperandAxis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainOperationContract {
    id: DomainOperationId,
    name: Arc<str>,
    exposure: DomainOperationExposure,
    operands: Arc<[DomainOperandContract]>,
    result: DomainValueShape,
    semantics: DomainOperationSemantics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DomainOperationSemantics {
    instruction: DomainInstructionKind,
    runtime_action: DomainRuntimeAction,
    effect: Effect,
    max_stage: Stage,
    temporal_lowering: Option<TemporalLoweringOpcode>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DomainOperationExposure {
    FreeFunction {
        name: Arc<str>,
    },
    Method {
        receiver: DomainType,
        name: Arc<str>,
    },
}

impl DomainOperandContract {
    pub(super) fn new(
        name: impl Into<Arc<str>>,
        shape: DomainValueShape,
        axis: OperandAxis,
    ) -> Self {
        Self {
            name: name.into(),
            shape,
            axis,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn shape(&self) -> DomainValueShape {
        self.shape
    }

    pub const fn axis(&self) -> OperandAxis {
        self.axis
    }
}

impl DomainOperationContract {
    pub(super) fn new(
        id: DomainOperationId,
        name: impl Into<Arc<str>>,
        exposure: DomainOperationExposure,
        operands: impl Into<Arc<[DomainOperandContract]>>,
        result: DomainValueShape,
        semantics: DomainOperationSemantics,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            exposure,
            operands: operands.into(),
            result,
            semantics,
        }
    }

    pub const fn id(&self) -> DomainOperationId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn exposure(&self) -> &DomainOperationExposure {
        &self.exposure
    }

    pub fn operands(&self) -> &[DomainOperandContract] {
        &self.operands
    }

    pub const fn result(&self) -> DomainValueShape {
        self.result
    }

    pub const fn instruction(&self) -> DomainInstructionKind {
        self.semantics.instruction
    }

    pub const fn runtime_action(&self) -> DomainRuntimeAction {
        self.semantics.runtime_action
    }

    pub const fn effect(&self) -> Effect {
        self.semantics.effect
    }

    pub const fn max_stage(&self) -> Stage {
        self.semantics.max_stage
    }

    pub const fn temporal_lowering(&self) -> Option<TemporalLoweringOpcode> {
        self.semantics.temporal_lowering
    }
}

impl DomainOperationExposure {
    pub(super) fn free_function(name: impl Into<Arc<str>>) -> Self {
        Self::FreeFunction { name: name.into() }
    }

    pub(super) fn method(receiver: DomainType, name: impl Into<Arc<str>>) -> Self {
        Self::Method {
            receiver,
            name: name.into(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::FreeFunction { name } | Self::Method { name, .. } => name,
        }
    }

    pub const fn receiver(&self) -> Option<DomainType> {
        match self {
            Self::FreeFunction { .. } => None,
            Self::Method { receiver, .. } => Some(*receiver),
        }
    }
}

#[cfg(test)]
#[path = "contract/tests.rs"]
mod tests;
