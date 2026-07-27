use tempfile::tempdir;

use super::support::*;

#[test]
fn composite_band_respects_boundaries_stage_order_and_half_open_time() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("apply-band.mp4");
    let mut project = project(false);
    let underlay = solid_clip("itm_band_under", color(20, 20, 120), 0, 2_000);
    let mut target = solid_clip("itm_band_target", color(80, 20, 20), 0, 2_000);
    target.visual = Some(framed_visual(Anchor::TopLeft, Some((32.0, 20.0))));
    let mut middle = solid_clip("itm_band_middle", color(20, 80, 20), 0, 2_000);
    middle.visual = Some(framed_visual(Anchor::TopRight, Some((32.0, 20.0))));
    let mut upper = solid_clip("itm_band_upper", color(180, 180, 20), 0, 2_000);
    upper.visual = Some(framed_visual(Anchor::BottomRight, Some((20.0, 16.0))));
    let sequence = &mut project.project.sequences[0];
    sequence.tracks.extend([
        track("trk_band_under", TrackKind::Video, 0, vec![underlay]),
        track("trk_band_target", TrackKind::Visual, 1, vec![target]),
        track("trk_band_middle", TrackKind::Visual, 2, vec![middle]),
        track("trk_band_upper", TrackKind::Visual, 3, vec![upper]),
    ]);
    sequence.applies.push(apply(
        "apl_band",
        ApplyTarget::CompositeBand {
            from_track_id: TrackId::new("trk_band_target").unwrap(),
            through_track_id: TrackId::new("trk_band_middle").unwrap(),
        },
        500,
        1_000,
        vec![
            color_stage("aps_band_color", exposure_pipeline()),
            brightness_stage("aps_band_effect", 0.15),
        ],
    ));

    let rendered = render(project, &BTreeMap::new(), &output);
    let graph = rendered.command.filter_graph.unwrap();
    assert!(
        graph.find("exposurev") < graph.find("eq=brightness"),
        "{graph}"
    );
    let before = samples(&output, 0.4);
    let first = samples(&output, 0.5);
    let last = samples(&output, 1.4);
    let after = samples(&output, 1.5);
    assert_changed(before.target, first.target);
    assert_changed(before.middle, first.middle);
    assert_changed(before.target, last.target);
    assert_close(before.target, after.target);
    assert_close(before.middle, after.middle);
    assert_close(before.underlay, first.underlay);
    assert_close(before.upper, first.upper);
}

#[test]
fn layer_target_changes_only_the_named_layer() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("apply-layer.mp4");
    let mut project = project(false);
    let mut left = solid_clip("itm_layer_left", color(50, 20, 20), 0, 2_000);
    left.visual = Some(framed_visual(Anchor::TopLeft, Some((40.0, 30.0))));
    let mut right = solid_clip("itm_layer_right", color(20, 50, 20), 0, 2_000);
    right.visual = Some(framed_visual(Anchor::TopRight, Some((40.0, 30.0))));
    let sequence = &mut project.project.sequences[0];
    sequence.tracks.extend([
        track("trk_layer_left", TrackKind::Visual, 0, vec![left]),
        track("trk_layer_right", TrackKind::Visual, 1, vec![right]),
    ]);
    sequence.applies.push(apply(
        "apl_layer",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_layer_left").unwrap(),
        },
        500,
        1_000,
        vec![brightness_stage("aps_layer", 0.25)],
    ));
    render(project, &BTreeMap::new(), &output);
    let before = pair(&output, 0.25);
    let during = pair(&output, 1.0);
    assert_changed(before.0, during.0);
    assert_close(before.1, during.1);
}

fn exposure_pipeline() -> ColorPipeline {
    ColorPipeline {
        input: rec709(),
        working: rec709(),
        output: rec709(),
        stages: vec![ColorStage::Basic {
            adjustment: BasicColorAdjustment {
                exposure_stops: 0.5,
                temperature_kelvin: 6_500.0,
                tint: 0.0,
                highlights: 0.0,
                shadows: 0.0,
                fade: 0.0,
            },
        }],
    }
}

struct Samples {
    target: [u8; 3],
    middle: [u8; 3],
    underlay: [u8; 3],
    upper: [u8; 3],
}

fn samples(output: &Path, second: f64) -> Samples {
    Samples {
        target: rgb_at(output, second, 5, 5),
        middle: rgb_at(output, second, 90, 5),
        underlay: rgb_at(output, second, WIDTH / 2, HEIGHT / 2),
        upper: rgb_at(output, second, 90, 48),
    }
}

fn pair(output: &Path, second: f64) -> ([u8; 3], [u8; 3]) {
    (rgb_at(output, second, 5, 5), rgb_at(output, second, 90, 5))
}

fn assert_changed(left: [u8; 3], right: [u8; 3]) {
    assert!(distance(left, right) > 35, "left={left:?}, right={right:?}");
}

fn assert_close(left: [u8; 3], right: [u8; 3]) {
    assert!(
        distance(left, right) <= 18,
        "left={left:?}, right={right:?}"
    );
}

fn distance(left: [u8; 3], right: [u8; 3]) -> u16 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| u16::from(left.abs_diff(right)))
        .sum()
}
