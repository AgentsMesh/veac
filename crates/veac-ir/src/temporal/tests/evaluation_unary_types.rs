use crate::*;

use super::eval_support::unary;

#[test]
fn remaining_typed_unary_forms_are_covered() {
    assert_eq!(
        unary(
            TemporalUnaryOperation::Negate,
            TemporalValue::Length {
                value: Length {
                    value: 2.0,
                    unit: LengthUnit::Percent
                }
            },
            TemporalType::Length,
        )
        .unwrap(),
        TemporalValue::Length {
            value: Length {
                value: -2.0,
                unit: LengthUnit::Percent
            }
        }
    );
    assert_eq!(
        unary(
            TemporalUnaryOperation::Negate,
            TemporalValue::Vec2 {
                value: Vec2 { x: 1.0, y: -2.0 }
            },
            TemporalType::Vec2,
        )
        .unwrap(),
        TemporalValue::Vec2 {
            value: Vec2 { x: -1.0, y: 2.0 }
        }
    );
    assert_eq!(
        unary(
            TemporalUnaryOperation::Absolute,
            TemporalValue::Time {
                value: RationalTime::new(-2, 30).unwrap()
            },
            TemporalType::Time,
        )
        .unwrap(),
        TemporalValue::Time {
            value: RationalTime::new(2, 30).unwrap()
        }
    );
    assert_eq!(
        unary(
            TemporalUnaryOperation::Absolute,
            TemporalValue::Angle { degrees: -45.0 },
            TemporalType::Angle,
        )
        .unwrap(),
        TemporalValue::Angle { degrees: 45.0 }
    );
}
