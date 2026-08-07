use veac_ir::{Animatable, LengthUnit, Transform2D, Vec2};

use super::support;

#[path = "transform/errors.rs"]
mod errors;

#[test]
fn atomic_transform_channels_lower_without_property_projection() {
    let transform = transform(
        "point(100px, -30px)",
        "vector(1.0, 1.0)",
        "30deg",
        "vector(0.75, 0.25)",
        "flip_vertical()",
    );
    let item = styled_item("card", &transform);
    let envelope = support::envelope(&support::visual_project(&[&item]));
    let value = lowered(&envelope);
    assert_eq!(position(value), (100.0, -30.0));
    assert_eq!(constant(&value.scale), &Vec2 { x: 1.0, y: 1.0 });
    assert_eq!(*constant(&value.rotation_degrees), 30.0);
    assert_eq!(value.anchor, Vec2 { x: 0.75, y: 0.25 });
    assert!(!value.flip_horizontal && value.flip_vertical);
}

#[test]
fn latest_visual_update_replaces_the_complete_transform() {
    let first = style(&transform(
        "point(90px, 80px)",
        "vector(2.0, 2.0)",
        "0deg",
        "vector(0.5, 0.5)",
        "flip_none()",
    ));
    let second = style(&transform(
        "point(0px, 0px)",
        "vector(1.0, 1.0)",
        "12deg",
        "vector(0.5, 0.5)",
        "flip_vertical()",
    ));
    let item = format!(
        "item(identifier(\"card\"), item_enabled(), during(0s, 1s), \
         source_generated(generator_transparent()), source_timing_native())\
         .with_visual({first}).with_visual({second})"
    );
    let envelope = support::envelope(&support::visual_project(&[&item]));
    let value = lowered(&envelope);
    assert_eq!(position(value), (0.0, 0.0));
    assert_eq!(constant(&value.scale), &Vec2 { x: 1.0, y: 1.0 });
    assert_eq!(*constant(&value.rotation_degrees), 12.0);
    assert!(value.flip_vertical);
}

#[test]
fn absent_visual_style_retains_the_canonical_identity_transform() {
    let item = support::solid("card", "#112233ff", "0s", "1s");
    let source = support::visual_project(&[&item]);
    let envelope = support::envelope(&source);
    let value = lowered(&envelope);
    assert_eq!(position(value), (0.0, 0.0));
    assert_eq!(constant(&value.scale), &Vec2 { x: 1.0, y: 1.0 });
    assert_eq!(*constant(&value.rotation_degrees), 0.0);
    assert_eq!(value.anchor, Vec2 { x: 0.5, y: 0.5 });
    assert!(!value.flip_horizontal && !value.flip_vertical);
}

#[test]
fn repeated_transform_execution_is_deterministic() {
    let transform = transform(
        "point(12px, 8px)",
        "vector(1.2, 0.8)",
        "5deg",
        "vector(0.5, 0.5)",
        "flip_none()",
    );
    let item = styled_item("card", &transform);
    let source = support::visual_project(&[&item]);
    assert_eq!(support::envelope(&source), support::envelope(&source));
}

pub(super) fn transform(
    position: &str,
    scale: &str,
    rotation: &str,
    pivot: &str,
    flip: &str,
) -> String {
    format!(
        "transform_2d(transform_motion(point_constant({position}), \
         vector_constant({scale}), angle_constant({rotation})), \
         transform_geometry(vector(0.0, 0.0), {flip}, {pivot}, crop_none()))"
    )
}

pub(super) fn styled_item(key: &str, transform: &str) -> String {
    let style = style(transform);
    format!(
        "item(identifier(\"{key}\"), item_enabled(), during(0s, 1s), \
         source_generated(generator_transparent()), source_timing_native()).with_visual({style})"
    )
}

fn style(transform: &str) -> String {
    format!(
        "visual_style(visual_layout(placement_anchor(anchor_center(), vector(0.0, 0.0)), \
         frame_none(), {transform}), visual_surface(percent_constant(100%), \
         compositing(0, blend_normal()), card_none()), [], color_pipeline_none())"
    )
}

fn lowered(envelope: &veac_ir::ProjectEnvelope) -> &Transform2D {
    &support::clip(envelope, 0)
        .visual
        .as_ref()
        .unwrap()
        .transform
}

fn position(value: &Transform2D) -> (f64, f64) {
    let point = constant(&value.position);
    assert_eq!(
        (point.x.unit, point.y.unit),
        (LengthUnit::Pixels, LengthUnit::Pixels)
    );
    (point.x.value, point.y.value)
}

fn constant<T>(value: &Animatable<T>) -> &T {
    match value {
        Animatable::Constant { value } => value,
        Animatable::Keyframes { .. } | Animatable::Binding { .. } => panic!("expected constant"),
    }
}
