use veac_plan::canonical::*;

use super::super::support::{assert_code, compile, node, plan, program, scalar};

#[test]
fn fan_out_expression_growth_is_bounded() {
    let mut nodes = vec![node(
        0,
        TemporalType::Scalar,
        TemporalNodeKind::Literal { value: scalar(1.0) },
    )];
    for id in 1..24 {
        nodes.push(node(
            id,
            TemporalType::Scalar,
            TemporalNodeKind::Binary {
                operation: TemporalBinaryOperation::Add,
                left: TemporalNodeId::new(id - 1),
                right: TemporalNodeId::new(id - 1),
            },
        ));
    }
    let (plan, binding) = plan(
        program(Vec::new(), nodes, 23, TemporalType::Scalar),
        Vec::new(),
        Vec::new(),
    );
    assert_code(compile(&plan, &binding, "t"), "TEMPORAL_EXPRESSION_BUDGET");
}

#[test]
fn dynamic_text_choice_growth_is_bounded() {
    let nodes = text_choice_tree(257);
    let result = nodes.len() as u32 - 1;
    let (plan, binding) = plan(
        program(Vec::new(), nodes, result, TemporalType::Text),
        Vec::new(),
        Vec::new(),
    );
    assert_code(
        compile(&plan, &binding, "t"),
        "TEMPORAL_BACKEND_UNSUPPORTED",
    );
}

fn text_choice_tree(count: usize) -> Vec<TemporalNode> {
    let mut nodes = vec![node(
        0,
        TemporalType::Boolean,
        TemporalNodeKind::Literal {
            value: TemporalValue::Boolean { value: true },
        },
    )];
    let mut level = Vec::with_capacity(count);
    for index in 0..count {
        let id = nodes.len() as u32;
        nodes.push(node(
            id,
            TemporalType::Text,
            TemporalNodeKind::Literal {
                value: TemporalValue::Text {
                    value: format!("choice_{index}"),
                },
            },
        ));
        level.push(id);
    }
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            if pair.len() == 1 {
                next.push(pair[0]);
                continue;
            }
            let id = nodes.len() as u32;
            nodes.push(node(
                id,
                TemporalType::Text,
                TemporalNodeKind::Select {
                    condition: TemporalNodeId::new(0),
                    when_true: TemporalNodeId::new(pair[0]),
                    when_false: TemporalNodeId::new(pair[1]),
                },
            ));
            next.push(id);
        }
        level = next;
    }
    nodes
}
