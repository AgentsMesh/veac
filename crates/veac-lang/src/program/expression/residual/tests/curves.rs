use super::support::*;
use crate::program::expression::{self, ResidualRuntimeValue};
use veac_ir::{
    Color, Length, LengthUnit, Point, Rect, TemporalNodeKind, TemporalType, TemporalValue, Vec2,
};

fn sampled(source: &str, input: TemporalType, name: &str) -> super::super::ResidualizedExpression {
    residualize(
        &compile(source, &[("clock", parameter("clock", input))]),
        &no_bindings(),
        name,
    )
}

#[test]
fn every_closed_interpolation_authors_a_curve_sample() {
    let cases = [
        "sample_curve_hold(clock, [(0.0, 0.0), (1.0, 1.0)])",
        "sample_curve_linear(clock, [(0.0, 0.0), (1.0, 1.0)])",
        "sample_curve_ease_in(clock, [(0.0, 0.0), (1.0, 1.0)])",
        "sample_curve_ease_out(clock, [(0.0, 0.0), (1.0, 1.0)])",
        "sample_curve_ease_in_out(clock, [(0.0, 0.0), (1.0, 1.0)])",
        "sample_curve_spring(clock, [(0.0, 0.0), (1.0, 1.0)], 2.0, 8.0, 0.0)",
        "sample_curve_cubic_bezier(clock, [(0.0, 0.0), (1.0, 1.0)], 0.25, 0.1, 0.25, 1.0)",
    ];
    for (index, source) in cases.into_iter().enumerate() {
        let result = sampled(source, TemporalType::Scalar, &format!("curve_{index}"));
        assert!(matches!(
            result.program().unwrap().nodes.last().unwrap().kind,
            TemporalNodeKind::CurveSample { .. }
        ));
        assert!(matches!(
            evaluate(&result, vec![(0, TemporalValue::Scalar { value: 0.5 })]),
            TemporalValue::Scalar { value } if value.is_finite()
        ));
    }
}

#[test]
fn time_positioned_curve_is_canonical_and_evaluable() {
    let result = sampled(
        "sample_curve_linear(clock, [(0s, 0px), (2s, 20px)])",
        TemporalType::Time,
        "time_curve",
    );
    assert_eq!(
        evaluate(
            &result,
            vec![(
                0,
                TemporalValue::Time {
                    value: veac_ir::RationalTime::new(1, 1).unwrap()
                }
            )],
        ),
        TemporalValue::Length {
            value: Length {
                value: 10.0,
                unit: LengthUnit::Pixels
            }
        }
    );
}

#[test]
fn curve_values_cover_every_authored_interpolatable_type() {
    let cases = [
        ("sample_curve_linear(clock, [(0.0, 0%), (1.0, 100%)])", TemporalValue::Scalar { value: 0.5 }),
        ("sample_curve_linear(clock, [(0.0, 0deg), (1.0, 90deg)])", TemporalValue::Angle { degrees: 45.0 }),
        ("sample_curve_linear(clock, [(0.0, vector(0.0, 2.0)), (1.0, vector(2.0, 4.0))])", TemporalValue::Vec2 { value: Vec2 { x: 1.0, y: 3.0 } }),
        ("sample_curve_linear(clock, [(0.0, point(0px, 2px)), (1.0, point(2px, 4px))])", TemporalValue::Point { value: Point { x: Length { value: 1.0, unit: LengthUnit::Pixels }, y: Length { value: 3.0, unit: LengthUnit::Pixels } } }),
        ("sample_curve_linear(clock, [(0.0, rect(0.0, 2.0, 4.0, 6.0)), (1.0, rect(2.0, 4.0, 6.0, 8.0))])", TemporalValue::Rect { value: Rect { x: 1.0, y: 3.0, width: 5.0, height: 7.0 } }),
        ("sample_curve_linear(clock, [(0.0, #000000ff), (1.0, #ffffffff)])", TemporalValue::Color { value: Color { red: 128, green: 128, blue: 128, alpha: 255 } }),
    ];
    for (index, (source, expected)) in cases.into_iter().enumerate() {
        let result = sampled(
            source,
            TemporalType::Scalar,
            &format!("typed_curve_{index}"),
        );
        assert_eq!(
            evaluate(&result, vec![(0, TemporalValue::Scalar { value: 0.5 })]),
            expected
        );
    }
}

#[test]
fn curve_keys_fail_closed_when_dynamic_empty_or_unordered() {
    let cases = [
        (
            "sample_curve_linear(clock, [(0.0, clock), (1.0, 1.0)])",
            "RESIDUAL_CURVE_DYNAMIC_KEY",
        ),
        (
            "{ let keys: list<(scalar, scalar)> = []; sample_curve_linear(clock, keys) }",
            "RESIDUAL_CURVE_KEYS",
        ),
        (
            "sample_curve_linear(clock, [(1.0, 0.0), (0.0, 1.0)])",
            "RESIDUAL_CURVE_KEY_CONTRACT",
        ),
    ];
    for (index, (source, code)) in cases.into_iter().enumerate() {
        let expression = compile(
            source,
            &[("clock", parameter("clock", TemporalType::Scalar))],
        );
        let error = super::super::residualize_expression(
            &expression,
            &no_bindings(),
            request(&format!("bad_curve_{index}")),
        )
        .unwrap_err();
        assert_eq!(error.code(), code);
    }
}

#[test]
fn curve_builtin_requires_temporal_residualization_context() {
    let error = expression::evaluate(
        "sample_curve_linear(0.5, [(0.0, 0.0), (1.0, 1.0)])",
        &Default::default(),
    )
    .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_TEMPORAL_CURVE_CONTEXT");
    assert!(!matches!(
        sampled(
            "sample_curve_linear(clock, [(0.0, 0.0), (1.0, 1.0)])",
            TemporalType::Scalar,
            "dynamic_curve"
        )
        .value(),
        ResidualRuntimeValue::Concrete(_)
    ));
}
