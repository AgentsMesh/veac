use super::support::*;
use crate::program::expression::ResidualRuntimeValue;
use veac_ir::{
    Length, LengthUnit, Point, Rect, TemporalNodeKind, TemporalType, TemporalValue, Vec2,
};

#[test]
fn vector_point_and_rect_use_closed_composite_nodes() {
    let vector = residualize(
        &compile(
            "vector(x, y)",
            &[
                ("x", parameter("x", TemporalType::Scalar)),
                ("y", parameter("y", TemporalType::Scalar)),
            ],
        ),
        &no_bindings(),
        "vector",
    );
    assert!(matches!(
        vector.program().unwrap().nodes.last().unwrap().kind,
        TemporalNodeKind::ComposeVec2 { .. }
    ));
    assert_eq!(
        evaluate(
            &vector,
            vec![
                (0, TemporalValue::Scalar { value: 2.0 }),
                (1, TemporalValue::Scalar { value: 3.0 }),
            ],
        ),
        TemporalValue::Vec2 {
            value: Vec2 { x: 2.0, y: 3.0 }
        }
    );

    let point = residualize(
        &compile(
            "point(x, y)",
            &[
                ("x", parameter("px", TemporalType::Length)),
                ("y", parameter("py", TemporalType::Length)),
            ],
        ),
        &no_bindings(),
        "point",
    );
    let x = Length {
        value: 4.0,
        unit: LengthUnit::Pixels,
    };
    let y = Length {
        value: 5.0,
        unit: LengthUnit::Pixels,
    };
    assert_eq!(
        evaluate(
            &point,
            vec![
                (0, TemporalValue::Length { value: x }),
                (1, TemporalValue::Length { value: y }),
            ],
        ),
        TemporalValue::Point {
            value: Point { x, y }
        }
    );

    let rect = residualize(
        &compile(
            "rect(x, 2.0, 3.0, 4.0)",
            &[("x", parameter("rect_x", TemporalType::Scalar))],
        ),
        &no_bindings(),
        "rect",
    );
    assert_eq!(
        evaluate(&rect, vec![(0, TemporalValue::Scalar { value: 1.0 })]),
        TemporalValue::Rect {
            value: Rect {
                x: 1.0,
                y: 2.0,
                width: 3.0,
                height: 4.0,
            }
        }
    );
}

#[test]
fn constant_domain_composite_still_has_a_temporal_program() {
    let result = residualize(
        &compile("vector(1.0, 2.0)", &[]),
        &no_bindings(),
        "constant_vector",
    );
    assert!(matches!(result.value(), ResidualRuntimeValue::Residual(_)));
    assert_eq!(
        evaluate(&result, Vec::new()),
        TemporalValue::Vec2 {
            value: Vec2 { x: 1.0, y: 2.0 }
        }
    );
}

#[test]
fn unsupported_pure_domain_operation_fails_closed() {
    let error = compile_error(
        "during(clock, 1s)",
        &[("clock", parameter("clock", TemporalType::Time))],
    );
    assert_eq!(error.code(), "EXPRESSION_CORE_VERIFY");
    assert!(error.message().contains("during"));
    assert!(error.message().contains("unavailable at Temporal stage"));
}

#[test]
fn graph_emission_is_never_residualized() {
    let expression = compile("project(identifier(\"demo\"), project_settings(600))", &[]);
    let error =
        super::super::residualize_expression(&expression, &no_bindings(), request("graph_emit"))
            .unwrap_err();
    assert_eq!(error.code(), "RESIDUAL_EFFECT_UNSUPPORTED");
}
