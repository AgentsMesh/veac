use super::*;

#[test]
fn text_edits_apply_in_source_order_without_touching_other_bytes() {
    let source = "start 4s middle true end";
    let edits = [
        TextEdit {
            range: TextRange { start: 16, end: 20 },
            replacement: "false".to_owned(),
        },
        TextEdit {
            range: TextRange { start: 6, end: 8 },
            replacement: "6s".to_owned(),
        },
    ];
    assert_eq!(
        apply_text_edits(source, &edits).unwrap(),
        "start 6s middle false end"
    );
}

#[test]
fn invalid_or_overlapping_edits_leave_the_input_unchanged() {
    let source = String::from("abcdef");
    let overlap = [
        TextEdit {
            range: TextRange { start: 1, end: 4 },
            replacement: "x".to_owned(),
        },
        TextEdit {
            range: TextRange { start: 3, end: 5 },
            replacement: "y".to_owned(),
        },
    ];
    assert_eq!(
        apply_text_edits(&source, &overlap),
        Err(SourceEditError::OverlappingTextEdits)
    );
    assert_eq!(source, "abcdef");
}

#[test]
fn text_edits_reject_non_utf8_boundaries_and_duplicate_inserts() {
    let source = "A中B";
    let split = TextEdit {
        range: TextRange { start: 2, end: 3 },
        replacement: "x".to_owned(),
    };
    assert!(matches!(
        apply_text_edits(source, &[split]),
        Err(SourceEditError::TextRangeNotUtf8Boundary { .. })
    ));
    let insert = TextEdit {
        range: TextRange { start: 1, end: 1 },
        replacement: "x".to_owned(),
    };
    assert_eq!(
        apply_text_edits(source, &[insert.clone(), insert]),
        Err(SourceEditError::OverlappingTextEdits)
    );
}

#[test]
fn text_edits_reject_out_of_bounds_ranges_and_allow_adjacency() {
    assert_eq!(apply_text_edits("abc", &[]).unwrap(), "abc");
    for range in [
        TextRange { start: 2, end: 1 },
        TextRange { start: 0, end: 4 },
    ] {
        assert!(matches!(
            apply_text_edits(
                "abc",
                &[TextEdit {
                    range,
                    replacement: String::new(),
                }]
            ),
            Err(SourceEditError::InvalidTextRange { .. })
        ));
    }
    let adjacent = [
        TextEdit {
            range: TextRange { start: 0, end: 1 },
            replacement: "A".to_owned(),
        },
        TextEdit {
            range: TextRange { start: 1, end: 2 },
            replacement: "B".to_owned(),
        },
    ];
    assert_eq!(apply_text_edits("abc", &adjacent).unwrap(), "ABc");
}

#[test]
fn resolved_replacements_are_confined_to_one_module() {
    let op = operation("8s");
    let resolved = resolve_source_edit_text(0, &op, TextRange { start: 0, end: 2 }).unwrap();
    assert_eq!(
        apply_resolved_text_replacements(
            "timeline/main.veac",
            "4s",
            std::slice::from_ref(&resolved)
        )
        .unwrap(),
        "8s"
    );
    assert!(matches!(
        apply_resolved_text_replacements("other.veac", "4s", &[resolved]),
        Err(SourceEditError::ResolvedModuleMismatch { .. })
    ));
}

#[test]
fn resolver_validates_an_operation_before_cloning_its_fields() {
    let invalid = operation(&"x".repeat(MAX_SOURCE_EDIT_SINGLE_FRAGMENT_BYTES + 1));
    assert!(matches!(
        resolve_source_edit_text(0, &invalid, TextRange { start: 0, end: 1 }),
        Err(SourceEditError::InvalidExpression(_))
    ));
}

#[test]
fn public_text_edit_api_enforces_count_and_string_budgets_atomically() {
    let too_many = vec![
        TextEdit {
            range: TextRange { start: 0, end: 0 },
            replacement: String::new(),
        };
        MAX_SOURCE_EDIT_OPERATIONS + 1
    ];
    assert!(matches!(
        apply_text_edits("", &too_many),
        Err(SourceEditError::TooManyTextEdits { .. })
    ));

    let source = String::from("a");
    let edits = [TextEdit {
        range: TextRange { start: 0, end: 1 },
        replacement: "x".repeat(MAX_SOURCE_EDIT_FRAGMENT_PAYLOAD_BYTES + 1),
    }];
    assert!(matches!(
        apply_text_edits(&source, &edits),
        Err(SourceEditError::ReplacementPayloadTooLarge { .. })
    ));
    assert_eq!(source, "a");
    assert_eq!(
        edits[0].replacement.len(),
        MAX_SOURCE_EDIT_FRAGMENT_PAYLOAD_BYTES + 1
    );
    assert_eq!(
        apply_text_edits(
            &source,
            &[TextEdit {
                range: TextRange { start: 0, end: 1 },
                replacement: "b".to_owned(),
            }]
        )
        .unwrap(),
        "b"
    );
}

#[test]
fn semantic_text_edit_errors_keep_priority_over_byte_budgets() {
    let oversized = "x".repeat(MAX_SOURCE_EDIT_FRAGMENT_PAYLOAD_BYTES + 1);
    let invalid = TextEdit {
        range: TextRange { start: 2, end: 1 },
        replacement: oversized,
    };
    assert!(matches!(
        apply_text_edits("a", &[invalid]),
        Err(SourceEditError::InvalidTextRange { .. })
    ));

    let source = "x".repeat(MAX_SOURCE_EDIT_OUTPUT_BYTES);
    let insertion = TextEdit {
        range: TextRange { start: 0, end: 0 },
        replacement: "y".to_owned(),
    };
    assert!(matches!(
        apply_text_edits(&source, &[insertion]),
        Err(SourceEditError::EditedSourceTooLarge { .. })
    ));
    assert_eq!(source.len(), MAX_SOURCE_EDIT_OUTPUT_BYTES);
}
