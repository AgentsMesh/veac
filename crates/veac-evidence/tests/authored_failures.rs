use veac_evidence::*;

const ALL_FEATURES: &str = include_str!("fixtures/authored_all_features.veac");

#[test]
fn checked_numeric_decode_reports_the_logical_field_path() {
    let source = ALL_FEATURES.replace("schema_version: 1", "schema_version: -1");
    let EvidenceAuthoringError::Decode(error) = build_evidence_source(&source).unwrap_err() else {
        panic!("negative schema version must fail checked decoding");
    };
    assert_eq!(error.path, "suite.schema_version");
    assert!(error.message.contains("u32"));

    let source = ALL_FEATURES.replace("transparent_below: 1", "transparent_below: 300");
    let EvidenceAuthoringError::Decode(error) = build_evidence_source(&source).unwrap_err() else {
        panic!("oversized alpha threshold must fail checked decoding");
    };
    assert!(error.path.ends_with("expectation.transparent_below"));
    assert!(error.message.contains("u8"));
}

#[test]
fn model_validation_remains_the_authoritative_semantic_gate() {
    let source = ALL_FEATURES.replace("width: 1.0, height: 1.0", "width: 1.1, height: 1.0");
    let EvidenceAuthoringError::Validation(error) = build_evidence_source(&source).unwrap_err()
    else {
        panic!("invalid normalized geometry must fail suite validation");
    };
    assert!(error
        .issues
        .iter()
        .any(|issue| issue.path == "regions[0].space"));
}

#[test]
fn host_contract_rejects_wrong_entries_and_impure_surface_features() {
    let wrong = "fn evidence() -> int { 1 }";
    let EvidenceAuthoringError::Language(error) = build_evidence_source(wrong).unwrap_err() else {
        panic!("wrong result type must be a language diagnostic");
    };
    assert_eq!(error.as_slice()[0].code, "PROGRAM_ENTRY_SIGNATURE");

    let missing = "fn other() -> EvidenceSuite { other() }";
    let EvidenceAuthoringError::Language(error) = build_evidence_source(missing).unwrap_err()
    else {
        panic!("missing entry must be a language diagnostic");
    };
    assert_eq!(error.as_slice()[0].code, "PROGRAM_ENTRY_MISSING");
}

#[test]
fn source_string_rejects_authored_imports_without_a_loader() {
    let source = format!("import \"./helper.veac\" as helper;\n{ALL_FEATURES}");
    let EvidenceAuthoringError::Language(error) = build_evidence_source(&source).unwrap_err()
    else {
        panic!("source-string import failure must be a language diagnostic");
    };
    assert_eq!(error.as_slice()[0].code, "PROGRAM_IMPORT_LOAD");
}

#[test]
fn every_checked_numeric_field_propagates_its_precise_decode_path() {
    let cases = [
        ("at: 3s", "at: 9223372036854775808.0s", ".at"),
        ("width: 1.0", "width: 9007199254740992.0", ".space.width"),
        ("minimum_frames: 1", "minimum_frames: -1", ".minimum_frames"),
        ("opaque_above: 254", "opaque_above: 300", ".opaque_above"),
        ("mean: optional_range(0.0, 1.0)", "mean: optional_range(9007199254740992.0, 1.0)", ".mean.value.minimum.value"),
        ("mean: optional_range(0.0, 1.0)", "mean: optional_range(0.0, 9007199254740992.0)", ".mean.value.maximum.value"),
        ("transparent_fraction: OptionalRangeExpectation.None", "transparent_fraction: OptionalRangeExpectation.Some { value: RangeExpectation { minimum: OptionalScalar.Some { value: 9007199254740992.0, }, maximum: OptionalScalar.None, }, }", ".transparent_fraction.value.minimum.value"),
        ("partial_fraction: OptionalRangeExpectation.None", "partial_fraction: OptionalRangeExpectation.Some { value: RangeExpectation { minimum: OptionalScalar.Some { value: 9007199254740992.0, }, maximum: OptionalScalar.None, }, }", ".partial_fraction.value.minimum.value"),
        ("opaque_fraction: optional_range(0.0, 1.0)", "opaque_fraction: optional_range(9007199254740992.0, 1.0)", ".opaque_fraction.value.minimum.value"),
        ("change_threshold: 1, expectation: diff_expectation()", "change_threshold: 300, expectation: diff_expectation()", ".change_threshold"),
        ("changed_fraction: optional_range(0.0, 1.0)", "changed_fraction: optional_range(9007199254740992.0, 1.0)", ".changed_fraction.value.minimum.value"),
        ("minimum_margin_pixels: OptionalInteger.Some { value: 0, }", "minimum_margin_pixels: OptionalInteger.Some { value: -1, }", ".minimum_margin_pixels.value"),
        ("maximum_width_pixels: OptionalInteger.None", "maximum_width_pixels: OptionalInteger.Some { value: -1, }", ".maximum_width_pixels.value"),
        ("maximum_height_pixels: OptionalInteger.Some { value: 1080, }", "maximum_height_pixels: OptionalInteger.Some { value: -1, }", ".maximum_height_pixels.value"),
        ("overlay_alpha_minimum: 1", "overlay_alpha_minimum: 300", ".overlay_alpha_minimum"),
        ("maximum_expected_rmse: 0.1", "maximum_expected_rmse: 9007199254740992.0", ".maximum_expected_rmse"),
        ("minimum_improvement_ratio: 1.1", "minimum_improvement_ratio: 9007199254740992.0", ".minimum_improvement_ratio"),
        ("visible_prefix: 2", "visible_prefix: -1", ".visible_prefix"),
        ("change_threshold: 1, visible_minimum", "change_threshold: 300, visible_minimum", ".change_threshold"),
        ("visible_minimum: 0.8", "visible_minimum: 9007199254740992.0", ".visible_minimum"),
        ("hidden_maximum: 0.2", "hidden_maximum: 9007199254740992.0", ".hidden_maximum"),
        ("ChangedFraction { threshold: 1", "ChangedFraction { threshold: 300", ".metric.threshold"),
        ("AlphaCentroid { minimum: 1", "AlphaCentroid { minimum: 300", ".metric.minimum"),
        ("minimum_interval_motion: 0.0", "minimum_interval_motion: 9007199254740992.0", ".minimum_interval_motion"),
        ("minimum_deceleration_ratio: 1.0", "minimum_deceleration_ratio: 9007199254740992.0", ".minimum_deceleration_ratio"),
        ("monotonic_tolerance: 0.0", "monotonic_tolerance: 9007199254740992.0", ".monotonic_tolerance"),
    ];
    for (old, new, suffix) in cases {
        let source = ALL_FEATURES.replacen(old, new, 1);
        assert_ne!(source, ALL_FEATURES, "fixture token must exist: {old}");
        let EvidenceAuthoringError::Decode(error) = build_evidence_source(&source).unwrap_err()
        else {
            panic!("{old} must fail checked decoding");
        };
        assert!(error.path.ends_with(suffix), "{} != {suffix}", error.path);
    }
}
