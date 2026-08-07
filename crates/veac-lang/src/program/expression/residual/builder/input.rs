use std::ops::Range;

use super::Builder;
use crate::program::expression::{
    CoreInput, CoreTemporalInputIdentity, InputId, ResidualRuntimeValue, ResidualizationError,
};
use veac_ir::{TemporalInputDeclaration, TemporalInputId, TemporalNodeKind};

impl Builder<'_> {
    pub(in crate::program::expression::residual) fn input(
        &mut self,
        input: &CoreInput,
        identity: &CoreTemporalInputIdentity,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        self.input_identity(input.id(), identity, input.span())
    }

    pub(in crate::program::expression::residual) fn synthetic_input(
        &mut self,
        id: InputId,
        identity: &CoreTemporalInputIdentity,
        span: Range<usize>,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        self.input_identity(id, identity, span)
    }

    fn input_identity(
        &mut self,
        id: InputId,
        identity: &CoreTemporalInputIdentity,
        span: Range<usize>,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        if let Some(value) = self.input_nodes.get(&id).copied() {
            return Ok(ResidualRuntimeValue::Residual(value));
        }
        if self.inputs.len() >= veac_ir::MAX_TEMPORAL_INPUTS {
            return Err(ResidualizationError::new(
                "RESIDUAL_INPUT_LIMIT",
                "temporal input limit exceeded",
                span,
            ));
        }
        let source = super::super::input::source(identity);
        if let Some(previous) = self.sources.insert(source.clone(), id) {
            return Err(ResidualizationError::new(
                "RESIDUAL_INPUT_SOURCE_DUPLICATE",
                format!(
                    "Core inputs {} and {} map to one Temporal source",
                    previous.value(),
                    id.value()
                ),
                span,
            ));
        }
        let temporal_input_id = TemporalInputId::new(self.inputs.len() as u32);
        let value_type = identity.temporal_type();
        self.inputs.push(TemporalInputDeclaration {
            id: temporal_input_id,
            value_type,
            source,
        });
        self.manifest.push(super::ResidualInput {
            core_input_id: id,
            temporal_input_id,
            identity: identity.clone(),
        });
        let value = self.node(
            value_type,
            TemporalNodeKind::Input {
                input_id: temporal_input_id,
            },
            span,
        )?;
        self.input_nodes.insert(id, value);
        Ok(ResidualRuntimeValue::Residual(value))
    }
}
