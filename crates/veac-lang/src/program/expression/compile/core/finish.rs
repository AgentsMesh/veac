use super::Builder;
use crate::program::expression::core::{
    BlockId, CoreBlock, CoreProgram, CoreTerminator, ValueId, CORE_VERSION,
};
use crate::program::expression::ValueType;

impl Builder<'_> {
    pub(super) fn finish(
        mut self,
        value: ValueId,
        result_type: &ValueType,
        span: std::ops::Range<usize>,
        extra_types: &[ValueType],
    ) -> CoreProgram {
        self.terminate(CoreTerminator::Return {
            value,
            span: span.clone(),
        });
        let result_type = self.types.intern_value(result_type);
        let types = self.types.finish();
        let nominal_definitions = super::nominal::collect(
            &types,
            &self.closure_definitions,
            extra_types,
            self.nominal_types,
            span,
        );
        CoreProgram {
            version: CORE_VERSION,
            domain_opset: self.domain.version(),
            domain_registry_digest: self.domain.digest(),
            entry: BlockId::new(0),
            types,
            nominal_definitions,
            inputs: self.inputs,
            local_slots: self.local_slots,
            closure_definitions: self.closure_definitions,
            blocks: self
                .blocks
                .into_iter()
                .map(|block| CoreBlock {
                    id: block.id,
                    parameters: block.parameters,
                    instructions: block.instructions,
                    terminator: block.terminator.expect("lowered block must terminate"),
                })
                .collect(),
            result_type,
        }
    }
}
