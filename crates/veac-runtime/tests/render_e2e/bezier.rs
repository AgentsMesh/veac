use tempfile::tempdir;
use veac_ir::{apply_edit_batch, EditBatch, EditOperation, EditOutcome};

use super::support::*;

#[test]
fn asymmetric_bezier_and_its_split_render_pixel_identically() {
    let temp = tempdir().unwrap();
    let original_path = temp.path().join("bezier-original.mp4");
    let split_path = temp.path().join("bezier-split.mp4");
    let original = bezier_project();
    let split = split_project(&original);

    let original_render = render(original, &BTreeMap::new(), &original_path);
    let split_render = render(split, &BTreeMap::new(), &split_path);
    for rendered in [&original_render, &split_render] {
        let graph = rendered.command.filter_graph.as_deref().unwrap();
        assert!(graph.contains("root("), "graph={graph}");
        assert!(!graph.contains("0.083333333333"), "graph={graph}");
    }
    for second in [0.2, 0.7, 0.8, 1.1, 1.4, 1.8] {
        let original_pixel = rgb_at(&original_path, second, WIDTH / 2, HEIGHT / 2);
        let split_pixel = rgb_at(&split_path, second, WIDTH / 2, HEIGHT / 2);
        assert_eq!(
            original_pixel, split_pixel,
            "center pixel changed at {second}s"
        );
        assert_eq!(
            rgb_frame(&original_path, second),
            rgb_frame(&split_path, second),
            "decoded frame changed at {second}s"
        );
    }
}

fn bezier_project() -> ProjectEnvelope {
    let mut value = project(false);
    let background = solid_clip("itm_bezier_bg", color(0, 0, 255), 0, 2_000);
    let mut foreground = solid_clip("itm_bezier_fg", color(255, 0, 0), 0, 2_000);
    let mut visual = full_visual();
    visual.opacity = Animatable::Keyframes {
        keyframes: vec![
            Keyframe {
                id: KeyframeId::new("kf_bezier_start").unwrap(),
                time: time(0),
                value: 0.0,
                interpolation: Interpolation::CubicBezier {
                    x1: 0.08,
                    y1: 0.2,
                    x2: 0.82,
                    y2: 0.9,
                },
            },
            Keyframe {
                id: KeyframeId::new("kf_bezier_end").unwrap(),
                time: time(2_000),
                value: 1.0,
                interpolation: Interpolation::Linear,
            },
        ],
    };
    foreground.visual = Some(visual);
    value.project.sequences[0].tracks.extend([
        track("trk_bezier_bg", TrackKind::Video, 0, vec![background]),
        track("trk_bezier_fg", TrackKind::Visual, 1, vec![foreground]),
    ]);
    value
}

fn split_project(value: &ProjectEnvelope) -> ProjectEnvelope {
    let batch = EditBatch {
        operation_id: OperationId::new("op_bezier_split_e2e").unwrap(),
        base_revision: value.project.revision,
        atomic: true,
        preconditions: vec![],
        operations: vec![EditOperation::SplitClip {
            clip_id: ItemId::new("itm_bezier_fg").unwrap(),
            at: time(800),
            right_clip_id: ItemId::new("itm_bezier_right").unwrap(),
            relation_fragments: vec![],
        }],
    };
    let EditOutcome::Applied { project, .. } = apply_edit_batch(value, &batch) else {
        panic!("Bezier split must apply")
    };
    project
}
