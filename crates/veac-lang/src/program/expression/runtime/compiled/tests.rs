use super::Evaluator;
use crate::program::expression::{
    compile_expression, Environment, ExecutionBudget, ExpressionContext, TypeEnvironment, Value,
};
use crate::program::DomainOperationRegistry;

#[path = "tests/feature_matrix.rs"]
mod feature_matrix;
#[path = "tests/local_contracts.rs"]
mod local_contracts;
#[path = "tests/runtime_contracts.rs"]
mod runtime_contracts;

#[test]
fn callback_failure_taints_and_prevents_partial_publication() {
    assert_tainted(
        r#"map([false, true], fn(fail: bool) -> Sequence effect emit {
            let checked = 1 / (if fail { 0 } else { 1 });
            sequence(
                if fail { identifier("second") } else { identifier("first") },
                "Runtime test",
                sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
            )
        })"#,
        4,
    );
}

#[test]
fn surface_for_failure_cannot_publish_iteration_provenance() {
    assert_tainted(
        r#"for fail in [false, true] {
            let checked = 1 / (if fail { 0 } else { 1 });
            sequence(
                if fail { identifier("second") } else { identifier("first") },
                "Runtime test",
                sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
            )
        }"#,
        4,
    );
}

#[test]
fn failure_after_effectful_iteration_taints_the_same_transaction() {
    assert_tainted(
        r#"{
            let sequences = map(
                [identifier("first"), identifier("second")],
                fn(key: identifier) -> Sequence effect emit {
                    sequence(
                        key,
                        "Runtime test",
                        sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
                    )
                }
            );
            let failed = 1 / 0;
            sequences
        }"#,
        8,
    );
}

#[test]
fn failure_after_graph_values_cross_local_slots_taints_the_transaction() {
    assert_tainted(
        r#"{
            var emitted = sequence(
                identifier("first"), "Runtime test",
                sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
            );
            set emitted = sequence(
                identifier("second"), "Runtime test",
                sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
            );
            let failed = 1 / 0;
            emitted
        }"#,
        8,
    );
}

fn assert_tainted(source: &str, records: usize) {
    let compiled =
        compile_expression(source, &TypeEnvironment::new(), &ExpressionContext::empty()).unwrap();
    let environment = Environment::new();
    let execution = ExecutionBudget::default();
    let registry = DomainOperationRegistry::standard();
    let mut evaluator =
        Evaluator::new(&environment, &execution, compiled.registry_arc(), &registry);
    let error = evaluator
        .program(compiled.verified(), &[], &[], 0, None, None)
        .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_DIVIDE_BY_ZERO");
    assert!(evaluator.iterations.is_empty());
    assert_eq!(evaluator.domain.record_count(), records);
    let graph = evaluator.domain;
    assert_eq!(
        graph.freeze(&Value::Integer(0), 0..0).unwrap_err().code(),
        "DOMAIN_TRANSACTION_FAILED"
    );
}
