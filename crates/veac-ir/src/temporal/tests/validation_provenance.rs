use crate::*;

use super::support::{codes, library, progress_program};

fn assert_code(value: &TemporalProgramLibrary, expected: &str) {
    let actual = codes(validate_temporal_library(value).unwrap_err());
    assert!(
        actual.iter().any(|code| code == expected),
        "missing {expected}: {actual:?}"
    );
}

#[test]
fn every_provenance_identity_and_authored_site_is_validated() {
    let mut value = library();
    let provenance = &mut value.provenance[0];
    provenance.id = serde_json::from_str("\"bad\"").unwrap();
    provenance.definition.id = serde_json::from_str("\"bad\"").unwrap();
    provenance.definition.source_id = serde_json::from_str("\"bad\"").unwrap();
    provenance.origin.definition_id = serde_json::from_str("\"bad\"").unwrap();
    provenance.origin.source_id = serde_json::from_str("\"bad\"").unwrap();
    provenance.origin.function = "bad\nname".to_owned();
    provenance.call_stack.push(provenance.origin.clone());
    assert_code(&value, "TEMPORAL_PROVENANCE_ID");
    assert_code(&value, "TEMPORAL_DEFINITION_ID");
    assert_code(&value, "TEMPORAL_SITE_ID");
    assert_code(&value, "TEMPORAL_SITE_NAME");
}

#[test]
fn logical_keys_are_valid_and_bounded() {
    let mut value = library();
    value.provenance[0].logical_keys = (0..=MAX_TEMPORAL_LOGICAL_KEYS)
        .map(|index| TemporalLogicalKey::new(format!("key_{index}")).unwrap())
        .collect();
    assert_code(&value, "TEMPORAL_LOGICAL_KEY_LIMIT");

    let mut value = library();
    value.provenance[0].logical_keys = vec![serde_json::from_str("\"bad\"").unwrap()];
    assert_code(&value, "TEMPORAL_LOGICAL_KEY");
}

#[test]
fn validation_error_reports_its_first_source_location() {
    let mut value = library();
    value.opset_version += 1;
    let error = validate_temporal_library(&value).unwrap_err();
    assert!(!error.diagnostics().is_empty());
    assert!(error.to_string().contains("TEMPORAL_OPSET"));
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn standalone_program_validation_checks_node_provenance_identity() {
    let mut value = progress_program();
    value.nodes[0].provenance_id = Some(serde_json::from_str("\"bad\"").unwrap());
    let actual = codes(validate_temporal_program(&value).unwrap_err());
    assert!(actual.iter().any(|code| code == "TEMPORAL_PROVENANCE_ID"));
}
