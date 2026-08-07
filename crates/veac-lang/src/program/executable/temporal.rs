use std::collections::BTreeMap;

use crate::program::expression::{
    CompiledExpression, ResidualBuildBindings, ResidualizationRequest,
};
use veac_ir::{TemporalBindingId, TemporalParameterId, TemporalValue};

mod identity;
pub(super) use identity::declared_inputs;
mod attached;
mod authored;
pub(super) use attached::{compile as compile_attached, AttachedTemporalLeaf};
pub(super) use authored::compile as compile_authored;
mod property;
pub(crate) use property::{ClipTemporalProperty, MaskTemporalProperty, TextTemporalProperty};
mod sink;
pub(crate) use sink::ExecutableTemporalSink;

#[derive(Debug, Clone)]
pub(crate) struct ExecutableTemporalLeaf {
    sink: ExecutableTemporalSink,
    binding_id: TemporalBindingId,
    expression: CompiledExpression,
    build_inputs: ResidualBuildBindings,
    parameter_values: BTreeMap<TemporalParameterId, TemporalValue>,
    request: ResidualizationRequest,
}

impl ExecutableTemporalLeaf {
    pub(crate) fn new(
        sink: ExecutableTemporalSink,
        binding_id: TemporalBindingId,
        expression: CompiledExpression,
        request: ResidualizationRequest,
    ) -> Self {
        Self {
            sink,
            binding_id,
            expression,
            build_inputs: BTreeMap::new(),
            parameter_values: BTreeMap::new(),
            request,
        }
    }

    pub(crate) fn sink(&self) -> &ExecutableTemporalSink {
        &self.sink
    }

    pub(crate) fn binding_id(&self) -> &TemporalBindingId {
        &self.binding_id
    }

    pub(crate) fn expression(&self) -> &CompiledExpression {
        &self.expression
    }

    pub(crate) fn build_inputs(&self) -> &ResidualBuildBindings {
        &self.build_inputs
    }

    pub(crate) fn parameter_values(&self) -> &BTreeMap<TemporalParameterId, TemporalValue> {
        &self.parameter_values
    }

    pub(crate) fn request(&self) -> &ResidualizationRequest {
        &self.request
    }
}

#[cfg(test)]
mod test_api {
    use super::{ExecutableTemporalLeaf, TemporalParameterId, TemporalValue};
    use crate::program::expression::{CoreBuildInputId, Value};

    impl ExecutableTemporalLeaf {
        pub(crate) fn bind_build(mut self, id: CoreBuildInputId, value: Value) -> Self {
            self.build_inputs.insert(id, value);
            self
        }

        pub(crate) fn bind_parameter(
            mut self,
            id: TemporalParameterId,
            value: TemporalValue,
        ) -> Self {
            self.parameter_values.insert(id, value);
            self
        }
    }
}
