use crate::*;

use super::eval_support::compare;

#[test]
fn ordered_time_length_and_angle_comparisons_are_typed() {
    assert_eq!(
        compare(
            TemporalCompareOperation::Less,
            TemporalValue::Time {
                value: RationalTime::new(1, 30).unwrap()
            },
            TemporalValue::Time {
                value: RationalTime::new(2, 30).unwrap()
            },
        ),
        TemporalValue::Boolean { value: true }
    );
    assert_eq!(
        compare(
            TemporalCompareOperation::Greater,
            TemporalValue::Length {
                value: Length {
                    value: 2.0,
                    unit: LengthUnit::Pixels
                }
            },
            TemporalValue::Length {
                value: Length {
                    value: 1.0,
                    unit: LengthUnit::Pixels
                }
            },
        ),
        TemporalValue::Boolean { value: true }
    );
    assert_eq!(
        compare(
            TemporalCompareOperation::LessOrEqual,
            TemporalValue::Angle { degrees: 45.0 },
            TemporalValue::Angle { degrees: 90.0 },
        ),
        TemporalValue::Boolean { value: true }
    );
}
