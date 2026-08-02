use crate::test_support::time;

use super::*;

fn key(id: &str, at: i64, value: f64) -> Keyframe<f64> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value,
        interpolation: Interpolation::Linear,
    }
}

#[test]
fn mutation_helpers_distinguish_absent_identical_and_conflicting_keys() {
    let mut constant = Animatable::constant(1.0);
    let absent = KeyframeId::new("kf_absent").unwrap();
    assert!(!remove(&mut constant, &absent).unwrap());
    assert_eq!(move_time(&mut constant, &absent, time(1)).unwrap(), None);

    let first = key("kf_first", 0, 1.0);
    assert!(upsert(&mut constant, first.clone()).unwrap());
    assert!(!upsert(&mut constant, first.clone()).unwrap());
    assert!(!remove(&mut constant, &absent).unwrap());
    assert_eq!(move_time(&mut constant, &absent, time(1)).unwrap(), None);
    assert_eq!(
        move_time(&mut constant, &first.id, first.time).unwrap(),
        Some(false)
    );

    assert!(upsert(&mut constant, key("kf_second", 10, 2.0)).unwrap());
    assert!(upsert(&mut constant, key("kf_collision", 10, 3.0)).is_err());
    assert!(move_time(&mut constant, &first.id, time(10)).is_err());
    assert!(remove(&mut constant, &first.id).unwrap());
}

fn rect_key(id: &str, at: i64, value: f64) -> Keyframe<Rect> {
    Keyframe {
        id: KeyframeId::new(id).expect("keyframe id"),
        time: time(at),
        value: Rect {
            x: value,
            y: value,
            width: value + 10.0,
            height: value + 10.0,
        },
        interpolation: Interpolation::Linear,
    }
}

#[test]
fn mutation_helpers_cover_crop_rectangle_keyframes() {
    let first = rect_key("kf_rect_first", 0, 1.0);
    let absent = KeyframeId::new("kf_rect_absent").expect("keyframe id");
    let mut crop = Animatable::constant(first.value);

    assert!(upsert(&mut crop, first.clone()).expect("insert first"));
    assert!(!upsert(&mut crop, first.clone()).expect("identical upsert"));
    assert!(!remove(&mut crop, &absent).expect("absent removal"));
    assert_eq!(
        move_time(&mut crop, &absent, time(1)).expect("absent move"),
        None
    );
    assert!(upsert(&mut crop, rect_key("kf_rect_second", 10, 2.0)).unwrap());
    assert!(upsert(&mut crop, rect_key("kf_rect_collision", 10, 3.0)).is_err());
    assert!(move_time(&mut crop, &first.id, time(10)).is_err());
    assert!(remove(&mut crop, &first.id).expect("remove first"));
}
