use veac_plan::canonical::{Animatable, Mask, MaskShape, Vec2};

use super::{alpha, signed_distance, star_points, Coordinates};
use crate::emitter::process_owner::ProcessOwner;
use crate::unit_tests::emitter_tests::support::{fixture, resolved};

#[test]
fn primitive_distances_use_their_actual_pixel_axes() {
    let point = point();
    assert_eq!(signed_distance(&MaskShape::Linear, &point), "x");
    assert_eq!(
        signed_distance(&MaskShape::Mirror, &point),
        "((w)/2)-abs(x)"
    );
    let circle = signed_distance(&MaskShape::Circle, &point);
    assert_eq!(circle, "(min(w\\,h))/2-hypot(x\\,y)");

    let rectangle = signed_distance(&MaskShape::Rectangle, &point);
    assert!(rectangle.contains("abs(x)-(((w)/2)-(0))"), "{rectangle}");
    assert!(rectangle.contains("abs(y)-(((h)/2)-(0))"), "{rectangle}");
    assert!(!rectangle.contains("min(w\\,h))*"), "{rectangle}");
}

#[test]
fn ellipse_normalizes_the_gradient_back_to_pixel_distance() {
    let distance = signed_distance(&MaskShape::Ellipse, &point());
    assert!(distance.contains("x)/(((w)/2)*((w)/2))"), "{distance}");
    assert!(distance.contains("y)/(((h)/2)*((h)/2))"), "{distance}");
    assert!(distance.contains("min((w)/2\\,(h)/2)"), "{distance}");
    assert!(!distance.contains("min(w\\,h))*"), "{distance}");
}

#[test]
fn rounded_rectangle_radius_uses_the_short_pixel_extent() {
    let distance = signed_distance(&MaskShape::RoundedRectangle { radius: 0.2 }, &point());
    assert!(distance.contains("0.2*(min(w\\,h))"), "{distance}");
    assert!(distance.contains("abs(x)"), "{distance}");
    assert!(distance.contains("abs(y)"), "{distance}");
}

#[test]
fn implicit_shapes_normalize_their_gradients_in_each_pixel_axis() {
    let distance = signed_distance(&MaskShape::Heart, &point());
    for axis in ["/(0.45*(w))", "/(0.45*(h))"] {
        assert!(distance.contains(axis), "missing {axis}: {distance}");
    }
    assert!(distance.contains("max(hypot("), "{distance}");
}

#[test]
fn star_is_a_ten_edge_polygon_with_alternating_radii() {
    let points = star_points();
    for (index, point) in points.iter().enumerate() {
        let radius = ((point.x - 0.5).powi(2) + (point.y - 0.5).powi(2)).sqrt();
        let expected = if index % 2 == 0 {
            0.45
        } else {
            0.171884705062547
        };
        assert!((radius - expected).abs() < 1e-12, "point {index}: {radius}");
    }
    let distance = signed_distance(&MaskShape::Star, &point());
    assert_eq!(distance.matches("hypot(").count(), 10, "{distance}");
    assert!(distance.contains("(0.5-0.5)*(w)"), "{distance}");
    assert!(distance.contains("(0.05-0.5)*(h)"), "{distance}");
    assert!(!distance.contains("cos("), "{distance}");
}

#[test]
fn alpha_keeps_animated_transform_and_pixel_feather_in_one_expression() {
    let mut mask = mask(MaskShape::Rectangle);
    mask.position = Animatable::constant(Vec2 { x: 0.25, y: 0.75 });
    mask.scale = Animatable::constant(Vec2 { x: 0.8, y: 0.3 });
    mask.rotation_degrees = Animatable::constant(30.0);
    mask.feather_pixels = Animatable::constant(6.0);
    mask.expansion_pixels = Animatable::constant(4.0);
    let plan = resolved(&fixture());
    let clip = &plan.sequences[0].tracks[0].clips[0];
    let expression = alpha(&plan, ProcessOwner::clip(clip), &mask);
    for marker in [
        "X-W*(0.25)",
        "Y-H*(0.75)",
        "((30)*PI/180)",
        "W*(0.8)",
        "H*(0.3)",
        "+(4)",
        "max(2*(6)",
    ] {
        assert!(
            expression.contains(marker),
            "missing {marker}: {expression}"
        );
    }
}

fn point() -> Coordinates {
    Coordinates {
        x: "x".to_owned(),
        y: "y".to_owned(),
        width: "w".to_owned(),
        height: "h".to_owned(),
    }
}

fn mask(shape: MaskShape) -> Mask {
    Mask {
        shape,
        position: Animatable::constant(Vec2 { x: 0.5, y: 0.5 }),
        scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
        rotation_degrees: Animatable::constant(0.0),
        feather_pixels: Animatable::constant(0.0),
        expansion_pixels: Animatable::constant(0.0),
        invert: false,
    }
}
