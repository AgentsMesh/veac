use super::core_instruction_support::{compiled, raw};
use super::support::{evaluate, no_bindings, request, residualize};
use crate::program::expression::core::{closure_digest, verify, FunctionRegistry};
use crate::program::expression::{
    evaluate_compiled, CoreInstructionKind, CoreTemporalComposeOperation as Compose,
    CoreTemporalProjectOperation as Project, CoreValueMetadata, Environment, InputId, Stage,
};
use veac_ir::{Length, LengthUnit, TemporalNodeKind, TemporalValue};

fn cases() -> Vec<(Compose, Project, TemporalValue)> {
    use Project::*;
    vec![
        (Compose::Vec2, Vec2X, TemporalValue::Scalar { value: 1.0 }),
        (Compose::Vec2, Vec2Y, TemporalValue::Scalar { value: 2.0 }),
        (
            Compose::Point,
            PointX,
            TemporalValue::Length {
                value: Length {
                    value: 1.0,
                    unit: LengthUnit::Pixels,
                },
            },
        ),
        (
            Compose::Point,
            PointY,
            TemporalValue::Length {
                value: Length {
                    value: 2.0,
                    unit: LengthUnit::Pixels,
                },
            },
        ),
        (Compose::Rect, RectX, TemporalValue::Scalar { value: 1.0 }),
        (Compose::Rect, RectY, TemporalValue::Scalar { value: 2.0 }),
        (
            Compose::Rect,
            RectWidth,
            TemporalValue::Scalar { value: 3.0 },
        ),
        (
            Compose::Rect,
            RectHeight,
            TemporalValue::Scalar { value: 4.0 },
        ),
        (
            Compose::Color,
            ColorRed,
            TemporalValue::Integer { value: 10 },
        ),
        (
            Compose::Color,
            ColorGreen,
            TemporalValue::Integer { value: 20 },
        ),
        (
            Compose::Color,
            ColorBlue,
            TemporalValue::Integer { value: 30 },
        ),
        (
            Compose::Color,
            ColorAlpha,
            TemporalValue::Integer { value: 40 },
        ),
    ]
}

#[test]
fn every_closed_compose_and_project_variant_residualizes_and_evaluates() {
    for (index, (compose, project, expected)) in cases().into_iter().enumerate() {
        let result = residualize(
            &compiled(compose, project),
            &no_bindings(),
            &format!("closed_{index}"),
        );
        assert_eq!(evaluate(&result, Vec::new()), expected);
        assert!(matches!(
            result.program().unwrap().nodes.last().unwrap().kind,
            TemporalNodeKind::ProjectVec2 { .. }
                | TemporalNodeKind::ProjectPoint { .. }
                | TemporalNodeKind::ProjectRect { .. }
                | TemporalNodeKind::ProjectColor { .. }
        ));
        assert!(result
            .program()
            .unwrap()
            .nodes
            .iter()
            .all(|node| node.provenance_id.is_some()));
    }
}

#[test]
fn closed_temporal_instruction_identity_enters_core_digest() {
    let baseline = closure_digest(
        &[],
        &[],
        &[],
        crate::program::expression::FunctionEffect::Pure,
        true,
        &raw(Compose::Vec2, Project::Vec2X),
    );
    assert_ne!(
        baseline,
        closure_digest(
            &[],
            &[],
            &[],
            crate::program::expression::FunctionEffect::Pure,
            true,
            &raw(Compose::Vec2, Project::Vec2Y),
        )
    );
    assert_ne!(
        baseline,
        closure_digest(
            &[],
            &[],
            &[],
            crate::program::expression::FunctionEffect::Pure,
            true,
            &raw(Compose::Rect, Project::RectX),
        )
    );
}

#[test]
fn verifier_rejects_temporal_instruction_arity_type_result_and_metadata_corruption() {
    let functions = FunctionRegistry::default();

    let mut arity = raw(Compose::Vec2, Project::Vec2X);
    let CoreInstructionKind::TemporalCompose { operands, .. } =
        &mut arity.blocks[0].instructions[2].kind
    else {
        unreachable!()
    };
    operands.pop();
    assert!(verify(arity, &functions, &[])
        .unwrap_err()
        .message()
        .contains("arity"));

    let mut operand = raw(Compose::Color, Project::ColorRed);
    let CoreInstructionKind::TemporalCompose { operation, .. } =
        &mut operand.blocks[0].instructions[4].kind
    else {
        unreachable!()
    };
    *operation = Compose::Rect;
    assert!(verify(operand, &functions, &[])
        .unwrap_err()
        .message()
        .contains("expects scalar"));

    let mut result = raw(Compose::Vec2, Project::Vec2X);
    result.blocks[0].instructions[3].type_id = result.blocks[0].instructions[2].type_id;
    assert!(verify(result, &functions, &[])
        .unwrap_err()
        .message()
        .contains("instruction declares"));

    let mut metadata = raw(Compose::Vec2, Project::Vec2X);
    metadata.blocks[0].instructions[2].metadata =
        CoreValueMetadata::input(Stage::Build, InputId::new(0));
    assert!(verify(metadata, &functions, &[])
        .unwrap_err()
        .message()
        .contains("metadata"));
}

#[test]
fn concrete_runtime_fails_closed_on_temporal_composite_instruction() {
    let expression = compiled(Compose::Color, Project::ColorRed);
    let error = evaluate_compiled(&expression, &Environment::new()).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_TEMPORAL_RESIDUALIZATION_REQUIRED");

    let mut limited = request("closed_budget");
    limited.limits.max_nodes = 0;
    let error =
        super::super::residualize_expression(&expression, &no_bindings(), limited).unwrap_err();
    assert_eq!(error.code(), "RESIDUAL_NODE_LIMIT");
}
