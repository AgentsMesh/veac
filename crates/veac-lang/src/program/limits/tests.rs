use crate::authoring::Span;

use super::{SourceBudget, MAX_SOURCE_BYTES};

#[test]
fn source_budget_rejects_oversized_modules_before_accounting_them() {
    let mut budget = SourceBudget::default();
    let large = "x".repeat(MAX_SOURCE_BYTES + 1);
    assert_eq!(
        budget
            .add("large.veac", &large, Span::default())
            .unwrap_err()
            .code,
        "PROGRAM_SOURCE_LIMIT"
    );
    budget
        .add("small.veac", "module {}", Span::default())
        .unwrap();
}

#[test]
fn source_budget_limits_graph_width_and_total_bytes() {
    let mut modules = SourceBudget::default();
    for index in 0..1024 {
        modules
            .add(&format!("m{index}.veac"), "", Span::default())
            .unwrap();
    }
    assert_eq!(
        modules
            .add("too-wide.veac", "", Span::default())
            .unwrap_err()
            .code,
        "PROGRAM_MODULE_LIMIT"
    );

    let mut bytes = SourceBudget::default();
    let sixteen_mib = "x".repeat(MAX_SOURCE_BYTES);
    for index in 0..4 {
        bytes
            .add(&format!("large{index}.veac"), &sixteen_mib, Span::default())
            .unwrap();
    }
    assert_eq!(
        bytes
            .add("overflow.veac", "x", Span::default())
            .unwrap_err()
            .code,
        "PROGRAM_SOURCE_GRAPH_LIMIT"
    );
}

#[test]
fn source_budget_byte_accumulation_overflow_fails_closed() {
    let mut budget = SourceBudget {
        bytes: usize::MAX,
        modules: 0,
    };
    let error = budget
        .add("overflow.veac", "x", Span::default())
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_SOURCE_GRAPH_LIMIT");
    assert!(error.message.contains("overflowed"));
}
