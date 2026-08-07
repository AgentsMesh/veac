use crate::*;

use super::{SinkScope, TemporalContract};

impl TemporalContract<'_, '_> {
    pub(super) fn project_sinks(&mut self, project: &Project) {
        for sequence in &project.sequences {
            let sequence_id = sequence.id.as_str();
            for track in &sequence.tracks {
                for clip in &track.clips {
                    let base = format!(
                        "/project/sequences/{sequence_id}/tracks/{}/clips/{}",
                        track.id, clip.id
                    );
                    self.clip(clip, &base, SinkScope::item(sequence_id, clip.id.as_str()));
                }
            }
            for apply in &sequence.applies {
                let base = format!("/project/sequences/{sequence_id}/applies/{}", apply.id);
                self.apply(apply, &base, SinkScope::sequence(sequence_id));
            }
        }
    }

    fn clip(&mut self, clip: &Clip, base: &str, scope: SinkScope<'_>) {
        if let Some(visual) = &clip.visual {
            self.visual(visual, &format!("{base}/visual"), scope);
        }
        if let Some(audio) = &clip.audio {
            self.leaf(
                &audio.gain,
                TemporalType::Scalar,
                &format!("{base}/audio/gain"),
                scope,
            );
            self.leaf(
                &audio.pan,
                TemporalType::Scalar,
                &format!("{base}/audio/pan"),
                scope,
            );
        }
        if let ClipSource::Text { style, .. } | ClipSource::Caption { style, .. } = &clip.source {
            if let Some(animation) = &style.animation {
                self.text_animation(animation, &format!("{base}/source/style/animation"), scope);
            }
        }
        self.effects(&clip.effects, &format!("{base}/effects"), scope);
    }

    fn visual(&mut self, value: &VisualProperties, base: &str, scope: SinkScope<'_>) {
        let transform = &value.transform;
        self.leaf(
            &transform.position,
            TemporalType::Point,
            &format!("{base}/transform/position"),
            scope,
        );
        self.leaf(
            &transform.scale,
            TemporalType::Vec2,
            &format!("{base}/transform/scale"),
            scope,
        );
        self.leaf(
            &transform.rotation_degrees,
            TemporalType::Angle,
            &format!("{base}/transform/rotation_degrees"),
            scope,
        );
        if let Some(crop) = &transform.crop {
            self.leaf(
                crop,
                TemporalType::Rect,
                &format!("{base}/transform/crop"),
                scope,
            );
        }
        self.leaf(
            &value.opacity,
            TemporalType::Scalar,
            &format!("{base}/opacity"),
            scope,
        );
        for (index, mask) in value.masks.iter().enumerate() {
            self.mask(mask, &format!("{base}/masks/{index}"), scope);
        }
    }

    fn mask(&mut self, value: &Mask, base: &str, scope: SinkScope<'_>) {
        self.leaf(
            &value.position,
            TemporalType::Vec2,
            &format!("{base}/position"),
            scope,
        );
        self.leaf(
            &value.scale,
            TemporalType::Vec2,
            &format!("{base}/scale"),
            scope,
        );
        self.leaf(
            &value.rotation_degrees,
            TemporalType::Angle,
            &format!("{base}/rotation_degrees"),
            scope,
        );
        self.leaf(
            &value.feather_pixels,
            TemporalType::Scalar,
            &format!("{base}/feather_pixels"),
            scope,
        );
        self.leaf(
            &value.expansion_pixels,
            TemporalType::Scalar,
            &format!("{base}/expansion_pixels"),
            scope,
        );
    }

    fn text_animation(&mut self, value: &TextAnimation, base: &str, scope: SinkScope<'_>) {
        self.leaf(
            &value.reveal,
            TemporalType::Scalar,
            &format!("{base}/reveal"),
            scope,
        );
        self.leaf(
            &value.opacity,
            TemporalType::Scalar,
            &format!("{base}/opacity"),
            scope,
        );
        if let Some(highlight) = &value.highlight {
            self.leaf(
                &highlight.progress,
                TemporalType::Scalar,
                &format!("{base}/highlight/progress"),
                scope,
            );
        }
        self.leaf(
            &value.transform.position_offset,
            TemporalType::Point,
            &format!("{base}/transform/position_offset"),
            scope,
        );
        self.leaf(
            &value.transform.scale,
            TemporalType::Vec2,
            &format!("{base}/transform/scale"),
            scope,
        );
        self.leaf(
            &value.transform.rotation_degrees,
            TemporalType::Angle,
            &format!("{base}/transform/rotation_degrees"),
            scope,
        );
    }

    fn effects(&mut self, values: &[EffectInstance], base: &str, scope: SinkScope<'_>) {
        for effect in values {
            for parameter in EffectParameter::ALL {
                if let Some(value) = effect.effect.curve(parameter) {
                    self.leaf(
                        value,
                        TemporalType::Scalar,
                        &format!("{base}/{}/{}", effect.id, parameter.name()),
                        scope,
                    );
                }
            }
        }
    }

    fn apply(&mut self, value: &Apply, base: &str, scope: SinkScope<'_>) {
        self.leaf(
            &value.mix.opacity,
            TemporalType::Scalar,
            &format!("{base}/mix/opacity"),
            scope,
        );
        for (index, mask) in value.mix.masks.iter().enumerate() {
            self.mask(mask, &format!("{base}/mix/masks/{index}"), scope);
        }
        for stage in &value.stages {
            if let ApplyOperation::Effect { effect } = &stage.operation {
                self.effects(
                    std::slice::from_ref(effect),
                    &format!("{base}/stages/{}", stage.id),
                    scope,
                );
            }
        }
    }
}
