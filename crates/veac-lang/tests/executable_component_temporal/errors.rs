use veac_lang::program::{build_source, prepare_source};

use super::SOURCE;

const ATTACHMENT: &str = "animate visual-opacity on clip(value) { pulse(progress) }";

#[test]
fn duplicate_component_sink_taints_the_graph_transaction() {
    let replacement = r#"let once = animate visual-opacity on clip(value) { pulse(progress) };
  animate visual-opacity on clip(once) { progress }"#;
    assert_error(
        &SOURCE.replace(ATTACHMENT, replacement),
        "DOMAIN_TEMPORAL_DUPLICATE",
    );
}

#[test]
fn invalid_mask_ordinals_fail_before_graph_publication() {
    for ordinal in ["-1", "4294967296"] {
        let replacement =
            format!("animate mask-feather on clip-mask(value, {ordinal}) {{ progress }}");
        assert_error(
            &SOURCE.replace(ATTACHMENT, &replacement),
            "DOMAIN_TEMPORAL_SELECTOR",
        );
    }
}

#[test]
fn source_time_requires_a_canonical_media_owner_only_when_used() {
    let replacement = "animate visual-opacity on clip(value) { \
                       clamp(source_time / 1s, 0.0, 1.0) }";
    assert_error(
        &SOURCE.replace(ATTACHMENT, replacement),
        "RESIDUAL_ANIMATION_CLOSURE",
    );
}

#[test]
fn animation_closure_rejects_local_mutation_and_graph_emit() {
    let local = "animate visual-opacity on clip(value) { \
                 var level = progress; set level = level * 2.0; level }";
    assert_prepare_error(&SOURCE.replace(ATTACHMENT, local), "EXPRESSION_CORE_VERIFY");

    let emitted = "animate visual-opacity on clip(value) { \
                   let nested = animate visual-opacity on clip(value) { progress }; progress }";
    assert_prepare_error(
        &SOURCE.replace(ATTACHMENT, emitted),
        "EXPRESSION_CORE_VERIFY",
    );
}

#[test]
fn build_only_domain_operations_are_rejected_before_verified_core() {
    let replacement = r#"animate visual-opacity on clip(value) {
      let invalid = transform_geometry(
        vector(progress, progress), flip_none(), vector(0.5, 0.5), crop_none()
      );
      progress
    }"#;
    assert_prepare_error(
        &SOURCE.replace(ATTACHMENT, replacement),
        "unavailable at Temporal stage",
    );
}

#[test]
fn apply_animation_has_no_item_clock_symbols() {
    let declaration = r#"
fn invalid(value: Apply) -> Apply {
  animate apply-opacity on apply(value) { progress }
}
"#;
    assert_prepare_error(
        &SOURCE.replacen("fn pulse", &format!("{declaration}\nfn pulse"), 1),
        "progress",
    );
}

#[test]
fn root_and_component_producers_cannot_claim_one_sink() {
    let root = r#"
animate visual-opacity on clip(
  @component-temporal, @main, @visual, @first
) { progress }
"#;
    assert_error(
        &SOURCE.replacen("fn pulse", &format!("{root}\nfn pulse"), 1),
        "EXECUTABLE_TEMPORAL_SINK_DUPLICATE",
    );
}

fn assert_error(source: &str, expected: &str) {
    let error = build_source(source).unwrap_err();
    assert!(error.to_string().contains(expected), "{error}");
}

fn assert_prepare_error(source: &str, expected: &str) {
    let error = prepare_source(source).unwrap_err();
    assert!(error.to_string().contains(expected), "{error}");
}
