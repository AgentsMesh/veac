use std::sync::Arc;

use super::{CoreValueMetadata, DeferredCall, Effect, EffectEvidence};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSummary {
    pub(crate) result: CoreValueMetadata,
    pub(crate) effect: EffectEvidence,
    pub(crate) deferred_effects: Vec<Arc<DeferredCall>>,
    pub(crate) instruction_count: usize,
}

impl FunctionSummary {
    pub fn result(&self) -> &CoreValueMetadata {
        &self.result
    }

    pub fn effect(&self) -> Effect {
        self.effect.summary()
    }

    pub fn contains_local_mutation(&self) -> bool {
        self.effect.contains_local_mutation()
    }

    pub fn instruction_count(&self) -> usize {
        self.instruction_count
    }

    pub(crate) fn instantiate(
        &self,
        arguments: &[CoreValueMetadata],
        captures: &[CoreValueMetadata],
    ) -> Option<CoreValueMetadata> {
        let mut output = self.result.bind(arguments, captures)?;
        output.effect = output.effect.join(self.effect);
        for call in &self.deferred_effects {
            output.absorb_effect(&call.bind(arguments, captures)?);
        }
        Some(output)
    }
}

impl crate::program::expression::core::CoreProgram {
    pub(crate) fn function_summary(&self) -> FunctionSummary {
        let values = self
            .blocks
            .iter()
            .flat_map(|block| {
                block
                    .parameters
                    .iter()
                    .map(|value| (value.id, value.metadata.clone()))
                    .chain(
                        block
                            .instructions
                            .iter()
                            .map(|value| (value.id, value.metadata.clone())),
                    )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        let returned = self
            .blocks
            .iter()
            .filter_map(|block| match &block.terminator {
                crate::program::expression::core::CoreTerminator::Return { value, .. } => {
                    values.get(value)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut result = CoreValueMetadata::combine(returned.iter().copied());
        result.join_contract_from(&returned);
        let all = self.blocks.iter().flat_map(|block| {
            block
                .instructions
                .iter()
                .map(|value| &value.metadata)
                .chain(match &block.terminator {
                    crate::program::expression::core::CoreTerminator::ForEach(value) => {
                        Some(value.result_metadata())
                    }
                    _ => None,
                })
        });
        let mut effect = EffectEvidence::PURE;
        let mut deferred_effects = Vec::new();
        for metadata in all {
            effect = effect.join(metadata.effect);
            for deferred in &metadata.deferred {
                if !deferred_effects.contains(&deferred.call) {
                    deferred_effects.push(Arc::clone(&deferred.call));
                }
            }
        }
        FunctionSummary {
            result,
            effect,
            deferred_effects,
            instruction_count: self
                .blocks
                .iter()
                .map(|block| block.instructions.len())
                .sum(),
        }
    }
}
