use super::*;

#[test]
fn public_text_edit_count_accepts_exact_boundary_before_collecting() {
    let source = "x".repeat(MAX_SOURCE_EDIT_OPERATIONS);
    let edits = (0..MAX_SOURCE_EDIT_OPERATIONS)
        .map(|start| TextEdit {
            range: TextRange {
                start,
                end: start + 1,
            },
            replacement: String::new(),
        })
        .collect::<Vec<_>>();
    assert_eq!(apply_text_edits(&source, &edits).unwrap(), "");

    let mut too_many = edits;
    too_many.push(TextEdit {
        range: TextRange {
            start: usize::MAX,
            end: 0,
        },
        replacement: String::new(),
    });
    assert!(matches!(
        apply_text_edits(&source, &too_many),
        Err(SourceEditError::TooManyTextEdits { .. })
    ));
}

#[test]
fn resolved_replacement_count_is_checked_before_borrowed_collection() {
    let source = "x".repeat(MAX_SOURCE_EDIT_OPERATIONS);
    let operation = operation("y");
    let mut replacements = (0..MAX_SOURCE_EDIT_OPERATIONS)
        .map(|start| {
            resolve_set_expression_text(
                start,
                &operation,
                TextRange {
                    start,
                    end: start + 1,
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        apply_resolved_text_replacements("timeline/main.veac", &source, &replacements).unwrap(),
        "y".repeat(MAX_SOURCE_EDIT_OPERATIONS)
    );

    replacements.push(replacements[0].clone());
    assert!(matches!(
        apply_resolved_text_replacements("timeline/main.veac", &source, &replacements),
        Err(SourceEditError::TooManyTextEdits { .. })
    ));
    assert!(matches!(
        apply_resolved_text_replacements("other.veac", &source, &replacements),
        Err(SourceEditError::ResolvedModuleMismatch { .. })
    ));
}
