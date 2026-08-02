use tempfile::tempdir;

use super::support::*;

#[test]
fn time_disjoint_crossing_bands_preserve_both_results() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("apply-crossing.mp4");
    let mut project = layered_project();
    project.project.sequences[0].applies.extend([
        apply(
            "apl_cross_lower",
            band("trk_stack_lower", "trk_stack_inner_cap"),
            250,
            500,
            vec![brightness_stage("aps_cross_lower", 0.2)],
        ),
        apply(
            "apl_cross_upper",
            band("trk_stack_upper", "trk_stack_outer_cap"),
            1_000,
            500,
            vec![brightness_stage("aps_cross_upper", -0.15)],
        ),
    ]);
    render(project, &BTreeMap::new(), &output);

    let baseline = samples(&output, 0.1);
    let lower_active = samples(&output, 0.5);
    let upper_active = samples(&output, 1.25);
    let after = samples(&output, 1.8);
    assert_changed(baseline.0, lower_active.0);
    assert_changed(baseline.1, lower_active.1);
    assert_close(baseline.0, upper_active.0);
    assert_changed(baseline.1, upper_active.1);
    assert_close(baseline.0, after.0);
    assert_close(baseline.1, after.1);
}

#[test]
fn nested_bands_apply_inner_then_outer_in_sequence_order() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("apply-nested.mp4");
    let mut project = layered_project();
    project.project.sequences[0].applies.extend([
        apply(
            "apl_inner",
            band("trk_stack_upper", "trk_stack_inner_cap"),
            500,
            1_000,
            vec![brightness_stage("aps_inner", 0.1)],
        ),
        apply(
            "apl_outer",
            band("trk_stack_lower", "trk_stack_outer_cap"),
            500,
            1_000,
            vec![brightness_stage("aps_outer", 0.1)],
        ),
    ]);
    render(project, &BTreeMap::new(), &output);

    let baseline = samples(&output, 0.25);
    let active = samples(&output, 1.0);
    let lower = distance(baseline.0, active.0);
    let upper = distance(baseline.1, active.1);
    assert!(lower > 20, "lower={lower}");
    assert!(upper > lower + 20, "lower={lower}, upper={upper}");
    let after = samples(&output, 1.75);
    assert_close(baseline.0, after.0);
    assert_close(baseline.1, after.1);
}

#[test]
fn same_band_applies_execute_serially() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("apply-same-band.mp4");
    let mut project = layered_project();
    let target = ApplyTarget::Layer {
        track_id: TrackId::new("trk_stack_lower").unwrap(),
    };
    project.project.sequences[0].applies.extend([
        apply(
            "apl_same_first",
            target.clone(),
            500,
            1_000,
            vec![brightness_stage("aps_same_first", 0.1)],
        ),
        apply(
            "apl_same_second",
            target,
            500,
            1_000,
            vec![brightness_stage("aps_same_second", 0.1)],
        ),
    ]);
    let rendered = render(project, &BTreeMap::new(), &output);
    let graph = rendered.command.filter_graph.unwrap();
    assert_eq!(graph.matches("eq=brightness").count(), 2, "{graph}");
    let baseline = samples(&output, 0.25);
    let active = samples(&output, 1.0);
    assert_changed(baseline.0, active.0);
    assert_close(baseline.1, active.1);
}

fn layered_project() -> ProjectEnvelope {
    let mut project = project(false);
    let underlay = solid_clip("itm_stack_under", color(20, 20, 100), 0, 2_000);
    let mut lower = solid_clip("itm_stack_lower", color(70, 20, 20), 0, 2_000);
    lower.visual = Some(framed_visual(Anchor::TopLeft, Some((32.0, 20.0))));
    let mut upper = solid_clip("itm_stack_upper", color(20, 70, 20), 0, 2_000);
    upper.visual = Some(framed_visual(Anchor::TopRight, Some((32.0, 20.0))));
    let mut inner_cap = solid_clip("itm_stack_inner_cap", color(70, 70, 20), 0, 2_000);
    inner_cap.visual = Some(framed_visual(Anchor::BottomLeft, Some((16.0, 12.0))));
    let mut outer_cap = solid_clip("itm_stack_outer_cap", color(20, 70, 70), 0, 2_000);
    outer_cap.visual = Some(framed_visual(Anchor::BottomRight, Some((16.0, 12.0))));
    project.project.sequences[0].tracks.extend([
        track("trk_stack_under", TrackKind::Video, 0, vec![underlay]),
        track("trk_stack_lower", TrackKind::Visual, 1, vec![lower]),
        track("trk_stack_upper", TrackKind::Visual, 2, vec![upper]),
        track("trk_stack_inner_cap", TrackKind::Visual, 3, vec![inner_cap]),
        track("trk_stack_outer_cap", TrackKind::Visual, 4, vec![outer_cap]),
    ]);
    project
}

fn band(from: &str, through: &str) -> ApplyTarget {
    ApplyTarget::CompositeBand {
        from_track_id: TrackId::new(from).unwrap(),
        through_track_id: TrackId::new(through).unwrap(),
    }
}

fn samples(output: &Path, second: f64) -> ([u8; 3], [u8; 3]) {
    (rgb_at(output, second, 5, 5), rgb_at(output, second, 90, 5))
}

fn distance(left: [u8; 3], right: [u8; 3]) -> u16 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| u16::from(left.abs_diff(right)))
        .sum()
}

fn assert_changed(left: [u8; 3], right: [u8; 3]) {
    assert!(distance(left, right) > 25, "left={left:?}, right={right:?}");
}

fn assert_close(left: [u8; 3], right: [u8; 3]) {
    assert!(
        distance(left, right) <= 18,
        "left={left:?}, right={right:?}"
    );
}
