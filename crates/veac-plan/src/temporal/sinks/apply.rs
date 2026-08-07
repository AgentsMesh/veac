use veac_ir::{EffectParameter, TemporalType};

use crate::{ResolvedApply, ResolvedApplyOperation, ResolvedEffect};

use super::{Collector, TemporalSinkScope};

impl Collector {
    pub(super) fn effects(
        &mut self,
        values: &[ResolvedEffect],
        base: &str,
        scope: &TemporalSinkScope,
    ) {
        for (effect_index, effect) in values.iter().enumerate() {
            for parameter in EffectParameter::ALL {
                if let Some(value) = effect.effect.curve(parameter) {
                    self.leaf(
                        value,
                        TemporalType::Scalar,
                        &format!("{base}/{effect_index}/effect/{}", parameter.name()),
                        scope,
                    );
                }
            }
        }
    }

    pub(super) fn apply(&mut self, value: &ResolvedApply, base: &str, scope: &TemporalSinkScope) {
        self.leaf(
            &value.mix.opacity,
            TemporalType::Scalar,
            &format!("{base}/mix/opacity"),
            scope,
        );
        for (index, mask) in value.mix.masks.iter().enumerate() {
            self.mask(mask, &format!("{base}/mix/masks/{index}"), scope);
        }
        for (index, stage) in value.stages.iter().enumerate() {
            let ResolvedApplyOperation::Effect { effect } = &stage.operation else {
                continue;
            };
            for parameter in EffectParameter::ALL {
                if let Some(value) = effect.curve(parameter) {
                    self.leaf(
                        value,
                        TemporalType::Scalar,
                        &format!(
                            "{base}/stages/{index}/operation/effect/{}",
                            parameter.name()
                        ),
                        scope,
                    );
                }
            }
        }
    }
}
