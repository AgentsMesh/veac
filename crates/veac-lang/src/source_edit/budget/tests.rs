use super::*;

#[test]
fn exact_public_text_edit_boundaries_are_accepted() {
    let plan = text_edit_plan(
        MAX_SOURCE_EDIT_OUTPUT_BYTES,
        MAX_SOURCE_EDIT_OUTPUT_BYTES,
        MAX_SOURCE_EDIT_FRAGMENT_PAYLOAD_BYTES,
    )
    .unwrap();
    assert_eq!(plan.output_bytes, MAX_SOURCE_EDIT_OUTPUT_BYTES);
    assert_eq!(
        MAX_SOURCE_EDIT_OUTPUT_BYTES * 2 + MAX_SOURCE_EDIT_FRAGMENT_PAYLOAD_BYTES,
        MAX_SOURCE_EDIT_WORKING_SET_BYTES
    );
}

#[test]
fn each_text_edit_budget_rejects_the_first_byte_over_its_limit() {
    assert!(matches!(
        text_edit_plan(0, 0, MAX_SOURCE_EDIT_FRAGMENT_PAYLOAD_BYTES + 1),
        Err(SourceEditError::ReplacementPayloadTooLarge { .. })
    ));
    assert!(matches!(
        text_edit_plan(MAX_SOURCE_EDIT_OUTPUT_BYTES, 0, 1),
        Err(SourceEditError::EditedSourceTooLarge { .. })
    ));
    let limits = Limits {
        replacement: usize::MAX,
        output: usize::MAX,
        working: 2,
    };
    assert!(matches!(
        plan_with_limits(1, 0, 1, limits),
        Err(SourceEditError::SourceEditWorkingSetTooLarge { .. })
    ));
}

#[test]
fn size_arithmetic_overflow_fails_closed() {
    let unlimited = Limits {
        replacement: usize::MAX,
        output: usize::MAX,
        working: usize::MAX,
    };
    assert_eq!(
        plan_with_limits(usize::MAX, 0, 1, unlimited).unwrap_err(),
        SourceEditError::SourceEditSizeOverflow
    );
    assert_eq!(
        plan_with_limits(usize::MAX - 1, usize::MAX - 1, 1, unlimited).unwrap_err(),
        SourceEditError::SourceEditSizeOverflow
    );
    assert_eq!(
        validate_fragment_payload([usize::MAX, 1]),
        Err(SourceEditError::SourceEditSizeOverflow)
    );
}
