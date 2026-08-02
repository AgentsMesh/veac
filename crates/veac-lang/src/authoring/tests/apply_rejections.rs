use super::apply::SOURCE;
use crate::authoring::parse;

#[test]
fn legacy_and_open_ended_apply_forms_are_rejected() {
    for (source, code) in [
        (
            SOURCE.replace("scope layer upper;", "scope sequence main;"),
            "AUTHORING_APPLY_SCOPE_KIND",
        ),
        (
            SOURCE.replace("scope layer upper;", "scope { layer upper; }"),
            "AUTHORING_APPLY_SCOPE_KIND",
        ),
        (
            SOURCE.replace(
                "pipeline { stage effect layer-fx",
                "modifiers { effect layer-fx",
            ),
            "AUTHORING_APPLY_FIELD",
        ),
        (
            SOURCE.replace("mix {}", "mix { z-index 4; }"),
            "AUTHORING_APPLY_MIX_FIELD",
        ),
    ] {
        let diagnostics = parse(&source).unwrap_err();
        assert!(
            diagnostics
                .as_slice()
                .iter()
                .any(|value| value.code == code),
            "expected {code}, got {:?}",
            diagnostics.as_slice()
        );
    }
}

#[test]
fn scope_and_pipeline_members_are_closed() {
    for (source, code) in [
        (
            SOURCE.replace("item gamma; group duo;", "layer lower;"),
            "AUTHORING_APPLY_ITEM_TARGET",
        ),
        (
            SOURCE.replace("stage effect layer-fx", "stage transform layer-fx"),
            "AUTHORING_APPLY_STAGE_KIND",
        ),
        (
            SOURCE.replace("stage effect layer-fx", "effect layer-fx"),
            "AUTHORING_APPLY_PIPELINE_MEMBER",
        ),
        (
            SOURCE.replace(
                "pipeline { stage effect layer-fx { type video.color_adjust; } }",
                "pipeline {}",
            ),
            "AUTHORING_REQUIRED_FIELD",
        ),
    ] {
        let diagnostics = parse(&source).unwrap_err();
        assert!(
            diagnostics
                .as_slice()
                .iter()
                .any(|value| value.code == code),
            "expected {code}, got {:?}",
            diagnostics.as_slice()
        );
    }
}

#[test]
fn duplicate_stage_ids_and_incomplete_band_are_rejected() {
    let duplicate = SOURCE.replace("stage color grade", "stage color tone");
    assert!(parse(&duplicate)
        .unwrap_err()
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_DUPLICATE_ID"));
    let incomplete = SOURCE.replace(
        "from layer lower; through layer upper;",
        "from layer lower;",
    );
    assert!(parse(&incomplete)
        .unwrap_err()
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_REQUIRED_FIELD"));
}

#[test]
fn malformed_scopes_and_mix_masks_have_local_diagnostics() {
    for (source, code) in [
        (
            SOURCE.replace("through layer upper;", "until layer upper;"),
            "AUTHORING_APPLY_BAND_FIELD",
        ),
        (
            SOURCE.replace("scope items { item gamma; group duo; }", "scope items {}"),
            "AUTHORING_REQUIRED_FIELD",
        ),
        (
            SOURCE.replace("scope layer upper;", "scope layer absent;"),
            "AUTHORING_REFERENCE_NOT_FOUND",
        ),
        (
            SOURCE.replace(
                "mask window { shape circle; feather 2px; }",
                "mask { shape circle; }",
            ),
            "AUTHORING_FIELD_SHAPE",
        ),
        (
            SOURCE.replace("mask window { shape circle; feather 2px; }", "mask window;"),
            "AUTHORING_FIELD_SHAPE",
        ),
    ] {
        let diagnostics = parse(&source).unwrap_err();
        assert!(
            diagnostics
                .as_slice()
                .iter()
                .any(|value| value.code == code),
            "expected {code}, got {:?}",
            diagnostics.as_slice()
        );
    }
}
