use super::*;

#[test]
fn closed_opcode_dispatch_rejects_unknown_and_taints_transaction() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let error = graph.evaluate(0xffff, Vec::new(), 3..7).unwrap_err();
    assert_eq!(error.code(), "DOMAIN_OPERATION_UNKNOWN");
    assert_eq!(error.span(), 3..7);
    assert_eq!(graph.record_count(), 0);
    assert_eq!(
        graph
            .evaluate(DomainOperationId::Canvas.opcode(), Vec::new(), 8..9)
            .unwrap_err()
            .code(),
        "DOMAIN_TRANSACTION_FAILED"
    );
}

#[test]
fn contract_arity_and_type_are_checked_before_allocation() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let error = graph
        .evaluate(DomainOperationId::Canvas.opcode(), vec![length(1)], 1..2)
        .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_OPERAND_ARITY");
    assert_eq!(graph.record_count(), 0);

    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let error = graph
        .evaluate(
            DomainOperationId::Canvas.opcode(),
            vec![Value::Integer(1), length(2)],
            1..2,
        )
        .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_OPERAND_TYPE");
    assert_eq!(graph.record_count(), 0);

    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let error = graph
        .evaluate(
            DomainOperationId::ProjectWithSequence.opcode(),
            vec![Value::Integer(1), Value::Integer(2)],
            1..2,
        )
        .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_OPERAND_TYPE");
}

#[test]
fn domain_lists_validate_each_runtime_handle_type() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let wrong = layer(&mut graph, "layer");
    let error = graph
        .evaluate(
            DomainOperationId::RelationGroup.opcode(),
            vec![identifier("group"), list(DomainType::Layer, vec![wrong])],
            1..2,
        )
        .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_OPERAND_TYPE");
}

#[test]
fn handles_cannot_cross_graph_transactions() {
    let (registry, first_budget) = standard();
    let second_budget = crate::program::expression::ExecutionBudget::default();
    let mut first = DomainGraphTransaction::new(&registry, &first_budget);
    let foreign = sequence(&mut first, "foreign");
    let mut second = DomainGraphTransaction::new(&registry, &second_budget);
    let root = project(&mut second, "project");
    let error = second
        .evaluate(
            DomainOperationId::ProjectWithSequence.opcode(),
            vec![root, foreign],
            1..2,
        )
        .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_CROSS_GRAPH");
}

#[test]
fn host_context_is_a_real_protected_graph_local_allocation() {
    let (registry, first_budget) = standard();
    let second_budget = crate::program::expression::ExecutionBudget::default();
    let mut first = DomainGraphTransaction::new(&registry, &first_budget);
    let context = first.host_context(2..4).unwrap();
    assert_eq!(first.record_count(), 1);
    assert!(first.state.records[0].operation.is_none());
    super::super::validation::node(&first, &context, DomainType::Context, &(2..4)).unwrap();

    let second = DomainGraphTransaction::new(&registry, &second_budget);
    let error = super::super::validation::node(&second, &context, DomainType::Context, &(5..7))
        .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_CROSS_GRAPH");
}

#[test]
fn invalid_and_forged_slots_are_rejected_without_panics() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let invalid = Value::Domain(Arc::new(
        crate::program::expression::DomainValue::from_arena(
            graph.scope.clone(),
            99,
            DomainType::Sequence,
        ),
    ));
    let root = project(&mut graph, "project");
    assert_eq!(
        graph
            .evaluate(
                DomainOperationId::ProjectWithSequence.opcode(),
                vec![root, invalid],
                1..2,
            )
            .unwrap_err()
            .code(),
        "DOMAIN_HANDLE_INVALID"
    );

    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let allocated = canvas(&mut graph);
    let forged = Value::Domain(Arc::new(
        crate::program::expression::DomainValue::from_arena(
            graph.scope.clone(),
            domain(&allocated).arena_slot(),
            DomainType::Sequence,
        ),
    ));
    let root = project(&mut graph, "project");
    assert_eq!(
        graph
            .evaluate(
                DomainOperationId::ProjectWithSequence.opcode(),
                vec![root, forged],
                1..2,
            )
            .unwrap_err()
            .code(),
        "DOMAIN_HANDLE_FORGED"
    );
}

#[test]
fn empty_entity_keys_and_accounting_overflow_fail_before_mutation() {
    use super::super::accounting::{
        reference_bytes, totals, LOGICAL_DOMAIN_OPERAND_BYTES, LOGICAL_GRAPH_REFERENCE_BYTES,
    };

    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let settings = sequence_settings(&mut graph);
    let before = graph.record_count();
    let error = graph
        .evaluate(
            DomainOperationId::Sequence.opcode(),
            vec![identifier(""), text("empty"), settings],
            1..2,
        )
        .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_KEY_REQUIRED");
    assert_eq!(graph.record_count(), before);
    assert_eq!(
        totals(10, 2, [3, 4]),
        Some(10 + 2 * LOGICAL_DOMAIN_OPERAND_BYTES + 7)
    );
    assert_eq!(totals(0, usize::MAX, []), None);
    assert_eq!(totals(usize::MAX, 0, [1]), None);
    assert_eq!(reference_bytes(3), Some(3 * LOGICAL_GRAPH_REFERENCE_BYTES));
    assert_eq!(reference_bytes(usize::MAX), None);
    assert_eq!(primitive(PrimitiveType::Integer, 7), Value::Integer(7));
}
