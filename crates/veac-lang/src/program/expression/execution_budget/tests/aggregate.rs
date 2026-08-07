use super::super::resource::Resource;
use super::super::{
    ExecutionBudget, LOGICAL_COLLECTION_BASE_BYTES, LOGICAL_COLLECTION_HANDLE_BYTES,
};
use super::{delta, unlimited};
use crate::program::expression::CollectionOperation;

#[test]
fn list_map_reserves_iterations_and_exact_output_shape() {
    let mut limits = unlimited();
    limits.iterations = 3;
    limits.collection_elements = 3;
    limits.collection_bytes = LOGICAL_COLLECTION_BASE_BYTES + 3 * LOGICAL_COLLECTION_HANDLE_BYTES;
    ExecutionBudget::with_resource_limits(limits)
        .reserve_aggregate(CollectionOperation::Map, false, 3, 1..2)
        .unwrap();
}

#[test]
fn map_input_reserves_every_temporary_entry_tuple_up_front() {
    let mut limits = unlimited();
    limits.iterations = 3;
    limits.collection_elements = 9;
    limits.collection_bytes = LOGICAL_COLLECTION_BASE_BYTES
        + 3 * LOGICAL_COLLECTION_HANDLE_BYTES
        + 3 * (LOGICAL_COLLECTION_BASE_BYTES + 2 * LOGICAL_COLLECTION_HANDLE_BYTES);
    ExecutionBudget::with_resource_limits(limits)
        .reserve_aggregate(CollectionOperation::Filter, true, 3, 2..3)
        .unwrap();
}

#[test]
fn map_fold_only_reserves_temporary_entry_tuples() {
    let mut limits = unlimited();
    limits.iterations = 2;
    limits.collection_elements = 4;
    limits.collection_bytes =
        2 * (LOGICAL_COLLECTION_BASE_BYTES + 2 * LOGICAL_COLLECTION_HANDLE_BYTES);
    ExecutionBudget::with_resource_limits(limits)
        .reserve_aggregate(CollectionOperation::Fold, true, 2, 3..4)
        .unwrap();
}

#[test]
fn empty_map_still_reserves_its_output_container_base() {
    let mut limits = unlimited();
    limits.collection_bytes = LOGICAL_COLLECTION_BASE_BYTES;
    ExecutionBudget::with_resource_limits(limits)
        .reserve_aggregate(CollectionOperation::Map, false, 0, 4..5)
        .unwrap();
}

#[test]
fn aggregate_failure_is_atomic_across_every_reserved_dimension() {
    let mut limits = unlimited();
    limits.iterations = 2;
    limits.collection_elements = 2;
    limits.collection_bytes = LOGICAL_COLLECTION_BASE_BYTES;
    let budget = ExecutionBudget::with_resource_limits(limits);
    let error = budget
        .reserve_aggregate(CollectionOperation::Map, false, 2, 5..6)
        .unwrap_err();
    assert!(error.message().contains("collection byte"));
    budget
        .reserve(delta(Resource::Iterations, 2), 6..7)
        .unwrap();
    budget
        .reserve(delta(Resource::CollectionElements, 2), 7..8)
        .unwrap();
}

#[test]
fn u64_cardinality_failure_does_not_commit_other_resources() {
    let budget = ExecutionBudget::with_resource_limits(unlimited());
    if usize::BITS < 64 {
        budget
            .reserve_aggregate(CollectionOperation::Fold, false, u64::MAX, 8..9)
            .unwrap_err();
        budget
            .reserve(delta(Resource::Iterations, 1), 9..10)
            .unwrap();
    }
}
