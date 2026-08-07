use super::*;

#[test]
fn singular_composition_methods_execute_the_static_topology() {
    let item = |key: &str, color: &str| {
        format!(
            "item(identifier(\"{key}\"), item_enabled(), during(0s, 1s), \
            source_generated(generator_solid({color})), source_timing_native())"
        )
    };
    let source = format!(
        r#"{{
            let state = track_state(track_playback_enabled(), track_audio_audible(),
                track_isolation_normal(), track_editing_unlocked());
            let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
                track_routing_default()).with_item({}).with_item({});
            let timeline = sequence(identifier("main"), "静态拓扑",
                sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
                .with_layer(visual);
            project(identifier("demo"), project_settings(600))
                .with_sequence(timeline).entry(timeline)
        }}"#,
        item("first", "#112233ff"),
        item("second", "#445566ff")
    );
    let compiled = compile_expression(
        &source,
        &TypeEnvironment::new(),
        &ExpressionContext::empty(),
    )
    .unwrap();
    let frozen = execute(&compiled);
    assert_eq!(frozen.entity_count(), 5);
    assert_eq!(frozen.root_logical_key(), ["demo"]);
}

#[test]
fn ordinary_value_boundary_rejects_domain_results() {
    let compiled = compile_expression(
        "generator_transparent()",
        &TypeEnvironment::new(),
        &ExpressionContext::empty(),
    )
    .unwrap();
    let error = super::super::super::runtime::execute(
        &compiled,
        &Environment::new(),
        &ExecutionBudget::default(),
    )
    .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_DOMAIN_RESULT_BOUNDARY");
}
