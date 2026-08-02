use super::*;
use crate::test_support::{range, time};

#[test]
fn switch_partition_rejects_empty_short_long_gap_and_overlap_timelines() {
    let mut empty = multicam_project();
    switches_mut(&mut empty).clear();
    assert_multicam_code(&empty, "MULTICAM_SWITCH_PARTITION");

    for ranges in [
        [range(0, 200), range(200, 300)],
        [range(0, 300), range(300, 301)],
        [range(0, 200), range(300, 300)],
        [range(0, 350), range(300, 300)],
    ] {
        let mut project = multicam_project();
        let switches = switches_mut(&mut project);
        switches[0].range = ranges[0];
        switches[1].range = ranges[1];
        assert_multicam_code(&project, "MULTICAM_SWITCH_PARTITION");
    }
}

#[test]
fn switch_ranges_reject_negative_zero_mixed_timebase_and_overflow_values() {
    let invalid = [
        TimeRange {
            start: time(-1),
            duration: time(300),
        },
        TimeRange {
            start: time(0),
            duration: time(0),
        },
        TimeRange {
            start: RationalTime {
                value: 0,
                timescale: 1,
            },
            duration: RationalTime {
                value: 1,
                timescale: 1,
            },
        },
        TimeRange {
            start: time(MAX_SAFE_INTEGER as i64),
            duration: time(1),
        },
    ];
    for range in invalid {
        let mut project = multicam_project();
        switches_mut(&mut project)[0].range = range;
        assert_multicam_code(&project, "MULTICAM_SWITCH_RANGE");
    }
}
