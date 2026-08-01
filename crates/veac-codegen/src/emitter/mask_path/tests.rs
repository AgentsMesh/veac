use veac_plan::canonical::Vec2;

use super::signed_distance;

#[test]
fn vertices_and_edge_distance_are_mapped_to_each_pixel_axis() {
    let points = [
        Vec2 { x: 0.1, y: 0.2 },
        Vec2 { x: 0.9, y: 0.2 },
        Vec2 { x: 0.5, y: 0.8 },
    ];
    let distance = signed_distance(&points, "x", "y", "w", "h");
    for marker in [
        "(0.1-0.5)*(w)",
        "(0.2-0.5)*(h)",
        "0.8*(w)",
        "0.6*(h)",
        "*(-0.4*(w))/(0.6*(h))",
        "hypot((x)",
        "(y)-(",
    ] {
        assert!(distance.contains(marker), "missing {marker}: {distance}");
    }
    assert!(!distance.contains("min(w\\,h)"), "{distance}");
}

#[test]
fn invalid_path_is_outside() {
    assert_eq!(signed_distance(&[], "x", "y", "w", "h"), "-1");
}
