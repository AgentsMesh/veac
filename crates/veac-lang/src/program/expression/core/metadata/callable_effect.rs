use super::{CallableContract, CoreValueMetadata, DeferredCall, EffectEvidence, FunctionSummary};
use crate::program::expression::{Effect, FunctionEffect, MAX_EXPRESSION_DEPTH};

impl CoreValueMetadata {
    pub(crate) fn known_callable_effect(&self, arguments: &[Self]) -> Option<EffectEvidence> {
        self.callable
            .as_ref()?
            .known_effect(arguments, MAX_EXPRESSION_DEPTH)
    }
}

impl CallableContract {
    fn known_effect(
        &self,
        arguments: &[CoreValueMetadata],
        remaining: usize,
    ) -> Option<EffectEvidence> {
        let remaining = remaining.checked_sub(1)?;
        match self {
            Self::Closure { summary, captures } => {
                summary.known_effect(arguments, captures, remaining)
            }
            Self::Join(values) => values
                .iter()
                .try_fold(EffectEvidence::PURE, |effect, value| {
                    value
                        .known_effect(arguments, remaining)
                        .map(|value| effect.join(value))
                }),
            Self::Bound(effect)
            | Self::Binding {
                fallback: effect, ..
            } => bound(*effect),
            Self::Impossible => Some(EffectEvidence::PURE),
            Self::Deferred { .. } => None,
        }
    }
}

fn bound(effect: FunctionEffect) -> Option<EffectEvidence> {
    match effect {
        FunctionEffect::Pure => Some(EffectEvidence::PURE),
        FunctionEffect::Local => Some(EffectEvidence::from_effect(Effect::LocalMutation)),
        FunctionEffect::Emit => Some(EffectEvidence::from_effect(Effect::GraphEmit)),
        FunctionEffect::Any => None,
    }
}

impl FunctionSummary {
    fn known_effect(
        &self,
        arguments: &[CoreValueMetadata],
        captures: &[CoreValueMetadata],
        remaining: usize,
    ) -> Option<EffectEvidence> {
        self.deferred_effects
            .iter()
            .try_fold(self.effect, |effect, call| {
                call.known_effect(arguments, captures, remaining)
                    .map(|value| effect.join(value))
            })
    }
}

impl DeferredCall {
    fn known_effect(
        &self,
        parameters: &[CoreValueMetadata],
        captures: &[CoreValueMetadata],
        remaining: usize,
    ) -> Option<EffectEvidence> {
        let callee = self.callee.bind(parameters, captures)?;
        let arguments = self
            .arguments
            .iter()
            .map(|value| value.bind(parameters, captures))
            .collect::<Option<Vec<_>>>()?;
        callee.known_effect(&arguments, remaining)
    }
}
