use veac_plan::canonical::*;

use super::super::support::{node, plan, program};
use super::assert_samples_match;

#[test]
fn both_select_arms_skip_an_unselected_division_by_zero_in_real_ffmpeg() {
    for safe_is_true in [true, false] {
        let (plan, binding) = lazy_plan(safe_is_true);
        assert_samples_match(&plan, &binding);
    }
}

fn lazy_plan(safe_is_true: bool) -> (veac_plan::ResolvedRenderPlan, TemporalBindingId) {
    let input_id = TemporalInputId::new(0);
    let nodes = vec![
        node(
            0,
            TemporalType::Scalar,
            TemporalNodeKind::Input { input_id },
        ),
        literal(
            1,
            TemporalValue::Boolean {
                value: safe_is_true,
            },
        ),
        literal(2, TemporalValue::Scalar { value: 10.0 }),
        literal(3, TemporalValue::Scalar { value: 0.0 }),
        node(
            4,
            TemporalType::Scalar,
            TemporalNodeKind::Binary {
                operation: TemporalBinaryOperation::Divide,
                left: TemporalNodeId::new(2),
                right: TemporalNodeId::new(3),
            },
        ),
        node(
            5,
            TemporalType::Scalar,
            TemporalNodeKind::Select {
                condition: TemporalNodeId::new(1),
                when_true: TemporalNodeId::new(if safe_is_true { 0 } else { 4 }),
                when_false: TemporalNodeId::new(if safe_is_true { 4 } else { 0 }),
            },
        ),
    ];
    plan(
        program(
            vec![TemporalInputDeclaration {
                id: input_id,
                value_type: TemporalType::Scalar,
                source: TemporalInputSource::Clock {
                    clock: TemporalClock::Progress,
                },
            }],
            nodes,
            5,
            TemporalType::Scalar,
        ),
        vec![(input_id, TemporalClock::Progress)],
        Vec::new(),
    )
}

fn literal(id: u32, value: TemporalValue) -> TemporalNode {
    node(id, value.value_type(), TemporalNodeKind::Literal { value })
}
