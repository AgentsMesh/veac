use tempfile::tempdir;

use super::support::*;

#[test]
fn true_overlap_uses_moving_frames_from_both_endpoints() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("transition-motion.mp4");
    let rendered = render(motion_project(), &BTreeMap::new(), &output);
    let graph = rendered.command.filter_graph.as_deref().unwrap();
    assert!(!graph.contains("transitionoutboundv"), "{graph}");
    assert!(!graph.contains("transitioninboundv"), "{graph}");

    let early = rgb_frame(&output, 0.85);
    let late = rgb_frame(&output, 1.10);
    let early_green = channel_center(&early, 1);
    let late_green = channel_center(&late, 1);
    let early_blue = channel_center(&early, 2);
    let late_blue = channel_center(&late, 2);
    assert!(
        late_green > early_green + 7.0,
        "green={early_green}..{late_green}"
    );
    assert!(
        late_blue > early_blue + 7.0,
        "blue={early_blue}..{late_blue}"
    );
}

fn motion_project() -> ProjectEnvelope {
    let mut value = project(false);
    let background = solid_clip("itm_bg", color(0, 0, 0), 0, 2_000);
    let outgoing = subject("itm_out", color(0, 255, 0), 0, -30.0, 30.0);
    let incoming = subject("itm_in", color(0, 0, 255), 750, -30.0, 30.0);
    value.project.sequences[0].tracks.extend([
        track("trk_bg", TrackKind::Video, 0, vec![background]),
        track("trk_motion", TrackKind::Visual, 1, vec![outgoing, incoming]),
    ]);
    add_transition(
        &mut value,
        "seq_main",
        "itm_out",
        "itm_in",
        Transition {
            kind: TransitionKind::Dissolve,
            duration: time(500),
            alignment: TransitionAlignment::Centered,
        },
    );
    value
}

fn subject(id: &str, color: Color, start: i64, from: f64, to: f64) -> Clip {
    let mut clip = solid_clip(id, color, start, 1_250);
    let mut visual = framed_visual(Anchor::Center, Some((12.0, 12.0)));
    visual.transform.position = Animatable::Keyframes {
        keyframes: vec![
            point_key(id, "start", 0, from),
            point_key(id, "end", 1_250, to),
        ],
    };
    clip.visual = Some(visual);
    clip
}

fn point_key(id: &str, suffix: &str, at: i64, x: f64) -> Keyframe<Point> {
    Keyframe {
        id: KeyframeId::new(format!("kf_{id}_{suffix}")).unwrap(),
        time: time(at),
        value: Point {
            x: pixels(x),
            y: pixels(0.0),
        },
        interpolation: Interpolation::Linear,
    }
}

fn channel_center(frame: &[u8], channel: usize) -> f64 {
    let mut weighted_x = 0.0;
    let mut weight = 0.0;
    for (index, pixel) in frame.chunks_exact(3).enumerate() {
        let primary = f64::from(pixel[channel]);
        let competing = f64::from(pixel[(channel + 1) % 3].max(pixel[(channel + 2) % 3]));
        let contribution = (primary - competing - 2.0).max(0.0);
        weighted_x += f64::from(index as u32 % WIDTH) * contribution;
        weight += contribution;
    }
    assert!(weight > 100.0, "channel {channel} marker is missing");
    weighted_x / weight
}
