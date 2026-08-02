use super::support::*;
use crate::{canonical::*, resolve};

#[test]
fn disabled_tracks_preserve_authored_sequence_duration() {
    let mut value = project();
    value.project.sequences[0].tracks[0].state.enabled = false;

    let plan = resolve(&value, None).unwrap().remove(0);

    assert_eq!(plan.sequences[0].duration, time(600));
    assert!(plan.sequences[0].tracks[0].clips.is_empty());
}

#[test]
fn solo_filter_does_not_shorten_authored_sequence_duration() {
    let mut value = project();
    value.project.sequences[0].tracks[0].state.solo = true;
    value.project.sequences[0].tracks.push(track(
        "trk_long_suppressed",
        TrackKind::Visual,
        10,
        vec![generated_clip(
            "itm_long_suppressed",
            Generator::Transparent,
            600,
        )],
    ));

    let plan = resolve(&value, None).unwrap().remove(0);

    assert_eq!(plan.sequences[0].duration, time(900));
    assert!(plan.sequences[0].tracks[1].clips.is_empty());
}
