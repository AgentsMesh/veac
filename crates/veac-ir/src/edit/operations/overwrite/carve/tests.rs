use std::collections::BTreeMap;

use super::*;

#[test]
fn invalid_clip_ranges_and_timebases_fail_closed() {
    let clip = overflowing_clip();
    let start = crate::test_support::time(100);
    let end = crate::test_support::time(200);
    let mut result = Vec::new();
    let mut rewrites = Vec::new();
    let mut changed = ChangeSet::new();

    assert!(carve(
        clip.clone(),
        start,
        end,
        &BTreeMap::new(),
        &mut result,
        &mut rewrites,
        &mut changed,
    )
    .is_err());
    assert!(split_around(
        clip,
        start,
        end,
        &ItemId::new("itm_right").unwrap(),
        &mut result,
        &mut changed,
    )
    .is_err());
    assert!(zero(
        RationalTime {
            value: 0,
            timescale: 0,
        },
        &ItemId::new("itm_invalid_timebase").unwrap(),
    )
    .is_err());
}

fn overflowing_clip() -> Clip {
    let envelope = crate::test_support::sample_project();
    let mut clip = envelope.project.sequences[0].tracks[0].clips[0].clone();
    clip.record_range = TimeRange {
        start: RationalTime {
            value: crate::MAX_SAFE_INTEGER as i64,
            timescale: 600,
        },
        duration: RationalTime {
            value: 1,
            timescale: 600,
        },
    };
    clip
}
