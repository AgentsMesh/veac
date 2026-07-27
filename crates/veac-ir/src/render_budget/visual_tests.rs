use super::*;

#[test]
fn transform_estimate_covers_scale_rotation_and_invalid_values() {
    assert_eq!(
        visual_transform_pixels(100, 50, 1.0, 1.0, false),
        Some(5_000)
    );
    assert_eq!(
        visual_transform_pixels(100, 50, 1.5, 2.0, false),
        Some(15_000)
    );
    assert!(visual_transform_pixels(100, 50, 1.0, 1.0, true).unwrap() > 50_000);
    assert_eq!(visual_transform_pixels(0, 50, 1.0, 1.0, false), None);
    assert_eq!(visual_transform_pixels(1, 1, f64::NAN, 1.0, false), None);
    assert_eq!(
        visual_transform_pixels(1, 1, MAX_VISUAL_SCALE + 1.0, 1.0, false),
        None
    );
}

#[test]
fn shadow_estimate_matches_backend_padding() {
    assert_eq!(shadow_intermediate_pixels(100, 50, 0.0), Some(5_000));
    assert_eq!(shadow_intermediate_pixels(100, 50, 1.25), Some(5_936));
    assert_eq!(shadow_intermediate_pixels(0, 50, 1.0), None);
    assert_eq!(shadow_intermediate_pixels(1, 1, f64::INFINITY), None);
}
