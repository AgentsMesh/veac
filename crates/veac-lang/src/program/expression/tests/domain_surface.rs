use super::super::{
    compile_expression, compile_functions, Effect, Environment, ExecutionBudget, ExpressionContext,
    FunctionDefinition, FunctionParameter, PrimitiveType, TypeEnvironment, ValueType,
};
use crate::program::{DomainOperationId, DomainType};

#[path = "domain_surface/static_topology.rs"]
mod static_topology;

const PROJECT: &str = r#"
    {
        let content = CONTENT;
        let state = track_state(
            track_playback_enabled(), track_audio_audible(),
            track_isolation_normal(), track_editing_unlocked()
        );
        let visual = visual_layer(
            identifier("visual"), 0, placement_free(), state, track_routing_default()
        ).with_item(content);
        let timeline = sequence(
            identifier("main"), "领域运行时",
            sequence_settings(canvas(1920px, 1080px), frame_rate(30, 1), 48000)
        ).with_layer(visual);
        project(identifier("demo"), project_settings(600))
            .with_sequence(timeline).entry(timeline)
    }
"#;

fn project(content: &str, context: &ExpressionContext) -> super::super::CompiledExpression {
    compile_expression(
        &PROJECT.replace("CONTENT", content),
        &TypeEnvironment::new(),
        context,
    )
    .unwrap()
}

fn execute(
    compiled: &super::super::CompiledExpression,
) -> super::super::runtime::domain_graph::FrozenDomainGraph {
    super::super::runtime::execute_project(
        compiled,
        &Environment::new(),
        &ExecutionBudget::default(),
    )
    .unwrap()
}

fn generated_project(generator: &str) -> super::super::CompiledExpression {
    let source = format!(
        r#"{{
            let keys = [identifier("first"), identifier("second")];
            let sources = {generator};
            let retained_sources = sources;
            let timeline = sequence(
                identifier("main"), "领域迭代",
                sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
            );
            project(identifier("demo"), project_settings(600))
                .with_sequence(timeline).entry(timeline)
        }}"#
    );
    compile_expression(
        &source,
        &TypeEnvironment::new(),
        &ExpressionContext::empty(),
    )
    .unwrap()
}

#[test]
fn surface_program_executes_and_freezes_a_connected_project() {
    let item = r#"item(identifier("background"), item_enabled(), during(0s, 3s),
        source_generated(generator_solid(#223344ff)), source_timing_native())"#;
    let frozen = execute(&project(item, &ExpressionContext::empty()));
    assert_eq!(frozen.root().domain_type(), DomainType::Project);
    assert_eq!(frozen.root_logical_key(), ["demo"]);
    assert_eq!(frozen.entity_count(), 4);
    assert_eq!(
        frozen.operation(frozen.root()),
        Some(DomainOperationId::ProjectEntry)
    );
    assert!(frozen.logical_bytes() > 0);
}

#[test]
fn map_and_surface_for_emit_descriptions_in_one_transaction() {
    for generator in [
        "map(keys, fn(key: identifier) -> Source effect emit { \
            source_generated(generator_solid(#223344ff)) })",
        "for key in keys { source_generated(generator_solid(#223344ff)) }",
    ] {
        let frozen = execute(&generated_project(generator));
        assert_eq!(frozen.entity_count(), 2);
        assert!(frozen.record_count() > frozen.entity_count());
        assert_eq!(frozen.root_logical_key(), ["demo"]);
    }
}

#[test]
fn duplicate_static_keys_fail_at_attachment() {
    let source = r#"{
        let first = sequence(identifier("same"), "一",
            sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000));
        let second = sequence(identifier("same"), "二",
            sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000));
        project(identifier("demo"), project_settings(600))
            .with_sequence(first).with_sequence(second).entry(first)
    }"#;
    let compiled =
        compile_expression(source, &TypeEnvironment::new(), &ExpressionContext::empty()).unwrap();
    let error = super::super::runtime::execute_project(
        &compiled,
        &Environment::new(),
        &ExecutionBudget::default(),
    )
    .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_DUPLICATE_KEY");
}

#[test]
fn repeated_entity_fails_single_ownership() {
    let source = r#"{
        let shared = sequence(identifier("shared"), "共享",
            sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000));
        project(identifier("demo"), project_settings(600))
            .with_sequence(shared).with_sequence(shared).entry(shared)
    }"#;
    let compiled =
        compile_expression(source, &TypeEnvironment::new(), &ExpressionContext::empty()).unwrap();
    let error = super::super::runtime::execute_project(
        &compiled,
        &Environment::new(),
        &ExecutionBudget::default(),
    )
    .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_SINGLE_OWNERSHIP");
}

#[test]
fn graph_emitting_helpers_share_the_callers_transaction() {
    let helper = FunctionDefinition::new(
        "background",
        vec![FunctionParameter::new(
            "key",
            ValueType::primitive(PrimitiveType::Identifier),
        )],
        ValueType::domain(DomainType::Item),
        "{ item(key, item_enabled(), during(0s, 3s), \
         source_generated(generator_solid(#223344ff)), source_timing_native()) }",
    );
    let context = compile_functions(&ExpressionContext::empty(), &[helper]).unwrap();
    assert_eq!(
        context
            .functions()
            .lookup("background")
            .unwrap()
            .summary()
            .effect(),
        Effect::GraphEmit
    );
    let compiled = project(r#"background(identifier("background"))"#, &context);
    let frozen = execute(&compiled);
    assert_eq!(frozen.entity_count(), 4);
    assert_eq!(frozen.root_logical_key(), ["demo"]);
}
