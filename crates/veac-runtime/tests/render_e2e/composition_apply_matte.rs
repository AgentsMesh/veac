use tempfile::tempdir;

use super::support::*;

#[test]
fn apply_effect_is_limited_by_its_matte_and_the_producer_stays_hidden() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("apply-matte.mp4");
    let mut project = project(false);
    let mut target = solid_clip("itm_apply_matte_target", color(35, 60, 95), 0, 2_000);
    target.visual = Some(full_visual());
    let mut matte = solid_clip("itm_apply_matte_source", color(20, 220, 20), 0, 2_000);
    matte.visual = Some(left_half_visual());
    let sequence = &mut project.project.sequences[0];
    sequence.tracks.extend([
        track("trk_apply_matte_target", TrackKind::Video, 0, vec![target]),
        track("trk_apply_matte_source", TrackKind::Visual, 1, vec![matte]),
    ]);
    sequence.applies.push(apply(
        "apl_matte",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_apply_matte_target").unwrap(),
        },
        500,
        1_000,
        vec![brightness_stage("aps_matte", 0.25)],
    ));
    add_apply_matte(
        &mut project,
        "seq_main",
        "itm_apply_matte_source",
        "apl_matte",
        TrackMatteMode::Alpha,
        false,
    );

    let rendered = render(project, &BTreeMap::new(), &output);
    let graph = rendered.command.filter_graph.unwrap();
    assert!(graph.contains("applystagev") && graph.contains("mattemergev"));
    let before = pair(&output, 0.25);
    let during = pair(&output, 1.0);
    let after = pair(&output, 1.75);
    assert_close(before.left, before.right);
    assert!(
        distance(before.left, during.left) > 50,
        "matte side unchanged: before={:?}, during={:?}",
        before.left,
        during.left,
    );
    assert_close(before.right, during.right);
    assert_close(before.left, after.left);
    assert_close(before.right, after.right);
}

fn left_half_visual() -> VisualProperties {
    let mut visual = framed_visual(Anchor::Center, Some((48.0, 54.0)));
    visual.placement = Placement::Absolute {
        position: Point {
            x: pixels(24.0),
            y: pixels(27.0),
        },
    };
    visual
}

struct Pair {
    left: [u8; 3],
    right: [u8; 3],
}

fn pair(output: &Path, second: f64) -> Pair {
    Pair {
        left: rgb_at(output, second, 16, HEIGHT / 2),
        right: rgb_at(output, second, 80, HEIGHT / 2),
    }
}

fn distance(left: [u8; 3], right: [u8; 3]) -> u16 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| u16::from(left.abs_diff(right)))
        .sum()
}

fn assert_close(left: [u8; 3], right: [u8; 3]) {
    assert!(
        distance(left, right) <= 18,
        "left={left:?}, right={right:?}"
    );
}
