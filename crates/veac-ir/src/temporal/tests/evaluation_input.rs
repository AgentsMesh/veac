use crate::*;

use super::{
    eval_support::evaluate,
    support::{progress_program, scalar},
};

#[test]
fn input_admission_is_exact_and_typed() {
    let program = progress_program();
    let input = |id, value| TemporalEvaluationInput {
        input_id: TemporalInputId::new(id),
        value,
    };
    assert_eq!(evaluate(&program, &[input(0, scalar(0.25))]), scalar(0.5));
    for inputs in [
        Vec::new(),
        vec![input(1, scalar(0.25))],
        vec![input(0, scalar(0.25)), input(0, scalar(0.5))],
        vec![input(0, TemporalValue::Boolean { value: true })],
        vec![input(0, scalar(-0.0))],
    ] {
        let error =
            evaluate_temporal_program(&program, &inputs, TemporalEvaluationLimits::default())
                .unwrap_err();
        assert_eq!(error.code(), "TEMPORAL_EVALUATION_INPUT");
    }
}

#[test]
fn evaluation_budgets_have_exact_boundaries() {
    let program = progress_program();
    let inputs = [TemporalEvaluationInput {
        input_id: TemporalInputId::new(0),
        value: scalar(0.25),
    }];
    let exact = TemporalEvaluationLimits {
        max_steps: 3,
        max_value_bytes: 32,
    };
    assert_eq!(
        evaluate_temporal_program(&program, &inputs, exact).unwrap(),
        scalar(0.5)
    );
    for limits in [
        TemporalEvaluationLimits {
            max_steps: 2,
            max_value_bytes: 32,
        },
        TemporalEvaluationLimits {
            max_steps: 3,
            max_value_bytes: 31,
        },
    ] {
        let error = evaluate_temporal_program(&program, &inputs, limits).unwrap_err();
        assert_eq!(error.code(), "TEMPORAL_EVALUATION_LIMIT");
    }
}

#[test]
fn evaluator_refuses_unverified_programs() {
    let mut program = progress_program();
    program.content_sha256 = "0".repeat(64);
    let error =
        evaluate_temporal_program(&program, &[], TemporalEvaluationLimits::default()).unwrap_err();
    assert_eq!(error.code(), "TEMPORAL_EVALUATION_PROGRAM");
    assert!(error.message().contains("TEMPORAL_PROGRAM_DIGEST"));
}

#[test]
fn input_admission_accepts_every_canonical_value_type() {
    let values = vec![
        TemporalValue::Boolean { value: true },
        TemporalValue::Integer { value: 7 },
        TemporalValue::Time {
            value: RationalTime::new(3, 30).unwrap(),
        },
        TemporalValue::Length {
            value: Length {
                value: 3.0,
                unit: LengthUnit::Pixels,
            },
        },
        TemporalValue::Angle { degrees: 45.0 },
        TemporalValue::Vec2 {
            value: Vec2 { x: 1.0, y: 2.0 },
        },
        TemporalValue::Color {
            value: Color {
                red: 1,
                green: 2,
                blue: 3,
                alpha: 4,
            },
        },
        TemporalValue::Point {
            value: Point {
                x: Length {
                    value: 1.0,
                    unit: LengthUnit::Pixels,
                },
                y: Length {
                    value: 2.0,
                    unit: LengthUnit::Percent,
                },
            },
        },
        TemporalValue::Rect {
            value: Rect {
                x: 0.0,
                y: 0.1,
                width: 0.8,
                height: 0.9,
            },
        },
        TemporalValue::Text {
            value: "title".to_owned(),
        },
    ];
    for value in values {
        let value_type = value.value_type();
        let inputs = vec![TemporalInputDeclaration {
            id: TemporalInputId::new(0),
            value_type,
            source: TemporalInputSource::Parameter {
                parameter_id: TemporalParameterId::new("tpm_value").unwrap(),
            },
        }];
        let nodes = vec![TemporalNode {
            id: TemporalNodeId::new(0),
            value_type,
            kind: TemporalNodeKind::Input {
                input_id: TemporalInputId::new(0),
            },
            provenance_id: None,
        }];
        let program = super::support::program(inputs, nodes, 0, value_type);
        let actual = evaluate_temporal_program(
            &program,
            &[TemporalEvaluationInput {
                input_id: TemporalInputId::new(0),
                value: value.clone(),
            }],
            TemporalEvaluationLimits::default(),
        )
        .unwrap();
        assert_eq!(actual, value);
    }
}
