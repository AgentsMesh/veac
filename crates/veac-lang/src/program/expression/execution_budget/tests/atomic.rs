use super::super::resource::Resource;
use super::super::{ExecutionBudget, ResourceDelta};
use super::{budget, delta, unlimited};

#[test]
fn a_late_dimension_failure_does_not_commit_earlier_previews() {
    let mut limits = unlimited();
    limits.fuel = 2;
    limits.residual_bytes = 0;
    let budget = ExecutionBudget::with_resource_limits(limits);

    budget.reserve(delta(Resource::Fuel, 1), 0..1).unwrap();
    let error = budget
        .reserve(
            ResourceDelta {
                fuel: 1,
                residual_bytes: 1,
                ..ResourceDelta::default()
            },
            2..3,
        )
        .unwrap_err();
    assert!(error.message().contains("residual program byte"));
    budget.reserve(delta(Resource::Fuel, 1), 4..5).unwrap();
}

#[test]
fn checked_arithmetic_overflow_fails_without_partial_commit() {
    let mut limits = unlimited();
    limits.fuel = 1;
    let budget = ExecutionBudget::with_resource_limits(limits);
    budget
        .reserve(delta(Resource::ResidualBytes, usize::MAX), 0..1)
        .unwrap();

    let error = budget
        .reserve(
            ResourceDelta {
                fuel: 1,
                residual_bytes: 1,
                ..ResourceDelta::default()
            },
            2..3,
        )
        .unwrap_err();
    assert!(error.message().contains("residual program byte"));
    budget.reserve(delta(Resource::Fuel, 1), 4..5).unwrap();
}

#[test]
fn nested_work_cannot_reset_aggregate_resource_limits() {
    let mut limits = unlimited();
    limits.iterations = 8;
    limits.collection_elements = 8;
    let budget = ExecutionBudget::with_resource_limits(limits);
    let work = ResourceDelta {
        iterations: 1,
        collection_elements: 1,
        ..ResourceDelta::default()
    };

    for outer in 0..3 {
        for inner in 0..3 {
            let ordinal = outer * 3 + inner;
            let result = budget.reserve(work, ordinal..ordinal + 1);
            if ordinal < 8 {
                result.unwrap();
            } else {
                let error = result.unwrap_err();
                assert!(error.message().contains("aggregate iteration"));
                assert_eq!(error.span(), 8..9);
            }
        }
    }
}

#[test]
fn failed_multi_resource_reservations_leave_every_counter_unchanged() {
    let mut limits = unlimited();
    limits.iterations = 1;
    limits.collection_elements = 2;
    let budget = ExecutionBudget::with_resource_limits(limits);
    let work = ResourceDelta {
        iterations: 1,
        collection_elements: 1,
        ..ResourceDelta::default()
    };
    budget.reserve(work, 0..1).unwrap();
    let error = budget.reserve(work, 2..3).unwrap_err();
    assert!(error.message().contains("aggregate iteration"));
    budget
        .reserve(delta(Resource::CollectionElements, 1), 4..5)
        .unwrap();
}

#[test]
fn a_failed_reservation_does_not_poison_zero_cost_operations() {
    let budget = budget(Resource::Iterations, 0);
    budget
        .reserve(delta(Resource::Iterations, 1), 0..1)
        .unwrap_err();
    budget.reserve(ResourceDelta::default(), 2..3).unwrap();
}
