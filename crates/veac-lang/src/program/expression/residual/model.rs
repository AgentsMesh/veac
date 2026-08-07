use std::collections::BTreeMap;

use super::super::{CoreBuildInputId, CoreTemporalInputIdentity, InputId, Value, ValueType};
use veac_ir::{
    TemporalInputId, TemporalNodeId, TemporalProgram, TemporalProgramId, TemporalProvenance,
    TemporalType, TemporalValue,
};

pub type ResidualBuildBindings = BTreeMap<CoreBuildInputId, Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum ResidualRuntimeValue {
    Concrete(Value),
    TemporalConstant(TemporalValue),
    Sequence {
        value_type: ValueType,
        values: Vec<ResidualRuntimeValue>,
    },
    Residual(ResidualValue),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResidualValue {
    pub(crate) node_id: TemporalNodeId,
    pub(crate) value_type: TemporalType,
}

impl ResidualValue {
    pub const fn node_id(self) -> TemporalNodeId {
        self.node_id
    }

    pub const fn value_type(self) -> TemporalType {
        self.value_type
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidualInput {
    pub(crate) core_input_id: InputId,
    pub(crate) temporal_input_id: TemporalInputId,
    pub(crate) identity: CoreTemporalInputIdentity,
}

impl ResidualInput {
    pub const fn core_input_id(&self) -> InputId {
        self.core_input_id
    }

    pub const fn temporal_input_id(&self) -> TemporalInputId {
        self.temporal_input_id
    }

    pub const fn identity(&self) -> &CoreTemporalInputIdentity {
        &self.identity
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResidualizationLimits {
    pub max_steps: usize,
    pub max_nodes: usize,
    pub max_value_bytes: usize,
}

impl Default for ResidualizationLimits {
    fn default() -> Self {
        Self {
            max_steps: super::super::execution_budget::MAX_EXECUTION_FUEL,
            max_nodes: veac_ir::MAX_TEMPORAL_NODES_PER_PROGRAM,
            max_value_bytes: veac_ir::MAX_TEMPORAL_VALUE_BYTES,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResidualizationRequest {
    pub program_id: TemporalProgramId,
    pub provenance: TemporalProvenance,
    pub limits: ResidualizationLimits,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResidualizedExpression {
    pub(crate) value: ResidualRuntimeValue,
    pub(crate) program: Option<TemporalProgram>,
    pub(crate) inputs: Vec<ResidualInput>,
    pub(crate) provenance: TemporalProvenance,
}

impl ResidualizedExpression {
    pub const fn value(&self) -> &ResidualRuntimeValue {
        &self.value
    }

    pub const fn program(&self) -> Option<&TemporalProgram> {
        self.program.as_ref()
    }

    pub fn inputs(&self) -> &[ResidualInput] {
        &self.inputs
    }

    pub const fn provenance(&self) -> &TemporalProvenance {
        &self.provenance
    }
}
