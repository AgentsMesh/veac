use super::{budget, used, Resource};
use crate::program::expression::{
    compile_expression, runtime, Environment, ExpressionContext, TypeEnvironment,
};

const ITERATIONS: usize = 2;
const ENTITIES: usize = 4;

fn compiled() -> crate::program::expression::CompiledExpression {
    compile_expression(
        r#"{
            let keys = [identifier("first"), identifier("second")];
            let items = map(
                keys,
                fn(key: identifier) -> Item effect emit {
                    item(key, item_enabled(), during(0s, 1s),
                        source_generated(generator_solid(#204060ff)), source_timing_native())
                }
            );
            let retained_items = items;
            let timeline = sequence(
                identifier("main"), "预算",
                sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
            );
            project(identifier("demo"), project_settings(600))
                .with_sequence(timeline).entry(timeline)
        }"#,
        &TypeEnvironment::new(),
        &ExpressionContext::empty(),
    )
    .unwrap()
}

#[test]
fn effectful_map_iteration_budget_reaches_topology_validation_at_exact_limit() {
    let compiled = compiled();
    let exact = budget(Resource::Iterations, ITERATIONS);
    let error = runtime::execute_project(&compiled, &Environment::new(), &exact).unwrap_err();
    assert_eq!(error.code(), "DOMAIN_PROJECT_DISCONNECTED");
    assert_eq!(used(&exact, Resource::Iterations), ITERATIONS);

    let short = budget(Resource::Iterations, ITERATIONS - 1);
    let error = runtime::execute_project(&compiled, &Environment::new(), &short).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert_eq!(used(&short, Resource::Iterations), 0);
    assert_eq!(used(&short, Resource::EmittedEntities), 0);
}

#[test]
fn effectful_map_entity_budget_is_exact_before_topology_publication() {
    let compiled = compiled();
    let exact = budget(Resource::EmittedEntities, ENTITIES);
    let error = runtime::execute_project(&compiled, &Environment::new(), &exact).unwrap_err();
    assert_eq!(error.code(), "DOMAIN_PROJECT_DISCONNECTED");
    assert_eq!(used(&exact, Resource::EmittedEntities), ENTITIES);

    let short = budget(Resource::EmittedEntities, ENTITIES - 1);
    let error = runtime::execute_project(&compiled, &Environment::new(), &short).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert_eq!(used(&short, Resource::EmittedEntities), ENTITIES - 1);
}

#[test]
fn effectful_map_byte_budget_is_exact_before_topology_publication() {
    let compiled = compiled();
    let measured = budget(Resource::EmittedBytes, usize::MAX);
    let error = runtime::execute_project(&compiled, &Environment::new(), &measured).unwrap_err();
    assert_eq!(error.code(), "DOMAIN_PROJECT_DISCONNECTED");
    let emitted_bytes = used(&measured, Resource::EmittedBytes);
    let exact = budget(Resource::EmittedBytes, emitted_bytes);
    let error = runtime::execute_project(&compiled, &Environment::new(), &exact).unwrap_err();
    assert_eq!(error.code(), "DOMAIN_PROJECT_DISCONNECTED");
    assert_eq!(used(&exact, Resource::EmittedBytes), emitted_bytes);

    let short = budget(Resource::EmittedBytes, emitted_bytes - 1);
    let error = runtime::execute_project(&compiled, &Environment::new(), &short).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert!(used(&short, Resource::EmittedBytes) < emitted_bytes);
}
