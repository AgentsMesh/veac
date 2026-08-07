use veac_ir::{Mask, TemporalType, TextAnimation};

use crate::EffectiveVisualProperties;

use super::{Collector, TemporalSinkScope};

impl Collector {
    pub(super) fn visual(
        &mut self,
        value: &EffectiveVisualProperties,
        base: &str,
        scope: &TemporalSinkScope,
    ) {
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

    pub(super) fn mask(&mut self, value: &Mask, base: &str, scope: &TemporalSinkScope) {
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

    pub(super) fn text(&mut self, value: &TextAnimation, base: &str, scope: &TemporalSinkScope) {
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
}
