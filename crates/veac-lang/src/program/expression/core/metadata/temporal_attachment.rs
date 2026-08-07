use super::{CoreValueMetadata, Effect, EffectEvidence, Stage};

impl CoreValueMetadata {
    pub(crate) fn temporal_attachment(
        owner: &Self,
        selectors: impl IntoIterator<Item = Self>,
        animation: &Self,
    ) -> Self {
        let mut output = Self::constant();
        output.absorb_shape(owner);
        for selector in selectors {
            output.absorb_shape(&selector);
        }
        output.absorb_leaf(animation);
        output.shape_stage = output.shape_stage.max(Stage::Build);
        output.effect = output
            .effect
            .join(EffectEvidence::from_effect(Effect::GraphEmit));
        output
    }
}
