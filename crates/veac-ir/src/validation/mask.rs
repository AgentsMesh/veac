use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn mask(
        &mut self,
        mask: &Mask,
        duration: RationalTime,
        timebase: u32,
        path: &str,
        item_id: &str,
    ) {
        self.animatable(&mask.position, duration, timebase, path, item_id, unit_vec);
        self.animatable(&mask.scale, duration, timebase, path, item_id, positive_vec);
        self.animatable(
            &mask.rotation_degrees,
            duration,
            timebase,
            path,
            item_id,
            |value| value.is_finite(),
        );
        self.animatable(
            &mask.feather_pixels,
            duration,
            timebase,
            path,
            item_id,
            |value| value.is_finite() && *value >= 0.0,
        );
        self.animatable(
            &mask.expansion_pixels,
            duration,
            timebase,
            path,
            item_id,
            |value| value.is_finite(),
        );
        match &mask.shape {
            MaskShape::RoundedRectangle { radius }
                if !radius.is_finite() || !(0.0..=0.5).contains(radius) =>
            {
                self.value_error("MASK_RADIUS", path, item_id);
            }
            MaskShape::Polygon { points } | MaskShape::Path { points } if !valid_path(points) => {
                self.value_error("MASK_PATH", path, item_id);
            }
            _ => {}
        }
    }
}

fn valid_path(points: &[Vec2]) -> bool {
    points.len() >= 3
        && points.iter().all(unit_vec)
        && points.windows(2).all(|pair| pair[0] != pair[1])
        && polygon_area(points).abs() > f64::EPSILON
}

fn polygon_area(points: &[Vec2]) -> f64 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(left, right)| left.x * right.y - right.x * left.y)
        .sum::<f64>()
        / 2.0
}

fn unit_vec(value: &Vec2) -> bool {
    value.x.is_finite()
        && value.y.is_finite()
        && (0.0..=1.0).contains(&value.x)
        && (0.0..=1.0).contains(&value.y)
}

fn positive_vec(value: &Vec2) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.x > 0.0 && value.y > 0.0
}
