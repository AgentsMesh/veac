use tempfile::tempdir;

use super::support::*;

struct ShapeCase {
    id: &'static str,
    geometry: VectorGeometry,
    fill: (u32, u32),
    stroke: (u32, u32),
    outside: (u32, u32),
}

#[test]
fn every_non_rectangle_geometry_renders_fill_stroke_and_shape_boundary() {
    let temp = tempdir().unwrap();
    let cases = cases();
    let shapes = cases
        .iter()
        .enumerate()
        .map(|(index, case)| shape_clip(case, index as i64 * 1_000))
        .collect();
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_background",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_background", color(0, 0, 0), 0, 4_000)],
        ),
        track("trk_shapes", TrackKind::Visual, 1, shapes),
    ]);
    let output = temp.path().join("shape-geometries.mp4");
    render(canonical, &BTreeMap::new(), &output);

    assert_media_contract(&output, 0, 4.0);
    for (index, case) in cases.iter().enumerate() {
        let second = index as f64 + 0.5;
        assert_fill(case, rgb_at(&output, second, case.fill.0, case.fill.1));
        assert_stroke(case, rgb_at(&output, second, case.stroke.0, case.stroke.1));
        assert_outside(
            case,
            rgb_at(&output, second, case.outside.0, case.outside.1),
        );
    }
}

fn cases() -> Vec<ShapeCase> {
    vec![
        ShapeCase {
            id: "ellipse",
            geometry: VectorGeometry::Ellipse {
                bounds: bounds(0.2, 0.15, 0.6, 0.7),
            },
            fill: (48, 27),
            stroke: (20, 27),
            outside: (20, 10),
        },
        ShapeCase {
            id: "rounded_rectangle",
            geometry: VectorGeometry::RoundedRectangle {
                bounds: bounds(0.15, 0.12, 0.7, 0.76),
                radius: 0.25,
            },
            fill: (48, 27),
            stroke: (48, 8),
            outside: (16, 8),
        },
        ShapeCase {
            id: "polygon",
            geometry: VectorGeometry::Polygon {
                points: vec![point(0.5, 0.12), point(0.85, 0.82), point(0.15, 0.82)],
            },
            fill: (48, 30),
            stroke: (48, 8),
            outside: (18, 8),
        },
        ShapeCase {
            id: "path",
            geometry: VectorGeometry::Path {
                commands: vec![
                    PathCommand::MoveTo {
                        point: point(0.5, 0.1),
                    },
                    PathCommand::LineTo {
                        point: point(0.9, 0.5),
                    },
                    PathCommand::LineTo {
                        point: point(0.5, 0.9),
                    },
                    PathCommand::LineTo {
                        point: point(0.1, 0.5),
                    },
                    PathCommand::Close,
                ],
            },
            fill: (48, 27),
            stroke: (48, 6),
            outside: (12, 8),
        },
    ]
}

fn shape_clip(case: &ShapeCase, start_ms: i64) -> Clip {
    let mut clip = solid_clip(&format!("itm_{}", case.id), color(0, 0, 0), start_ms, 1_000);
    clip.source = ClipSource::Generated {
        generator: Generator::Shape {
            shape: VectorShape {
                geometry: case.geometry.clone(),
                fill: Some(Paint::Solid {
                    color: color(0, 220, 30),
                }),
                stroke: Some(VectorStroke {
                    paint: Paint::Solid {
                        color: color(255, 255, 255),
                    },
                    width_pixels: 4.0,
                }),
            },
        },
    };
    clip
}

fn bounds(x: f64, y: f64, width: f64, height: f64) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}

fn point(x: f64, y: f64) -> Vec2 {
    Vec2 { x, y }
}

fn assert_fill(case: &ShapeCase, pixel: [u8; 3]) {
    assert!(
        pixel[1] > 150 && pixel[0] < 90 && pixel[2] < 90,
        "{} fill={pixel:?}",
        case.id
    );
}

fn assert_stroke(case: &ShapeCase, pixel: [u8; 3]) {
    assert!(
        pixel.iter().all(|channel| *channel > 145),
        "{} stroke={pixel:?}",
        case.id
    );
}

fn assert_outside(case: &ShapeCase, pixel: [u8; 3]) {
    assert!(
        pixel.iter().all(|channel| *channel < 55),
        "{} outside={pixel:?}",
        case.id
    );
}
