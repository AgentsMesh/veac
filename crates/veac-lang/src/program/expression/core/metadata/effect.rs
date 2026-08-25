use veac_lang_model::Effect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EffectEvidence {
    summary: Effect,
    contains_local_mutation: bool,
}

impl EffectEvidence {
    pub(crate) const PURE: Self = Self {
        summary: Effect::Pure,
        contains_local_mutation: false,
    };

    pub(crate) const fn from_effect(effect: Effect) -> Self {
        Self {
            summary: effect,
            contains_local_mutation: matches!(effect, Effect::LocalMutation),
        }
    }

    pub(crate) const fn summary(self) -> Effect {
        self.summary
    }

    pub(crate) const fn contains_local_mutation(self) -> bool {
        self.contains_local_mutation
    }

    pub(crate) fn covers(self, other: Self) -> bool {
        self.summary >= other.summary
            && (!other.contains_local_mutation || self.contains_local_mutation)
    }

    pub(crate) fn join(self, other: Self) -> Self {
        Self {
            summary: self.summary.max(other.summary),
            contains_local_mutation: self.contains_local_mutation || other.contains_local_mutation,
        }
    }
}

impl super::CoreValueMetadata {
    pub(crate) fn local_mutation<'a>(values: impl IntoIterator<Item = &'a Self>) -> Self {
        let mut output = Self::combine(values);
        output.effect = output
            .effect
            .join(EffectEvidence::from_effect(Effect::LocalMutation));
        output
    }
}
