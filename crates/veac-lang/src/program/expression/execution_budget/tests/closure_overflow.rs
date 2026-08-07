use super::super::ExecutionBudget;

#[test]
fn closure_capture_storage_arithmetic_fails_closed_before_reservation() {
    let error = ExecutionBudget::default()
        .reserve_closure(usize::MAX, 4..9)
        .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert_eq!(error.span(), 4..9);
}
