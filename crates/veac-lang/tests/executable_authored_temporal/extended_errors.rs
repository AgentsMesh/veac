use veac_lang::program::{build_source, prepare_source};

const SOURCE: &str = include_str!("../fixtures/authored_sink_matrix.veac");

#[test]
fn nested_targets_require_existing_typed_owners_and_parameters() {
    let wrong_text_owner = SOURCE.replace("@text, @title)", "@visual, @picture)");
    assert_reason(
        build_source(&wrong_text_owner).unwrap_err(),
        "EXECUTABLE_TEMPORAL_SINK_OWNER",
    );

    let missing_parameter = SOURCE.replacen("@blur, @radius", "@blur, @missing", 1);
    let error = build_source(&missing_parameter).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_TEMPORAL_TARGET");
    assert!(
        error.as_slice()[0]
            .message
            .contains("closed animatable effect parameter"),
        "{}",
        error.as_slice()[0].message
    );
}

#[test]
fn mask_ordinals_and_apply_clock_surfaces_fail_closed() {
    let fractional = SOURCE.replacen("@picture, 0)", "@picture, 0.5)", 1);
    assert_eq!(
        prepare_source(&fractional).unwrap_err().as_slice()[0].code,
        "PROGRAM_TEMPORAL_INDEX"
    );

    let item_clock = SOURCE.replace(
        "clamp(sequence_time / 3s, 0.0, 1.0)",
        "clamp(progress, 0.0, 1.0)",
    );
    let error = prepare_source(&item_clock).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_TEMPORAL_EXPRESSION");
    assert!(error.as_slice()[0].message.contains("progress"));
}

#[test]
fn apply_target_cannot_claim_an_item_source_clock() {
    let source = SOURCE.replace(
        "animate apply-opacity on apply(@sink-matrix, @main, @grade) {",
        "animate apply-opacity on apply(@sink-matrix, @main, @grade)\
         using resource(@sink-matrix, @font) {",
    );
    assert_eq!(
        prepare_source(&source).unwrap_err().as_slice()[0].code,
        "PROGRAM_TEMPORAL_SOURCE_TARGET"
    );
}

fn assert_reason(error: veac_lang::program::Diagnostics, reason: &str) {
    assert!(error.as_slice()[0].message.contains(reason));
}
