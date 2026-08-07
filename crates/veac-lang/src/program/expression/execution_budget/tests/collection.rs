use super::super::resource::Resource;
use super::super::{
    ExecutionBudget, LOGICAL_COLLECTION_BASE_BYTES, LOGICAL_COLLECTION_HANDLE_BYTES,
};
use super::{delta, unlimited};

#[test]
fn sequence_and_map_reserve_their_complete_container_shape() {
    let mut sequence_limits = unlimited();
    sequence_limits.collection_elements = 2;
    sequence_limits.collection_bytes =
        LOGICAL_COLLECTION_BASE_BYTES + 2 * LOGICAL_COLLECTION_HANDLE_BYTES;
    ExecutionBudget::with_resource_limits(sequence_limits)
        .reserve_sequence_collection(2, 1..2)
        .unwrap();

    let mut map_limits = unlimited();
    map_limits.collection_elements = 1;
    map_limits.collection_bytes =
        LOGICAL_COLLECTION_BASE_BYTES + 2 * LOGICAL_COLLECTION_HANDLE_BYTES;
    ExecutionBudget::with_resource_limits(map_limits)
        .reserve_map_collection(1, 3..4)
        .unwrap();
}

#[test]
fn a_collection_byte_failure_does_not_commit_elements() {
    let mut limits = unlimited();
    limits.collection_elements = 1;
    limits.collection_bytes = LOGICAL_COLLECTION_HANDLE_BYTES - 1;
    let budget = ExecutionBudget::with_resource_limits(limits);
    let error = budget.reserve_sequence_collection(1, 2..3).unwrap_err();
    assert!(error.message().contains("collection byte"));
    budget
        .reserve(delta(Resource::CollectionElements, 1), 4..5)
        .unwrap();
}

#[test]
fn map_slot_overflow_fails_before_any_counter_commits() {
    let mut limits = unlimited();
    limits.collection_elements = 1;
    let budget = ExecutionBudget::with_resource_limits(limits);
    let error = budget.reserve_map_collection(usize::MAX, 6..7).unwrap_err();
    assert!(error.message().contains("collection byte"));
    budget
        .reserve(delta(Resource::CollectionElements, 1), 8..9)
        .unwrap();
}
