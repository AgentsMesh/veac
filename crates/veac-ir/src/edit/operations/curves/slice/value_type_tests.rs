use super::*;
use crate::test_support::time;

fn keyframed<T>(values: [T; 3]) -> Animatable<T> {
    Animatable::Keyframes {
        keyframes: values
            .into_iter()
            .enumerate()
            .map(|(index, value)| Keyframe {
                id: KeyframeId::new(format!("kf_types_{index:03}")).expect("keyframe id"),
                time: time((index as i64) * 100),
                value,
                interpolation: Interpolation::Linear,
            })
            .collect(),
    }
}

fn assert_sliced<T>(value: &Animatable<T>) {
    let Animatable::Keyframes { keyframes } = value else {
        panic!("expected keyframes");
    };
    assert_eq!(keyframes.len(), 3);
    assert_eq!(keyframes[0].time, time(0));
    assert_eq!(keyframes[1].time, time(60));
    assert_eq!(keyframes[2].time, time(120));
}

fn length(value: f64) -> Length {
    Length {
        value,
        unit: LengthUnit::Pixels,
    }
}

#[test]
fn slices_every_animatable_geometry_value_type() {
    let mut scalar = keyframed([0.0, 1.0, 2.0]);
    let mut point = keyframed([
        Point {
            x: length(0.0),
            y: length(0.0),
        },
        Point {
            x: length(1.0),
            y: length(1.0),
        },
        Point {
            x: length(2.0),
            y: length(2.0),
        },
    ]);
    let mut vector = keyframed([
        Vec2 { x: 0.0, y: 0.0 },
        Vec2 { x: 1.0, y: 1.0 },
        Vec2 { x: 2.0, y: 2.0 },
    ]);
    let mut rect = keyframed([
        Rect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        },
        Rect {
            x: 1.0,
            y: 1.0,
            width: 11.0,
            height: 11.0,
        },
        Rect {
            x: 2.0,
            y: 2.0,
            width: 12.0,
            height: 12.0,
        },
    ]);
    let owner = ItemId::new("itm_slice_types").expect("owner id");

    let mut changed = ChangeSet::default();
    crop_animatable(
        &mut scalar,
        time(40),
        time(160),
        &owner,
        "opacity",
        true,
        &mut changed,
    )
    .expect("scalar slice");
    crop_animatable(
        &mut point,
        time(40),
        time(160),
        &owner,
        "position",
        true,
        &mut changed,
    )
    .expect("point slice");
    crop_animatable(
        &mut vector,
        time(40),
        time(160),
        &owner,
        "scale",
        true,
        &mut changed,
    )
    .expect("vector slice");
    crop_animatable(
        &mut rect,
        time(40),
        time(160),
        &owner,
        "crop",
        true,
        &mut changed,
    )
    .expect("rect slice");

    assert_sliced(&scalar);
    assert_sliced(&point);
    assert_sliced(&vector);
    assert_sliced(&rect);
}
