use crate::edit::ChangeSet;

use super::*;

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 100).unwrap()
}

fn numbers() -> Animatable<f64> {
    Animatable::Keyframes {
        keyframes: vec![
            key("kf_zero", 0, 0.0),
            key("kf_middle", 50, 0.5),
            key("kf_end", 100, 1.0),
        ],
    }
}

#[test]
fn constants_and_full_curve_slices_are_noops() {
    let owner = ItemId::new("itm_owner").unwrap();
    let mut changed = ChangeSet::new();
    let mut constant = Animatable::constant(0.5);
    crop_animatable(
        &mut constant,
        time(0),
        time(100),
        &owner,
        "opacity",
        false,
        &mut changed,
    )
    .unwrap();
    assert_eq!(constant, Animatable::constant(0.5));

    let mut curve = numbers();
    let original = curve.clone();
    crop_animatable(
        &mut curve,
        time(0),
        time(100),
        &owner,
        "opacity",
        false,
        &mut changed,
    )
    .unwrap();
    assert_eq!(curve, original);
    assert!(changed.is_empty());
}

#[test]
fn interior_slice_synthesizes_boundaries_and_shifts_inside_keys() {
    let owner = ItemId::new("itm_owner").unwrap();
    let mut changed = ChangeSet::new();
    let mut curve = numbers();
    crop_animatable(
        &mut curve,
        time(25),
        time(75),
        &owner,
        "opacity",
        false,
        &mut changed,
    )
    .unwrap();
    let keys = curve.keyframes().unwrap();
    assert_eq!(keys.len(), 3);
    assert_eq!((keys[0].time, keys[0].value), (time(0), 0.25));
    assert_eq!((keys[1].time, keys[1].value), (time(25), 0.5));
    assert_eq!((keys[2].time, keys[2].value), (time(50), 0.75));
    assert_eq!(keys[0].interpolation, Interpolation::Linear);
    assert_eq!(keys[1].id.as_str(), "kf_middle");
    assert_eq!(changed.len(), 5);
}

#[test]
fn reidentified_and_zero_width_slices_have_stable_unique_ids() {
    let owner = ItemId::new("itm_clone").unwrap();
    let mut changed = ChangeSet::new();
    let mut curve = numbers();
    crop_animatable(
        &mut curve,
        time(0),
        time(100),
        &owner,
        "opacity",
        true,
        &mut changed,
    )
    .unwrap();
    let keys = curve.keyframes().unwrap();
    for (key, original) in keys.iter().zip(["kf_zero", "kf_middle", "kf_end"]) {
        assert_ne!(key.id.as_str(), original);
    }
    assert_eq!(changed.len(), 6);

    let mut collapsed = numbers();
    crop_animatable(
        &mut collapsed,
        time(50),
        time(50),
        &owner,
        "opacity",
        false,
        &mut ChangeSet::new(),
    )
    .unwrap();
    let keys = collapsed.keyframes().unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!((keys[0].time, keys[0].value), (time(0), 0.5));
}

#[test]
fn empty_and_incompatible_curves_fail_closed() {
    let owner = ItemId::new("itm_owner").unwrap();
    let mut empty = Animatable::<f64>::Keyframes { keyframes: vec![] };
    assert!(crop_animatable(
        &mut empty,
        time(0),
        time(10),
        &owner,
        "opacity",
        false,
        &mut ChangeSet::new(),
    )
    .is_err());

    let mut points = Animatable::Keyframes {
        keyframes: vec![
            point_key("kf_left", 0, LengthUnit::Pixels),
            point_key("kf_right", 100, LengthUnit::Normalized),
        ],
    };
    assert!(crop_animatable(
        &mut points,
        time(0),
        time(50),
        &owner,
        "position",
        false,
        &mut ChangeSet::new(),
    )
    .is_err());
}

#[test]
fn a_boundary_before_the_first_key_uses_hold_interpolation() {
    let owner = ItemId::new("itm_owner").unwrap();
    let mut curve = numbers();
    crop_animatable(
        &mut curve,
        time(-10),
        time(0),
        &owner,
        "opacity",
        false,
        &mut ChangeSet::new(),
    )
    .unwrap();
    assert_eq!(
        curve.keyframes().unwrap()[0].interpolation,
        Interpolation::Hold
    );
}

fn key(id: &str, at: i64, value: f64) -> Keyframe<f64> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value,
        interpolation: Interpolation::Linear,
    }
}

fn point_key(id: &str, at: i64, unit: LengthUnit) -> Keyframe<Point> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value: Point {
            x: Length { value: 0.0, unit },
            y: Length { value: 0.0, unit },
        },
        interpolation: Interpolation::Linear,
    }
}
