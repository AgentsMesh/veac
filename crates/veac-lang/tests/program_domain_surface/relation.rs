use super::*;

fn item(key: &str, start: &str) -> String {
    format!(
        "item(identifier(\"{key}\"), item_enabled(), during({start}, 2s), \
         source_generated(generator_transparent()), source_timing_native())"
    )
}

#[test]
fn transition_surface_lowers_to_v6_pure_and_graph_operations() {
    let source = format!(
        r#"{{
          let outgoing = {};
          let incoming = {};
          let cross = relation_transition(
            identifier("cross"), outgoing, incoming, transition_dissolve(1s));
          sequence(identifier("main"), "主时间线",
            sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
            .with_relation(cross)
        }}"#,
        item("out", "0s"),
        item("in", "1s")
    );
    let compiled = compile(&source);
    assert_eq!(
        compiled.result_type().as_domain(),
        Some(DomainType::Sequence)
    );
    let domain = instructions(&compiled)
        .filter(|instruction| {
            matches!(
                instruction.kind(),
                CoreInstructionKind::DomainConstruct { .. } | CoreInstructionKind::GraphEmit { .. }
            )
        })
        .collect::<Vec<_>>();
    let transition = domain
        .iter()
        .find(|instruction| {
            matches!(instruction.kind(), CoreInstructionKind::DomainConstruct { opcode, .. }
            if *opcode == DomainOperationId::TransitionDissolve.opcode())
        })
        .unwrap();
    assert_eq!(transition.metadata().effect(), Effect::Pure);
    let relation = domain
        .iter()
        .find(|instruction| {
            matches!(instruction.kind(), CoreInstructionKind::GraphEmit { opcode, operands }
            if *opcode == DomainOperationId::RelationTransition.opcode() && operands.len() == 4)
        })
        .unwrap();
    assert_eq!(relation.metadata().effect(), Effect::GraphEmit);
    assert!(matches!(
        domain.last().unwrap().kind(),
        CoreInstructionKind::GraphEmit { opcode, operands }
            if *opcode == DomainOperationId::SequenceWithRelation.opcode() && operands.len() == 2
    ));
}

#[test]
fn repeated_relation_method_is_typed_sequence_composition() {
    let source = format!(
        r#"{{
          let a = {};
          let b = {};
          let first = relation_transition(
            identifier("first"), a, b, transition_dissolve(500ms));
          let second = relation_group(identifier("second"), [a, b]);
          sequence(identifier("main"), "主时间线",
            sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
            .with_relation(first).with_relation(second)
        }}"#,
        item("a", "0s"),
        item("b", "1500ms")
    );
    let compiled = compile(&source);
    let updates = instructions(&compiled)
        .filter(|instruction| {
            matches!(
                instruction.kind(), CoreInstructionKind::GraphEmit { opcode, .. }
                    if *opcode == DomainOperationId::SequenceWithRelation.opcode()
            )
        })
        .count();
    assert_eq!(updates, 2);
}

#[test]
fn relation_surface_rejects_arity_units_endpoints_and_receivers() {
    let item = item("a", "0s");
    for (source, code) in [
        ("transition_dissolve()".to_owned(), "EXPRESSION_CALL_ARITY"),
        (
            "transition_dissolve(1px)".to_owned(),
            "EXPRESSION_CALL_ARGUMENT_TYPE",
        ),
        (
            format!(r#"relation_transition(identifier("x"), {item}, {item}, transform_2d)"#),
            "EXPRESSION_UNKNOWN_SYMBOL",
        ),
        (
            format!(r#"visual_layer.with_relation(relation_group(identifier("x"), [{item}]))"#),
            "EXPRESSION_UNKNOWN_SYMBOL",
        ),
    ] {
        let error = compile_expression(
            &source,
            &TypeEnvironment::new(),
            &ExpressionContext::empty(),
        )
        .unwrap_err();
        assert_eq!(error.code(), code, "{source}: {}", error.message());
    }
}
