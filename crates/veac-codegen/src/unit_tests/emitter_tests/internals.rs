use veac_plan::canonical::*;

use crate::emitter::{animation, audio, geometry, graph::Graph, process_owner::ProcessOwner, time};

use super::support::{fixture, resolved};

#[test]
fn exact_time_and_number_formatting_covers_boundaries() {
    assert_eq!(time::seconds(value(1_200, 600)), "2");
    assert_eq!(time::seconds(value(1, 3)), "0.333333333333");
    assert_eq!(time::rate(Rational::new(2, 1).unwrap()), "2");
    assert_eq!(time::rate(Rational::new(1, 3).unwrap()), "0.333333333333");
    assert_eq!(time::number(-0.0), "0");
    assert_eq!(time::seconds_delta(value(1, 4), value(3, 4)), "0.5");
    let adjacent = [
        time::seconds(value(9_007_199_254_740_990, u32::MAX)),
        time::seconds(value(9_007_199_254_740_991, u32::MAX)),
    ];
    assert_ne!(adjacent[0], adjacent[1]);
    assert_eq!(
        time::seconds_delta(
            value(9_007_199_254_740_990, u32::MAX),
            value(9_007_199_254_740_991, u32::MAX),
        ),
        "0.000000000233"
    );
    assert_eq!(time::samples(value(1, 2), 48_000), Some(24_000));
    assert_eq!(time::samples(value(1, 3), 48_000), Some(16_000));
    assert_eq!(time::samples(value(1, 600), 44_100), None);
    assert_eq!(time::samples(value(-1, 600), 48_000), None);
    assert_eq!(time::samples(value(1, 0), 48_000), None);
    assert_eq!(time::samples(value(i64::MAX, 1), u32::MAX), None);
    let overflow = TimeRange {
        start: value(i64::MAX, 1),
        duration: value(1, 1),
    };
    assert_eq!(time::end(overflow), "0");
}

#[test]
fn animation_expressions_cover_every_interpolation_and_unit() {
    let plan = resolved(&fixture());
    let clip = &plan.sequences[0].tracks[0].clips[0];
    let owner = ProcessOwner::clip(clip);
    let cases = [
        (Interpolation::Hold, "*(0)"),
        (Interpolation::Linear, "clip("),
        (Interpolation::EaseIn, "\\,2)"),
        (Interpolation::EaseOut, "1-pow"),
        (Interpolation::EaseInOut, "*(3-2*("),
        (
            Interpolation::CubicBezier {
                x1: 0.2,
                y1: 0.1,
                x2: 0.8,
                y2: 0.9,
            },
            "root(",
        ),
    ];
    for (index, (interpolation, marker)) in cases.into_iter().enumerate() {
        let curve = Animatable::Keyframes {
            keyframes: vec![
                frame(&format!("kf_i{index}_a"), 0, 0.0, interpolation),
                frame(&format!("kf_i{index}_b"), 600, 1.0, Interpolation::Linear),
            ],
        };
        let expression = animation::number(&plan, owner, &curve, "t");
        assert!(expression.contains(marker), "{expression}");
        if index == 5 {
            assert!(!expression.contains("0.083333333333"), "{expression}");
            assert!(expression.contains("ld(0)"), "{expression}");
            assert!(expression.contains("st(1\\,root("), "{expression}");
        }
    }
    assert_eq!(
        animation::number(
            &plan,
            owner,
            &Animatable::Keyframes { keyframes: vec![] },
            "t",
        ),
        "0"
    );
    let vector = Animatable::constant(Vec2 { x: 2.0, y: 3.0 });
    assert_eq!(animation::vec_x(&plan, owner, &vector, "t"), "2");
    assert_eq!(animation::vec_y(&plan, owner, &vector, "t"), "3");
    for (unit, marker) in [
        (LengthUnit::Pixels, "2"),
        (LengthUnit::Normalized, "W*2"),
        (LengthUnit::Percent, "W*2/100"),
    ] {
        let point = Animatable::constant(Point {
            x: Length { value: 2.0, unit },
            y: Length { value: 2.0, unit },
        });
        assert_eq!(animation::point_x(&plan, owner, &point, "t", "W"), marker);
        assert!(animation::point_y(&plan, owner, &point, "t", "H").contains('2'));
    }
}

#[test]
fn geometry_graph_and_channel_helpers_cover_all_shapes() {
    for (unit, expected) in [
        (LengthUnit::Pixels, "20"),
        (LengthUnit::Normalized, "200"),
        (LengthUnit::Percent, "2"),
    ] {
        let length = Length { value: 20.0, unit };
        assert_eq!(time::number(geometry::pixel_value(length, 10)), expected);
        assert!(geometry::pixel_count(length, 10) >= 1);
    }
    assert_eq!(
        geometry::pixel_count(
            Length {
                value: -5.0,
                unit: LengthUnit::Pixels,
            },
            10,
        ),
        1
    );
    assert_eq!(audio::channel_layout(1), "mono");
    assert_eq!(audio::channel_layout(2), "stereo");
    assert_eq!(audio::channel_layout(6), "6c");
    let mut graph = Graph::default();
    assert!(graph.is_empty());
    let source = graph.source("color=black", "source");
    let outputs = graph.filter_many(&[&source], "split=2", "split", 2);
    let _ = graph.split(&outputs[0], "again");
    assert_eq!(outputs.len(), 2);
    assert!(graph.render_with_inputs(&[], &[]).contains("split=2"));
    assert!(!graph.is_empty());

    let mut external = Graph::default();
    external.filter(&["0:1"], "anull", "firsta");
    external.filter(&["0:1"], "anull", "seconda");
    external.filter(&["1:0"], "null", "firstv");
    external.filter(&["1:0"], "null", "secondv");
    external.filter(&["2:0"], "null", "singlev");
    let rendered =
        external.render_with_inputs(&["1:0".to_owned(), "2:0".to_owned()], &["0:1".to_owned()]);
    assert!(rendered.contains("[0:1]asplit=2[inputsplita5][inputsplita6]"));
    assert!(rendered.contains("[1:0]split=2[inputsplitv7][inputsplitv8]"));
    assert_eq!(rendered.matches("[2:0]").count(), 1);
}

fn value(value: i64, timescale: u32) -> RationalTime {
    RationalTime { value, timescale }
}

fn frame(id: &str, ticks: i64, value: f64, interpolation: Interpolation) -> Keyframe<f64> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: RationalTime::new(ticks, 600).unwrap(),
        value,
        interpolation,
    }
}
