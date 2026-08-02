use crate::edit::ChangeSet;
use crate::test_support::{sample_project, time};

use super::*;

#[test]
fn add_and_subtract_propagate_mismatch_and_overflow() {
    assert_eq!(add(time(7), time(5), "itm").unwrap(), time(12));
    assert_eq!(subtract(time(7), time(5), "itm").unwrap(), time(2));

    let other_scale = RationalTime::new(1, 1_000).unwrap();
    assert!(add(time(1), other_scale, "itm").is_err());
    assert!(subtract(time(1), other_scale, "itm").is_err());

    let maximum = crate::MAX_SAFE_INTEGER as i64;
    assert!(add(time(maximum), time(1), "itm").is_err());
    let minimum = RationalTime {
        value: i64::MIN,
        timescale: 600,
    };
    let error = subtract(time(0), minimum, "itm").unwrap_err();
    assert!(error.message.contains("overflowed"));
}

#[test]
fn scale_requires_exact_safe_results() {
    assert_eq!(
        scale(time(6), Rational::new(3, 2).unwrap(), "itm").unwrap(),
        time(9)
    );
    assert_eq!(
        scale(time(6), Rational::new(-1, 2).unwrap(), "itm").unwrap(),
        time(-3)
    );
    let error = scale(time(1), Rational::new(1, 2).unwrap(), "itm").unwrap_err();
    assert!(error.message.contains("not exact"));

    let maximum = crate::MAX_SAFE_INTEGER as i64;
    let unsafe_result = scale(time(maximum), Rational::new(2, 1).unwrap(), "itm").unwrap_err();
    assert!(unsafe_result.message.contains("safe range"));
    let conversion = scale(time(maximum), Rational::new(maximum, 1).unwrap(), "itm").unwrap_err();
    assert!(conversion.message.contains("overflowed"));
}

#[test]
fn shift_from_honors_threshold_exclusions_and_change_tracking() {
    let mut track = sample_project().project.sequences[0].tracks[0].clone();
    let mut middle = track.clips[0].clone();
    middle.id = ItemId::new("itm_middle").unwrap();
    middle.record_range.start = time(10);
    let mut last = track.clips[0].clone();
    last.id = ItemId::new("itm_last").unwrap();
    last.record_range.start = time(20);
    track.clips.extend([middle, last]);

    let excluded = ItemId::new("itm_middle").unwrap();
    let mut changed = ChangeSet::new();
    shift_from(&mut track, time(10), time(5), &[&excluded], &mut changed).unwrap();
    assert_eq!(track.clips[0].record_range.start, time(0));
    assert_eq!(track.clips[1].record_range.start, time(10));
    assert_eq!(track.clips[2].record_range.start, time(25));
    assert_eq!(changed.len(), 1);
    assert!(changed.contains(&ChangedObjectId::Item {
        id: ItemId::new("itm_last").unwrap(),
    }));
}

#[test]
fn shift_from_fails_before_marking_an_overflowed_clip() {
    let mut track = sample_project().project.sequences[0].tracks[0].clone();
    let maximum = crate::MAX_SAFE_INTEGER as i64;
    track.clips[0].record_range.start = time(maximum);
    let mut changed = ChangeSet::new();
    assert!(shift_from(&mut track, time(maximum), time(1), &[], &mut changed,).is_err());
    assert!(changed.is_empty());
}
