use super::*;

#[test]
fn transform_estimate_covers_scale_rotation_and_invalid_values() {
    assert_eq!(
        transform_pixels(100, 50, vec2(1.0, 1.0), false, vec2(0.0, 0.0), false),
        Some(5_000)
    );
    assert_eq!(
        transform_pixels(100, 50, vec2(1.5, 2.0), false, vec2(0.0, 0.0), false),
        Some(15_000)
    );
    assert_eq!(
        transform_pixels(100, 50, vec2(1.0, 1.0), true, vec2(0.0, 0.0), false),
        Some(5_000)
    );
    assert_eq!(
        transform_pixels(100, 50, vec2(1.0, 1.0), false, vec2(0.5, 0.25), false),
        Some(9_375)
    );
    assert!(
        transform_pixels(100, 50, vec2(1.0, 1.0), true, vec2(0.0, 0.0), true).unwrap() > 50_000
    );
    assert_eq!(
        transform_pixels(0, 50, vec2(1.0, 1.0), false, vec2(0.0, 0.0), false),
        None
    );
    assert_eq!(
        transform_pixels(1, 1, vec2(f64::NAN, 1.0), false, vec2(0.0, 0.0), false),
        None
    );
    assert_eq!(
        transform_pixels(
            1,
            1,
            vec2(MAX_VISUAL_SCALE + 1.0, 1.0),
            false,
            vec2(0.0, 0.0),
            false,
        ),
        None
    );
    assert_eq!(
        transform_pixels(
            1,
            1,
            vec2(1.0, 1.0),
            true,
            vec2(MAX_VISUAL_SHEAR + 1.0, 0.0),
            false,
        ),
        None
    );
}

#[test]
fn transform_extent_charges_each_noncenter_pivot_stage() {
    let one = vec2(1.0, 1.0);
    let shear = vec2(0.5, 0.25);
    assert_eq!(
        transform_pixels(100, 50, one, true, shear, false),
        Some(37_500)
    );
    assert_eq!(
        transform_pixels(100, 50, one, true, vec2(0.0, 0.0), true),
        Some(50_176)
    );
    assert_eq!(
        transform_pixels(100, 50, one, false, shear, true),
        Some(21_316)
    );
    assert_eq!(
        transform_pixels(100, 50, one, true, shear, true),
        Some(341_056)
    );
}

fn vec2(x: f64, y: f64) -> Vec2 {
    Vec2 { x, y }
}

fn transform_pixels(
    width: u32,
    height: u32,
    scale: Vec2,
    pivot_off_center: bool,
    shear: Vec2,
    rotation_square: bool,
) -> Option<u128> {
    let extent = visual_transform_extent(
        width,
        height,
        scale,
        pivot_off_center,
        shear,
        rotation_square,
    )?;
    visual_intermediate_pixels(extent, None, (width, height), None)
}

#[test]
fn placement_estimate_matches_dynamic_pad_geometry() {
    assert_eq!(placement_pixels((100, 50), (100, 50)), Some(45_000));
    assert_eq!(placement_pixels((100, 50), (150, 100)), Some(100_000));
    assert_eq!(placement_intermediate_extent((0, 1), (1, 1)), None);
    assert_eq!(
        placement_intermediate_extent((1, 1), (u128::MAX, 1)),
        Some((u128::MAX, 3))
    );
}

fn placement_pixels(canvas: (u32, u32), transformed: (u64, u64)) -> Option<u128> {
    let transformed = (u128::from(transformed.0), u128::from(transformed.1));
    let placement = placement_intermediate_extent(canvas, transformed)?;
    visual_intermediate_pixels(transformed, Some(placement), canvas, None)
}

#[test]
fn visible_overflow_extent_scales_the_surface_from_its_layout_basis() {
    assert_eq!(
        visible_overflow_frame_extent((100, 100), (20.0, 20.0), (40, 40), FitMode::Contain),
        Some((200, 200))
    );
    assert_eq!(
        visible_overflow_frame_extent((100, 100), (20.0, 20.0), (40, 20), FitMode::Fill),
        Some((200, 100))
    );
    assert_eq!(
        visible_overflow_frame_extent((100, 50), (20.0, 20.0), (40, 20), FitMode::Cover),
        Some((200, 100))
    );
    assert_eq!(
        visible_overflow_frame_extent((100, 100), (0.0, 20.0), (40, 40), FitMode::Contain),
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
