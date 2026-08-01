use veac_plan::canonical::{Color, Shadow, Vec2};

use super::{effective_opacity, placement_delta};

#[test]
fn color_alpha_multiplies_authored_shadow_opacity() {
    let shadow = Shadow {
        color: Color {
            red: 255,
            green: 0,
            blue: 0,
            alpha: 128,
        },
        offset: Vec2 { x: 0.0, y: 0.0 },
        blur_pixels: 0.0,
        opacity: 0.5,
    };

    assert!((effective_opacity(&shadow) - 0.5 * 128.0 / 255.0).abs() < f64::EPSILON);
}

#[test]
fn padding_compensation_preserves_fractional_offsets_at_any_pivot() {
    assert_eq!(placement_delta(3.25, 16, 0.0), -12.75);
    assert_eq!(placement_delta(3.25, 16, 0.5), 3.25);
    assert_eq!(placement_delta(3.25, 16, 1.0), 19.25);
}
