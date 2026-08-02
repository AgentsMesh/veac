use super::*;
use crate::unit_tests::emitter_tests::support::{fixture, resolved};

#[test]
fn empty_and_missing_entry_diagnostics_have_stable_display_contracts() {
    let empty = CodegenErrors {
        diagnostics: vec![],
    };
    assert_eq!(empty.to_string(), "codegen failed without a diagnostic");
    assert!(CodegenErrors::new(vec![]).is_none());

    let plan = resolved(&fixture());
    let diagnostic = missing_entry(&plan);
    assert_eq!(diagnostic.kind, CodegenErrorKind::MissingSequence);
    assert_eq!(diagnostic.code, "ENTRY_SEQUENCE_MISSING");
    assert_eq!(
        diagnostic.object_id.as_deref(),
        Some(plan.entry_sequence_id.as_str())
    );
    assert_eq!(
        diagnostic.location,
        format!("render-plan:object:{}", plan.entry_sequence_id)
    );
    assert_eq!(
        diagnostic.suggested_repair.as_deref(),
        Some("regenerate the render plan from validated canonical IR")
    );
}

#[test]
fn root_diagnostics_keep_a_location_and_kind_specific_repair() {
    let diagnostic = diagnostic(
        CodegenErrorKind::InvalidResourceBinding,
        "BINDING_INVALID",
        None,
        "invalid binding",
    );
    assert_eq!(diagnostic.location, "render-plan:root");
    assert_eq!(
        diagnostic.suggested_repair.as_deref(),
        Some("regenerate and verify the machine-local execution bindings")
    );
}
