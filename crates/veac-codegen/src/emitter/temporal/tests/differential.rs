use veac_plan::canonical::*;

#[path = "differential/comparison.rs"]
mod comparison;
#[path = "differential/composites.rs"]
mod composites;
#[path = "differential/lazy_select.rs"]
mod lazy_select;
#[path = "differential/scaling.rs"]
mod scaling;

use super::support::{
    compile, evaluate, ffmpeg, node, numeric_expression, numeric_value, plan, program, scalar,
};

const SAMPLES: [f64; 5] = [0.75, 0.1, 0.9, 0.25, 0.5];

#[test]
fn fixed_seed_numeric_dag_matches_reference_evaluator_in_random_access_order() {
    let input_id = TemporalInputId::new(0);
    let inputs = vec![clock_input(input_id)];
    let mut nodes = vec![node(
        0,
        TemporalType::Scalar,
        TemporalNodeKind::Input { input_id },
    )];
    for value in [0.125, 0.5, 1.25] {
        push_literal(&mut nodes, value);
    }
    let mut seed = 0x5eed_cafe_u64;
    for _ in 0..12 {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let operation = [
            TemporalBinaryOperation::Add,
            TemporalBinaryOperation::Subtract,
            TemporalBinaryOperation::Multiply,
            TemporalBinaryOperation::Minimum,
            TemporalBinaryOperation::Maximum,
        ][(seed as usize) % 5];
        let len = nodes.len() as u32;
        let left = TemporalNodeId::new((seed as u32) % len);
        let right = TemporalNodeId::new(((seed >> 32) as u32) % len);
        nodes.push(node(
            len,
            TemporalType::Scalar,
            TemporalNodeKind::Binary {
                operation,
                left,
                right,
            },
        ));
    }
    let result = nodes.len() as u32 - 1;
    let (plan, binding) = plan(
        program(inputs, nodes, result, TemporalType::Scalar),
        vec![(input_id, TemporalClock::Progress)],
        Vec::new(),
    );
    assert_samples_match(&plan, &binding);
}

#[test]
fn every_curve_easing_matches_reference_evaluator_through_real_ffmpeg() {
    let easings = [
        Interpolation::Hold,
        Interpolation::Linear,
        Interpolation::EaseIn,
        Interpolation::EaseOut,
        Interpolation::EaseInOut,
        Interpolation::Spring {
            frequency: 1.5,
            decay: 4.0,
            initial_velocity: 0.2,
        },
        Interpolation::CubicBezier {
            x1: 0.2,
            y1: 0.1,
            x2: 0.8,
            y2: 0.9,
        },
    ];
    for easing in easings {
        let (plan, binding) = curve_plan(easing);
        assert_samples_match(&plan, &binding);
    }
}

fn curve_plan(interpolation: Interpolation) -> (veac_plan::ResolvedRenderPlan, TemporalBindingId) {
    let input_id = TemporalInputId::new(0);
    let nodes = vec![
        node(
            0,
            TemporalType::Scalar,
            TemporalNodeKind::Input { input_id },
        ),
        node(
            1,
            TemporalType::Scalar,
            TemporalNodeKind::CurveSample {
                input: TemporalNodeId::new(0),
                keys: vec![
                    key(0.0, 0.2, interpolation),
                    key(1.0, 0.9, Interpolation::Hold),
                ],
            },
        ),
    ];
    plan(
        program(vec![clock_input(input_id)], nodes, 1, TemporalType::Scalar),
        vec![(input_id, TemporalClock::Progress)],
        Vec::new(),
    )
}

fn assert_samples_match(plan: &veac_plan::ResolvedRenderPlan, binding: &TemporalBindingId) {
    for sample in SAMPLES {
        let expected = numeric_value(&evaluate(plan, binding, sample));
        let compiled = compile(plan, binding, &format!("{sample:.12}")).unwrap();
        let actual = ffmpeg(numeric_expression(&compiled));
        let tolerance = 2e-8_f64.max(expected.abs() * 2e-8);
        assert!(
            (actual - expected).abs() <= tolerance,
            "{sample}: {actual} != {expected}"
        );
    }
}

fn clock_input(id: TemporalInputId) -> TemporalInputDeclaration {
    TemporalInputDeclaration {
        id,
        value_type: TemporalType::Scalar,
        source: TemporalInputSource::Clock {
            clock: TemporalClock::Progress,
        },
    }
}

fn push_literal(nodes: &mut Vec<TemporalNode>, value: f64) {
    let id = nodes.len() as u32;
    nodes.push(node(
        id,
        TemporalType::Scalar,
        TemporalNodeKind::Literal {
            value: scalar(value),
        },
    ));
}

fn key(position: f64, value: f64, interpolation: Interpolation) -> TemporalCurveKey {
    TemporalCurveKey {
        position: TemporalCurvePosition::Scalar { value: position },
        value: scalar(value),
        interpolation,
    }
}
