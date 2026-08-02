use tempfile::tempdir;

use super::support::*;

#[test]
fn nested_sequence_is_composited_only_inside_parent_record_range() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    let mut child_layer = solid_clip("itm_child", color(255, 0, 0), 0, 1_000);
    child_layer.visual = Some(child_visual());
    let mut child = sequence(
        "seq_child",
        vec![track("trk_child", TrackKind::Video, 0, vec![child_layer])],
    );
    child.settings.width = 48;
    child.settings.height = 36;
    child.settings.frame_rate = ratio(5, 1);
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_background",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_background", color(0, 0, 255), 0, 2_000)],
        ),
        track(
            "trk_nested",
            TrackKind::Visual,
            1,
            vec![nested_clip("itm_nested", "seq_child", 500, 1_000)],
        ),
    ]);
    canonical.project.sequences.push(child);
    let output = temp.path().join("nested.mp4");
    let rendered = render(canonical, &BTreeMap::new(), &output);

    assert_eq!(rendered.plan.inputs.len(), 0);
    assert_eq!(rendered.plan.sequences.len(), 2);
    assert_eq!(rendered.plan.sequences[0].id.as_str(), "seq_child");
    assert_eq!(rendered.plan.sequences[1].id.as_str(), "seq_main");
    assert_eq!(rendered.plan.sequences[0].settings.width, 48);
    assert_eq!(rendered.plan.sequences[0].settings.height, 36);
    assert_eq!(rendered.plan.sequences[0].settings.frame_rate, ratio(5, 1));
    assert_media_contract(&output, 0, 2.0);
    assert_blue(rgb_at(&output, 0.25, WIDTH / 2, HEIGHT / 2));
    let nested = rgb_at(&output, 0.75, WIDTH / 2, HEIGHT / 2);
    assert!(nested[0] > 180 && nested[2] < 40, "nested={nested:?}");
    assert_blue(rgb_at(&output, 0.75, 30, HEIGHT / 2));
    assert_blue(rgb_at(&output, 1.75, WIDTH / 2, HEIGHT / 2));
    let graph = rendered.command.filter_graph.as_deref().unwrap();
    assert!(graph.contains("s=48x36:r=5/1"), "{graph}");
}

#[test]
fn nested_sequence_reverse_mapping_reverses_rendered_content() {
    let temp = tempdir().unwrap();
    let child = sequence(
        "seq_child_reverse",
        vec![track(
            "trk_child_reverse",
            TrackKind::Video,
            0,
            vec![
                solid_clip("itm_child_red", color(255, 0, 0), 0, 1_000),
                solid_clip("itm_child_blue", color(0, 0, 255), 1_000, 1_000),
            ],
        )],
    );
    let mut parent = nested_clip("itm_parent_reverse", "seq_child_reverse", 0, 2_000);
    let mut identity = project(false);
    identity.project.sequences[0].tracks = vec![track(
        "trk_parent_identity",
        TrackKind::Video,
        0,
        vec![parent.clone()],
    )];
    identity.project.sequences.push(child.clone());
    let identity_output = temp.path().join("nested-identity.mp4");
    render(identity, &BTreeMap::new(), &identity_output);
    let first = rgb_at(&identity_output, 0.25, WIDTH / 2, HEIGHT / 2);
    assert!(first[0] > 180 && first[2] < 40, "pixel={first:?}");
    assert_blue(rgb_at(&identity_output, 1.75, WIDTH / 2, HEIGHT / 2));

    let SourceTimeMap::Linear { direction, .. } =
        &mut parent.source_mapping.as_mut().unwrap().time_map
    else {
        panic!("linear mapping");
    };
    *direction = PlaybackDirection::Reverse;
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks = vec![track(
        "trk_parent_reverse",
        TrackKind::Video,
        0,
        vec![parent],
    )];
    canonical.project.sequences.push(child);
    let output = temp.path().join("nested-reverse.mp4");
    let rendered = render(canonical, &BTreeMap::new(), &output);

    let graph = rendered.command.filter_graph.unwrap();
    assert!(graph.contains("reverse"), "{graph}");
    let samples = [0.25, 0.75, 1.25, 1.75].map(|time| rgb_at(&output, time, WIDTH / 2, HEIGHT / 2));
    assert_blue(samples[0]);
    let red = samples[3];
    assert!(red[0] > 180 && red[2] < 40, "samples={samples:?}\n{graph}");
}

fn child_visual() -> VisualProperties {
    VisualProperties {
        placement: Placement::Anchor {
            anchor: Anchor::Center,
            inset: Vec2 { x: 0.0, y: 0.0 },
        },
        frame: Some(Frame {
            width: pixels(20.0),
            height: pixels(16.0),
            fit: FitMode::Fill,
        }),
        transform: Transform2D {
            position: Animatable::constant(Point {
                x: pixels(0.0),
                y: pixels(0.0),
            }),
            scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            shear: Vec2 { x: 0.0, y: 0.0 },
            flip_horizontal: false,
            flip_vertical: false,
            rotation_degrees: Animatable::constant(0.0),
            anchor: Vec2 { x: 0.5, y: 0.5 },
            crop: None,
        },
        opacity: Animatable::constant(1.0),
        compositing: Compositing {
            z_index: 0,
            blend_mode: BlendMode::Normal,
        },
        masks: Vec::new(),
        card: None,
        color_pipeline: None,
    }
}

fn pixels(value: f64) -> Length {
    Length {
        value,
        unit: LengthUnit::Pixels,
    }
}

fn assert_blue(pixel: [u8; 3]) {
    assert!(pixel[2] > 180 && pixel[0] < 40, "pixel={pixel:?}");
}
