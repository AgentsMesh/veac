use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn visual(
        &mut self,
        visual: &VisualProperties,
        duration: RationalTime,
        timebase: u32,
        path: &str,
        item_id: &str,
    ) {
        match visual.placement {
            Placement::Anchor { inset, .. } => {
                self.finite_vec(inset, true, "PLACEMENT", path, item_id)
            }
            Placement::Absolute { position } => {
                self.length(position.x, false, "PLACEMENT", path, item_id);
                self.length(position.y, false, "PLACEMENT", path, item_id);
            }
        }
        if let Some(frame) = visual.frame {
            self.length(frame.width, true, "FRAME", path, item_id);
            self.length(frame.height, true, "FRAME", path, item_id);
        }
        self.transform(visual, duration, timebase, path, item_id);
        self.animatable(
            &visual.opacity,
            duration,
            timebase,
            &format!("{path}/visual/opacity"),
            item_id,
            |value| value.is_finite() && (0.0..=1.0).contains(value),
        );
        for (index, mask) in visual.masks.iter().enumerate() {
            self.mask(
                mask,
                duration,
                timebase,
                &format!("{path}/visual/masks/{index}"),
                item_id,
            );
        }
        self.visual_shape(visual, path, item_id);
        if let Some(pipeline) = &visual.color_pipeline {
            self.color_pipeline(pipeline, path, item_id);
        }
    }

    fn transform(
        &mut self,
        visual: &VisualProperties,
        duration: RationalTime,
        timebase: u32,
        path: &str,
        item_id: &str,
    ) {
        self.animatable(
            &visual.transform.position,
            duration,
            timebase,
            &format!("{path}/visual/transform/position"),
            item_id,
            |point| point.x.value.is_finite() && point.y.value.is_finite(),
        );
        self.animatable(
            &visual.transform.scale,
            duration,
            timebase,
            &format!("{path}/visual/transform/scale"),
            item_id,
            |scale| visual_scale_valid(*scale),
        );
        self.animatable(
            &visual.transform.rotation_degrees,
            duration,
            timebase,
            &format!("{path}/visual/transform/rotation_degrees"),
            item_id,
            |value| value.is_finite(),
        );
        if !visual_shear_valid(visual.transform.shear) {
            self.push(
                "SHEAR",
                Some(item_id.to_owned()),
                format!("{path}/transform/shear"),
                "x and y shear factors must each be finite and within [-2, 2]",
                Some("use unitless x/y shear factors in the closed range [-2, 2]"),
            );
        }
        if let Some(crop) = &visual.transform.crop {
            self.animatable(
                crop,
                duration,
                timebase,
                &format!("{path}/visual/transform/crop"),
                item_id,
                |_| true,
            );
            if crop_values(crop).any(invalid_crop) {
                self.value_error("CROP", path, item_id);
            }
        }
    }

    fn visual_shape(&mut self, visual: &VisualProperties, path: &str, item_id: &str) {
        let anchor = visual.transform.anchor;
        if !anchor.x.is_finite()
            || !anchor.y.is_finite()
            || !(0.0..=1.0).contains(&anchor.x)
            || !(0.0..=1.0).contains(&anchor.y)
        {
            self.value_error("ANCHOR", path, item_id);
        }
        if let Some(card) = &visual.card {
            if !card.corner_radius_pixels.is_finite() || card.corner_radius_pixels < 0.0 {
                self.value_error("CARD", path, item_id);
            }
            if let Some(shadow) = &card.shadow {
                self.shadow(shadow, "SHADOW", path, item_id);
            }
        }
    }

    pub(super) fn shadow(&mut self, shadow: &Shadow, code: &str, path: &str, item_id: &str) {
        if !shadow_blur_valid(shadow.blur_pixels)
            || !shadow.opacity.is_finite()
            || !(0.0..=1.0).contains(&shadow.opacity)
        {
            self.value_error(code, path, item_id);
        }
        self.finite_vec(shadow.offset, false, code, path, item_id);
    }
}

fn invalid_crop(crop: Rect) -> bool {
    !crop.x.is_finite()
        || !crop.y.is_finite()
        || !crop.width.is_finite()
        || !crop.height.is_finite()
        || crop.x < 0.0
        || crop.y < 0.0
        || crop.width < MIN_CROP_EXTENT
        || crop.height < MIN_CROP_EXTENT
        || crop.x + crop.width > 1.0
        || crop.y + crop.height > 1.0
}

fn crop_values(value: &Animatable<Rect>) -> impl Iterator<Item = Rect> + '_ {
    let constant = match value {
        Animatable::Constant { value } => Some(*value),
        Animatable::Keyframes { .. } | Animatable::Binding { .. } => None,
    };
    constant.into_iter().chain(
        value
            .keyframes()
            .unwrap_or_default()
            .iter()
            .map(|keyframe| keyframe.value),
    )
}
