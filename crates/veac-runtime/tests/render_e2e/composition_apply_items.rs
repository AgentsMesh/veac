use tempfile::tempdir;

use super::support::*;

#[test]
fn item_set_selects_one_item_without_widening_to_its_track() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("apply-items-same-track.mp4");
    let mut project = project(false);
    let mut selected = solid_clip("itm_set_selected", color(55, 20, 20), 0, 2_000);
    selected.visual = Some(framed_visual(Anchor::TopLeft, Some((40.0, 30.0))));
    let mut sibling = solid_clip("itm_set_sibling", color(20, 55, 20), 0, 2_000);
    sibling.visual = Some(framed_visual(Anchor::TopRight, Some((40.0, 30.0))));
    let sequence = &mut project.project.sequences[0];
    sequence.tracks.push(track(
        "trk_set_shared",
        TrackKind::Visual,
        0,
        vec![selected, sibling],
    ));
    sequence.applies.push(apply(
        "apl_set_exact",
        ApplyTarget::ItemSet {
            item_ids: vec![ItemId::new("itm_set_selected").unwrap()],
        },
        500,
        1_000,
        vec![brightness_stage("aps_set_exact", 0.25)],
    ));
    render(project, &BTreeMap::new(), &output);
    let before = sample_three(&output, 0.25);
    let during = sample_three(&output, 1.0);
    assert_changed(before.0, during.0);
    assert_close(before.1, during.1);
}

#[test]
fn item_set_can_select_exact_items_across_tracks() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("apply-items-cross-track.mp4");
    let mut project = project(false);
    let mut left = solid_clip("itm_set_left", color(55, 20, 20), 0, 2_000);
    left.visual = Some(framed_visual(Anchor::TopLeft, Some((32.0, 20.0))));
    let mut right = solid_clip("itm_set_right", color(20, 55, 20), 0, 2_000);
    right.visual = Some(framed_visual(Anchor::TopRight, Some((32.0, 20.0))));
    let mut excluded = solid_clip("itm_set_excluded", color(20, 20, 80), 0, 2_000);
    excluded.visual = Some(framed_visual(Anchor::BottomLeft, Some((32.0, 20.0))));
    let sequence = &mut project.project.sequences[0];
    sequence.tracks.extend([
        track("trk_set_left", TrackKind::Visual, 0, vec![left]),
        track("trk_set_right", TrackKind::Visual, 1, vec![right]),
        track("trk_set_excluded", TrackKind::Visual, 2, vec![excluded]),
    ]);
    sequence.applies.push(apply(
        "apl_set_cross",
        ApplyTarget::ItemSet {
            item_ids: ["itm_set_left", "itm_set_right"]
                .map(|id| ItemId::new(id).unwrap())
                .to_vec(),
        },
        500,
        1_000,
        vec![brightness_stage("aps_set_cross", 0.25)],
    ));
    render(project, &BTreeMap::new(), &output);
    let before = sample_three(&output, 0.25);
    let during = sample_three(&output, 1.0);
    assert_changed(before.0, during.0);
    assert_changed(before.1, during.1);
    assert_close(before.2, during.2);
}

fn sample_three(output: &Path, second: f64) -> ([u8; 3], [u8; 3], [u8; 3]) {
    (
        rgb_at(output, second, 5, 5),
        rgb_at(output, second, 90, 5),
        rgb_at(output, second, 5, 48),
    )
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
