use crate::*;

use super::eval_support::compare;

#[test]
fn point_and_rect_equality_remain_closed_and_typed() {
    let point = |unit, scale| TemporalValue::Point {
        value: Point {
            x: Length { value: scale, unit },
            y: Length {
                value: scale * 2.0,
                unit,
            },
        },
    };
    for (left, right) in [
        (
            point(LengthUnit::Normalized, 1.0),
            point(LengthUnit::Percent, 100.0),
        ),
        (
            TemporalValue::Rect {
                value: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 1.0,
                    height: 1.0,
                },
            },
            TemporalValue::Rect {
                value: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 1.0,
                    height: 1.0,
                },
            },
        ),
    ] {
        assert_eq!(
            compare(TemporalCompareOperation::Equal, left, right),
            TemporalValue::Boolean { value: true }
        );
    }
}
