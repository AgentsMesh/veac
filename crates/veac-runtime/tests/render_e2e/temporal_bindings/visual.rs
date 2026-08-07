use tempfile::tempdir;

use super::super::support::*;
use super::{install, temporal_node};

#[test]
fn progress_bindings_drive_position_and_opacity_in_rendered_pixels() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("temporal-visual.mp4");
    let mut project = project(false);
    let base = solid_clip("itm_temporal_base", color(20, 48, 72), 0, 1_000);
    let mut marker = solid_clip("itm_temporal_marker", color(245, 245, 245), 0, 1_000);
    let position = install(
        &mut project,
        "visual_position",
        "itm_temporal_marker",
        TemporalType::Point,
        position_nodes(),
        1,
    );
    let opacity = install(
        &mut project,
        "visual_opacity",
        "itm_temporal_marker",
        TemporalType::Scalar,
        vec![temporal_node(
            0,
            TemporalType::Scalar,
            TemporalNodeKind::Input {
                input_id: TemporalInputId::new(0),
            },
        )],
        0,
    );
    let mut visual = framed_visual(Anchor::Center, Some((12.0, 12.0)));
    visual.transform.position = Animatable::Binding {
        binding_id: position,
    };
    visual.opacity = Animatable::Binding {
        binding_id: opacity,
    };
    marker.visual = Some(visual);
    project.project.sequences[0].tracks.extend([
        track("trk_temporal_base", TrackKind::Video, 0, vec![base]),
        track("trk_temporal_marker", TrackKind::Visual, 1, vec![marker]),
    ]);
    render(project, &BTreeMap::new(), &output);

    let early = rgb_frame(&output, 0.2);
    let late = rgb_frame(&output, 0.8);
    let (early_x, early_energy) = marker_stats(&early);
    let (late_x, late_energy) = marker_stats(&late);
    assert!(late_x > early_x + 25.0, "marker x {early_x}..{late_x}");
    assert!(
        late_energy > early_energy * 2.2,
        "opacity energy {early_energy}..{late_energy}"
    );
}

fn position_nodes() -> Vec<TemporalNode> {
    vec![
        temporal_node(
            0,
            TemporalType::Scalar,
            TemporalNodeKind::Input {
                input_id: TemporalInputId::new(0),
            },
        ),
        temporal_node(
            1,
            TemporalType::Point,
            TemporalNodeKind::CurveSample {
                input: TemporalNodeId::new(0),
                keys: vec![point_key(0.0, -30.0), point_key(1.0, 30.0)],
            },
        ),
    ]
}

fn point_key(position: f64, x: f64) -> TemporalCurveKey {
    TemporalCurveKey {
        position: TemporalCurvePosition::Scalar { value: position },
        value: TemporalValue::Point {
            value: Point {
                x: pixels(x),
                y: pixels(0.0),
            },
        },
        interpolation: Interpolation::Linear,
    }
}

fn marker_stats(frame: &[u8]) -> (f64, f64) {
    let mut weighted_x = 0.0;
    let mut energy = 0.0;
    for (index, pixel) in frame.chunks_exact(3).enumerate() {
        let brightness = pixel.iter().map(|value| f64::from(*value)).sum::<f64>() / 3.0;
        let contribution = (brightness - 55.0).max(0.0);
        weighted_x += f64::from(index as u32 % WIDTH) * contribution;
        energy += contribution;
    }
    assert!(energy > 100.0, "marker is missing");
    (weighted_x / energy, energy)
}
