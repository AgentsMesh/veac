use tempfile::tempdir;

use super::support::*;

#[test]
fn radial_gradient_renders_center_ring_and_edge_colors() {
    let temp = tempdir().unwrap();
    let gradient = Gradient::Radial {
        center: point(0.5, 0.5),
        radius: 0.35,
        stops: stops(color(255, 0, 0), color(0, 0, 255)),
    };
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks.push(track(
        "trk_radial",
        TrackKind::Video,
        0,
        vec![generated_clip(
            "itm_radial",
            0,
            Generator::Gradient { gradient },
        )],
    ));
    let output = temp.path().join("radial.mp4");
    render(canonical, &BTreeMap::new(), &output);

    assert_media_contract(&output, 0, 1.0);
    assert_red(rgb_at(&output, 0.5, 48, 27));
    let ring = rgb_at(&output, 0.5, 60, 27);
    assert!(ring[0] > 100 && ring[2] > 45, "ring={ring:?}");
    assert_blue(rgb_at(&output, 0.5, 5, 27));
}

#[test]
fn shape_gradient_fill_and_stroke_render_independent_axes() {
    let temp = tempdir().unwrap();
    let shape = VectorShape {
        geometry: VectorGeometry::Rectangle {
            bounds: Rect {
                x: 0.15,
                y: 0.15,
                width: 0.7,
                height: 0.7,
            },
        },
        fill: Some(Paint::Gradient {
            gradient: linear(
                point(0.0, 0.5),
                point(1.0, 0.5),
                color(255, 0, 0),
                color(0, 0, 255),
            ),
        }),
        stroke: Some(VectorStroke {
            paint: Paint::Gradient {
                gradient: linear(
                    point(0.5, 0.0),
                    point(0.5, 1.0),
                    color(0, 255, 0),
                    color(255, 0, 255),
                ),
            },
            width_pixels: 5.0,
        }),
    };
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_background",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_background", color(0, 0, 0), 0, 1_000)],
        ),
        track(
            "trk_gradient_shape",
            TrackKind::Visual,
            1,
            vec![generated_clip(
                "itm_gradient_shape",
                0,
                Generator::Shape { shape },
            )],
        ),
    ]);
    let output = temp.path().join("gradient-paints.mp4");
    render(canonical, &BTreeMap::new(), &output);

    assert_media_contract(&output, 0, 1.0);
    let fill_left = rgb_at(&output, 0.5, 30, 27);
    let fill_right = rgb_at(&output, 0.5, 66, 27);
    assert!(fill_left[0] > fill_left[2] + 50, "left={fill_left:?}");
    assert!(fill_right[2] > fill_right[0] + 50, "right={fill_right:?}");
    let stroke_top = rgb_at(&output, 0.5, 48, 10);
    let stroke_bottom = rgb_at(&output, 0.5, 48, 44);
    assert!(
        stroke_top[1] > 160 && stroke_top[0] < 100,
        "top={stroke_top:?}"
    );
    assert!(
        stroke_bottom[0] > 160 && stroke_bottom[2] > 160 && stroke_bottom[1] < 100,
        "bottom={stroke_bottom:?}"
    );
}

fn generated_clip(id: &str, start_ms: i64, generator: Generator) -> Clip {
    let mut clip = solid_clip(id, color(0, 0, 0), start_ms, 1_000);
    clip.source = ClipSource::Generated { generator };
    clip
}

fn linear(start: Vec2, end: Vec2, first: Color, last: Color) -> Gradient {
    Gradient::Linear {
        start,
        end,
        stops: stops(first, last),
    }
}

fn stops(first: Color, last: Color) -> Vec<GradientStop> {
    vec![
        GradientStop {
            offset: 0.0,
            color: first,
        },
        GradientStop {
            offset: 1.0,
            color: last,
        },
    ]
}

fn point(x: f64, y: f64) -> Vec2 {
    Vec2 { x, y }
}

fn assert_red(pixel: [u8; 3]) {
    assert!(pixel[0] > 190 && pixel[2] < 60, "pixel={pixel:?}");
}

fn assert_blue(pixel: [u8; 3]) {
    assert!(pixel[2] > 190 && pixel[0] < 60, "pixel={pixel:?}");
}
