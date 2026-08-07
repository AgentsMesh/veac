use std::collections::BTreeMap;
use std::ops::Range;

use super::budget::Budget;
use super::convert;
use super::{
    ResidualInput, ResidualRuntimeValue, ResidualValue, ResidualizationError,
    ResidualizationRequest, ResidualizedExpression,
};
use crate::program::expression::InputId;
use veac_ir::{
    temporal_program_digest, TemporalInputDeclaration, TemporalInputSource, TemporalNode,
    TemporalNodeId, TemporalNodeKind, TemporalProgram, TemporalProvenanceId, TemporalType,
    TEMPORAL_OPSET_VERSION,
};

mod input;
mod validation;

pub(super) struct Builder<'a> {
    pub(super) budget: Budget<'a>,
    nodes: Vec<TemporalNode>,
    inputs: Vec<TemporalInputDeclaration>,
    manifest: Vec<ResidualInput>,
    input_nodes: BTreeMap<InputId, ResidualValue>,
    sources: BTreeMap<TemporalInputSource, InputId>,
    provenance_id: TemporalProvenanceId,
}

impl<'a> Builder<'a> {
    pub(super) fn new(
        request: &ResidualizationRequest,
        ledger: crate::program::expression::ResidualLedger<'a>,
    ) -> Self {
        Self {
            budget: Budget::new(request.limits, ledger),
            nodes: Vec::new(),
            inputs: Vec::new(),
            manifest: Vec::new(),
            input_nodes: BTreeMap::new(),
            sources: BTreeMap::new(),
            provenance_id: request.provenance.id.clone(),
        }
    }

    pub(super) fn node(
        &mut self,
        value_type: TemporalType,
        kind: TemporalNodeKind,
        span: Range<usize>,
    ) -> Result<ResidualValue, ResidualizationError> {
        self.budget.node(span)?;
        let node_id = TemporalNodeId::new(self.nodes.len() as u32);
        self.nodes.push(TemporalNode {
            id: node_id,
            value_type,
            kind,
            provenance_id: Some(self.provenance_id.clone()),
        });
        Ok(ResidualValue {
            node_id,
            value_type,
        })
    }

    pub(super) fn as_node(
        &mut self,
        value: ResidualRuntimeValue,
        span: Range<usize>,
    ) -> Result<ResidualValue, ResidualizationError> {
        match value {
            ResidualRuntimeValue::Residual(value) => Ok(value),
            ResidualRuntimeValue::TemporalConstant(value) => {
                let value_type = value.value_type();
                self.budget.value(value.logical_bytes(), span.clone())?;
                self.node(value_type, TemporalNodeKind::Literal { value }, span)
            }
            ResidualRuntimeValue::Concrete(value) => {
                let (value_type, value) = convert::temporal_value(&value, span.clone())?;
                self.budget.value(value.logical_bytes(), span.clone())?;
                self.node(value_type, TemporalNodeKind::Literal { value }, span)
            }
            ResidualRuntimeValue::Sequence { .. } => Err(ResidualizationError::new(
                "RESIDUAL_VALUE_UNSUPPORTED",
                "structural values cannot become Temporal nodes",
                span,
            )),
        }
    }

    pub(super) fn finish(
        mut self,
        mut value: ResidualRuntimeValue,
        request: ResidualizationRequest,
    ) -> Result<ResidualizedExpression, ResidualizationError> {
        if matches!(value, ResidualRuntimeValue::TemporalConstant(_)) {
            value = ResidualRuntimeValue::Residual(self.as_node(value, 0..0)?);
        }
        if matches!(value, ResidualRuntimeValue::Sequence { .. }) {
            return Err(ResidualizationError::new(
                "RESIDUAL_RESULT_STRUCTURAL",
                "a Temporal expression cannot return a structural value",
                0..0,
            ));
        }
        let ResidualRuntimeValue::Residual(result) = value else {
            validation::library(None, &request)?;
            return Ok(ResidualizedExpression {
                value,
                program: None,
                inputs: Vec::new(),
                provenance: request.provenance,
            });
        };
        let mut program = TemporalProgram {
            id: request.program_id.clone(),
            opset_version: TEMPORAL_OPSET_VERSION,
            inputs: self.inputs,
            result_type: result.value_type,
            nodes: self.nodes,
            result: result.node_id,
            content_sha256: String::new(),
            provenance_id: request.provenance.id.clone(),
        };
        program.content_sha256 = temporal_program_digest(&program).map_err(|error| {
            ResidualizationError::new("RESIDUAL_TEMPORAL_DIGEST", error.to_string(), 0..0)
        })?;
        validation::library(Some(&program), &request)?;
        Ok(ResidualizedExpression {
            value: ResidualRuntimeValue::Residual(result),
            program: Some(program),
            inputs: self.manifest,
            provenance: request.provenance,
        })
    }
}
