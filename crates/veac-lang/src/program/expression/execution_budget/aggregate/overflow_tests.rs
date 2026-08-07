use super::{checked_mul, collection_bytes};
use crate::program::expression::execution_budget::limits::ResourceLimits;
use crate::program::expression::execution_budget::resource::Resource;
use crate::program::expression::execution_budget::ExecutionBudget;
use crate::program::expression::CollectionOperation;

fn unlimited() -> ExecutionBudget {
    ExecutionBudget::with_resource_limits(ResourceLimits {
        fuel: usize::MAX,
        value_bytes: usize::MAX,
        iterations: usize::MAX,
        collection_elements: usize::MAX,
        collection_bytes: usize::MAX,
        emitted_entities: usize::MAX,
        emitted_bytes: usize::MAX,
        residual_nodes: usize::MAX,
        residual_bytes: usize::MAX,
        evaluator_storage_bytes: usize::MAX,
    })
}

#[test]
fn aggregate_cardinality_arithmetic_fails_closed_without_wrapping() {
    let budget = unlimited();
    let error = budget
        .reserve_aggregate(CollectionOperation::Fold, true, u64::MAX, 4..9)
        .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert_eq!(error.span(), 4..9);
    if usize::BITS == 64 {
        assert!(error.message().contains("collection element"));
    } else {
        assert!(error.message().contains("iteration"));
    }
}

#[test]
fn collection_byte_helpers_report_the_correct_resource_and_span() {
    let budget = unlimited();
    for error in [
        checked_mul(&budget, usize::MAX, 2, Resource::CollectionBytes, &(10..12)).unwrap_err(),
        collection_bytes(&budget, usize::MAX, 2, false, &(13..17)).unwrap_err(),
    ] {
        assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
        assert!(error.message().contains("collection byte"));
    }
    let error = budget.overflow_iterations(&(18..21));
    assert!(error.message().contains("iteration"));
    assert_eq!(error.span(), 18..21);
}
