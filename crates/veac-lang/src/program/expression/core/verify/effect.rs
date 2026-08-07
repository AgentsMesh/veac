use super::super::metadata::EffectEvidence;
use crate::program::expression::{CollectionOperation, Effect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CollectionEffectViolation {
    LocalMutation,
    MapUnverified,
    RequiresPure,
    PureUnverified,
}

impl CollectionEffectViolation {
    pub(super) const fn message(self) -> &'static str {
        match self {
            Self::LocalMutation => "collection callbacks cannot perform LocalMutation",
            Self::MapUnverified => "map callback effects must be verified as Pure or GraphEmit",
            Self::RequiresPure => "filter and fold callbacks must be Pure",
            Self::PureUnverified => "filter and fold callback effects must be provably Pure",
        }
    }
}

pub(super) fn verify_collection(
    operation: CollectionOperation,
    evidence: Option<EffectEvidence>,
) -> Result<(), CollectionEffectViolation> {
    if evidence.is_some_and(EffectEvidence::contains_local_mutation) {
        return Err(CollectionEffectViolation::LocalMutation);
    }
    match (operation, evidence.map(EffectEvidence::summary)) {
        (_, Some(Effect::Pure)) | (CollectionOperation::Map, Some(Effect::GraphEmit)) => Ok(()),
        (CollectionOperation::Map, Some(Effect::LocalMutation)) => {
            Err(CollectionEffectViolation::LocalMutation)
        }
        (CollectionOperation::Map, None) => Err(CollectionEffectViolation::MapUnverified),
        (CollectionOperation::Filter | CollectionOperation::Fold, Some(_)) => {
            Err(CollectionEffectViolation::RequiresPure)
        }
        (CollectionOperation::Filter | CollectionOperation::Fold, None) => {
            Err(CollectionEffectViolation::PureUnverified)
        }
    }
}

#[cfg(test)]
#[path = "effect_tests.rs"]
mod tests;
