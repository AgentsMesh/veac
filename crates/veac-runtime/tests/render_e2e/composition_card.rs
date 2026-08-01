use super::support::*;
use tempfile::tempdir;

#[test]
fn padded_translucent_card_keeps_color_across_both_halves() {
    let temp = tempdir().expect("tempdir");

    let base = solid_clip("itm_base", color(16, 37, 63), 0, 1_000);
    let mut card = solid_clip("itm_card", color(0, 0, 0), 0, 1_000);
    card.source = ClipSource::Generated {
        generator: Generator::Shape {
            shape: VectorShape {
                geometry: VectorGeometry::RoundedRectangle {
                    bounds: Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 1.0,
                        height: 1.0,
                    },
                    radius: 0.18,
                },
                fill: Some(Paint::Solid {
                    color: color(247, 247, 242),
                }),
                stroke: Some(VectorStroke {
                    paint: Paint::Solid {
                        color: color(30, 195, 180),
                    },
                    width_pixels: 2.0,
                }),
            },
        },
    };
    let mut visual = framed_visual(Anchor::Center, Some((54.0, 28.0)));
    visual.opacity = Animatable::constant(0.92);
    card.visual = Some(visual);

    let mut project = project(false);
    project.project.sequences[0].tracks = vec![
        track("trk_base", TrackKind::Video, 0, vec![base]),
        track("trk_card", TrackKind::Visual, 1, vec![card]),
    ];
    let output = temp.path().join("card.mp4");
    render(project, &BTreeMap::new(), &output);

    let left = rgb_at(&output, 0.5, 32, 27);
    let right = rgb_at(&output, 0.5, 64, 27);
    assert_light_card(left);
    assert_light_card(right);
    for channel in 0..3 {
        assert!(
            left[channel].abs_diff(right[channel]) <= 4,
            "{left:?} {right:?}"
        );
    }
    assert_navy(rgb_at(&output, 0.5, 5, 5));
    assert_navy(rgb_at(&output, 0.5, 21, 13));
    let stroke = rgb_at(&output, 0.5, 22, 27);
    assert!(
        stroke[1] > stroke[0] + 15 && stroke[2] > stroke[0] + 10,
        "{stroke:?}"
    );
}

fn assert_light_card(pixel: [u8; 3]) {
    assert!(
        pixel[0] > 205 && pixel[1] > 205 && pixel[2] > 200,
        "{pixel:?}"
    );
}

fn assert_navy(pixel: [u8; 3]) {
    assert!(pixel[0] < 40 && pixel[1] < 60 && pixel[2] > 45, "{pixel:?}");
}
