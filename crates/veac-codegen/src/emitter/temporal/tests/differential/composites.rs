use veac_plan::canonical::*;

#[path = "composites/cases.rs"]
mod cases;

use super::super::support::{compile, evaluate, ffmpeg, node, plan, program};
use super::{assert_samples_match, SAMPLES};
use crate::emitter::temporal::value::CompiledValue;
use cases::{color_case, point_case, rect_case, vector_case, Case};

#[test]
fn composed_vector_point_rect_and_color_match_reference_components() {
    for case in cases() {
        let (plan, binding) = install(case);
        for sample in SAMPLES {
            let expected = expected_components(&evaluate(&plan, &binding, sample));
            let compiled = compile(&plan, &binding, &format!("{sample:.12}")).unwrap();
            let actual: Vec<_> = compiled_components(&compiled)
                .iter()
                .map(|expression| ffmpeg(expression))
                .collect();
            assert_components(sample, &actual, &expected);
        }
    }
}

#[test]
fn every_composite_projection_matches_reference_samples() {
    for axis in [TemporalVectorAxis::X, TemporalVectorAxis::Y] {
        let mut case = vector_case();
        project(&mut case, TemporalType::Scalar, |value| {
            TemporalNodeKind::ProjectVec2 { value, axis }
        });
        assert_case(case);
    }
    for axis in [TemporalPointAxis::X, TemporalPointAxis::Y] {
        let mut case = point_case();
        project(&mut case, TemporalType::Length, |value| {
            TemporalNodeKind::ProjectPoint { value, axis }
        });
        assert_case(case);
    }
    for field in [
        TemporalRectField::X,
        TemporalRectField::Y,
        TemporalRectField::Width,
        TemporalRectField::Height,
    ] {
        let mut case = rect_case();
        project(&mut case, TemporalType::Scalar, |value| {
            TemporalNodeKind::ProjectRect { value, field }
        });
        assert_case(case);
    }
    for channel in [
        TemporalColorChannel::Red,
        TemporalColorChannel::Green,
        TemporalColorChannel::Blue,
        TemporalColorChannel::Alpha,
    ] {
        let mut case = color_case();
        project(&mut case, TemporalType::Integer, |value| {
            TemporalNodeKind::ProjectColor { value, channel }
        });
        assert_case(case);
    }
}

#[test]
fn composite_equality_and_inequality_match_reference_samples() {
    for operation in [
        TemporalCompareOperation::Equal,
        TemporalCompareOperation::NotEqual,
    ] {
        for mut case in cases() {
            let literal_id = case.nodes.len() as u32;
            case.nodes.push(node(
                literal_id,
                case.result_type,
                TemporalNodeKind::Literal {
                    value: case.equal.clone(),
                },
            ));
            let result = case.nodes.len() as u32;
            case.nodes.push(node(
                result,
                TemporalType::Boolean,
                TemporalNodeKind::Compare {
                    operation,
                    left: TemporalNodeId::new(case.result),
                    right: TemporalNodeId::new(literal_id),
                },
            ));
            case.result = result;
            case.result_type = TemporalType::Boolean;
            assert_case(case);
        }
    }
}

fn cases() -> [Case; 4] {
    [vector_case(), point_case(), rect_case(), color_case()]
}

fn project(
    case: &mut Case,
    result_type: TemporalType,
    kind: impl FnOnce(TemporalNodeId) -> TemporalNodeKind,
) {
    let result = case.nodes.len() as u32;
    case.nodes.push(node(
        result,
        result_type,
        kind(TemporalNodeId::new(case.result)),
    ));
    case.result = result;
    case.result_type = result_type;
}

fn assert_case(case: Case) {
    let (plan, binding) = install(case);
    assert_samples_match(&plan, &binding);
}

fn install(case: Case) -> (veac_plan::ResolvedRenderPlan, TemporalBindingId) {
    let input_id = TemporalInputId::new(0);
    plan(
        program(
            vec![TemporalInputDeclaration {
                id: input_id,
                value_type: case.input_type,
                source: TemporalInputSource::Clock { clock: case.clock },
            }],
            case.nodes,
            case.result,
            case.result_type,
        ),
        vec![(input_id, case.clock)],
        Vec::new(),
    )
}

fn compiled_components(value: &CompiledValue) -> Vec<String> {
    match value {
        CompiledValue::Vec2(x, y) => vec![x.to_string(), y.to_string()],
        CompiledValue::Point(x, y) => vec![x.value.to_string(), y.value.to_string()],
        CompiledValue::Rect(values) | CompiledValue::Color(values) => {
            values.iter().map(ToString::to_string).collect()
        }
        _ => panic!("expected a compiled composite"),
    }
}

fn expected_components(value: &TemporalValue) -> Vec<f64> {
    match value {
        TemporalValue::Vec2 { value } => vec![value.x, value.y],
        TemporalValue::Point { value } => vec![value.x.value, value.y.value],
        TemporalValue::Rect { value } => vec![value.x, value.y, value.width, value.height],
        TemporalValue::Color { value } => [value.red, value.green, value.blue, value.alpha]
            .map(f64::from)
            .to_vec(),
        _ => panic!("expected a reference composite"),
    }
}

fn assert_components(sample: f64, actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        let tolerance = 2e-8_f64.max(expected.abs() * 2e-8);
        assert!(
            (actual - expected).abs() <= tolerance,
            "{sample}: {actual} != {expected}"
        );
    }
}
