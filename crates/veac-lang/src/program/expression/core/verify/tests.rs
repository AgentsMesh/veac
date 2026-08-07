use super::*;
use crate::program::expression::{MAX_EXPRESSION_DEPTH, MAX_EXPRESSION_NODES};

#[test]
fn recursive_verify_budget_fails_closed_for_depth_count_and_arithmetic() {
    let (scalar, _) = test_support::raw("1");
    let (closure, _) = test_support::raw("fn(value: int) -> int effect pure { value }");

    let mut budget = VerifyBudget::default();
    assert!(budget
        .enter(&scalar, MAX_EXPRESSION_DEPTH + 1)
        .unwrap_err()
        .message()
        .contains("depth"));

    let mut budget = VerifyBudget {
        definitions: usize::MAX,
        values: 0,
    };
    assert!(budget
        .enter(&closure, 0)
        .unwrap_err()
        .message()
        .contains("definition count overflows"));

    let mut budget = VerifyBudget {
        definitions: 0,
        values: usize::MAX,
    };
    assert!(budget
        .enter(&scalar, 0)
        .unwrap_err()
        .message()
        .contains("value count overflows"));

    let mut budget = VerifyBudget {
        definitions: MAX_EXPRESSION_NODES,
        values: 0,
    };
    assert!(budget
        .enter(&closure, 0)
        .unwrap_err()
        .message()
        .contains("aggregate limit"));
}
