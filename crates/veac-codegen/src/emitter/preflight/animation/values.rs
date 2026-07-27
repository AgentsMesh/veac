use veac_plan::canonical::{Interpolation, Point, Rect, Vec2};

pub(super) fn finite(value: &f64) -> bool {
    value.is_finite()
}

pub(super) fn nonnegative(value: &f64) -> bool {
    value.is_finite() && *value >= 0.0
}

pub(super) fn unit_number(value: &f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(value)
}

pub(super) fn pan(value: &f64) -> bool {
    value.is_finite() && (-1.0..=1.0).contains(value)
}

pub(super) fn point(value: &Point) -> bool {
    value.x.value.is_finite() && value.y.value.is_finite()
}

pub(super) fn positive_vec(value: &Vec2) -> bool {
    veac_plan::canonical::visual_scale_valid(*value)
}

pub(super) fn unit_vec(value: &Vec2) -> bool {
    value.x.is_finite()
        && value.y.is_finite()
        && (0.0..=1.0).contains(&value.x)
        && (0.0..=1.0).contains(&value.y)
}

pub(super) fn rect(value: &Rect) -> bool {
    value.x.is_finite()
        && value.y.is_finite()
        && value.width.is_finite()
        && value.height.is_finite()
        && value.x >= 0.0
        && value.y >= 0.0
        && value.width >= veac_plan::canonical::MIN_CROP_EXTENT
        && value.height >= veac_plan::canonical::MIN_CROP_EXTENT
        && value.x + value.width <= 1.0
        && value.y + value.height <= 1.0
}

pub(super) fn interpolation(value: &Interpolation) -> bool {
    match value {
        Interpolation::CubicBezier { x1, y1, x2, y2 } => {
            x1.is_finite()
                && y1.is_finite()
                && x2.is_finite()
                && y2.is_finite()
                && (0.0..=1.0).contains(x1)
                && (0.0..=1.0).contains(y1)
                && (0.0..=1.0).contains(x2)
                && (0.0..=1.0).contains(y2)
        }
        _ => true,
    }
}
