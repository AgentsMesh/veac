use super::support::*;
use crate::program::expression::ResidualRuntimeValue;
use veac_ir::{TemporalType, TemporalValue};

fn concrete(source: &str) -> String {
    let result = residualize(&compile(source, &[]), &no_bindings(), "for_each_concrete");
    let ResidualRuntimeValue::Concrete(value) = result.value() else {
        panic!("for-each must finish as a concrete value")
    };
    value.render()
}

#[test]
fn concrete_list_range_and_map_for_each_preserve_source_order() {
    assert_eq!(concrete("for value in [1, 2] { value + 10 }"), "[11, 12]");
    assert_eq!(concrete("for value in 1 .. 4 { value * 2 }"), "[2, 4, 6]");
    assert_eq!(
        concrete("for entry in #{\"b\": 2, \"a\": 1} { entry }"),
        "[(\"a\", 1), (\"b\", 2)]"
    );
}

#[test]
fn static_shape_for_each_can_carry_residual_leaf_values() {
    let expression = compile(
        "fold(for value in [clock, clock + 1.0] { value * 2.0 }, 0.0, \
         fn(total: scalar, ignored: scalar) -> scalar effect pure { total + 1.0 }) + clock",
        &[("clock", parameter("clock", TemporalType::Scalar))],
    );
    let result = residualize(&expression, &no_bindings(), "for_each_residual_leaf");
    assert_eq!(
        evaluate(&result, vec![(0, TemporalValue::Scalar { value: 3.0 })]),
        TemporalValue::Scalar { value: 5.0 }
    );
}

#[test]
fn temporal_leaf_collections_map_filter_and_fold_without_changing_shape() {
    let input = [("clock", parameter("aggregate_clock", TemporalType::Scalar))];
    let mapped = compile(
        "fold(map([1.0, 2.0], fn(value: scalar) -> scalar effect pure { \
         value * 2.0 }), 0.0, fn(total: scalar, value: scalar) \
         -> scalar effect pure { total + value }) + clock",
        &input,
    );
    let filtered = compile(
        "fold(filter([1.0, 2.0, 3.0], fn(value: scalar) -> bool effect pure { \
         value > 1.0 }), 0.0, fn(total: scalar, value: scalar) \
         -> scalar effect pure { total + value }) + clock",
        &input,
    );
    for (name, expression, expected) in [
        ("aggregate_map", mapped, 8.0),
        ("aggregate_filter", filtered, 7.0),
    ] {
        let result = residualize(&expression, &no_bindings(), name);
        assert_eq!(
            evaluate(&result, vec![(0, TemporalValue::Scalar { value: 2.0 })]),
            TemporalValue::Scalar { value: expected }
        );
    }
}

#[test]
fn graph_emit_for_each_is_rejected_before_residual_execution() {
    let expression = compile(
        "for key in [identifier(\"one\")] { \
         item(key, item_enabled(), during(0s, 1s), \
         source_generated(generator_solid(#223344ff)), source_timing_native()) }",
        &[],
    );
    let error = super::super::residualize_expression(
        &expression,
        &no_bindings(),
        request("for_each_graph_emit"),
    )
    .unwrap_err();
    assert_eq!(error.code(), "RESIDUAL_EFFECT_UNSUPPORTED");
    assert!(error.message().contains("Pure for-each"));
}

#[test]
fn oversized_range_fails_before_for_each_output_allocation() {
    let expression = compile(
        "for value in -9223372036854775808 .. 9223372036854775807 { value }",
        &[],
    );
    let error = super::super::residualize_expression(
        &expression,
        &no_bindings(),
        request("for_each_bound"),
    )
    .unwrap_err();
    assert_eq!(error.code(), "RESIDUAL_CORE_CONTRACT");
    assert!(error.message().contains("static bound"));
}

#[test]
fn local_mutation_cannot_enter_a_verified_for_each_body() {
    let error = crate::program::expression::compile_temporal_expression(
        "for value in [1] { var changed = value; set changed = changed + 1; changed }",
        &Default::default(),
        &Default::default(),
        &Default::default(),
    )
    .unwrap_err();
    assert!(matches!(
        error.code(),
        "EXPRESSION_CORE_VERIFY" | "EXPRESSION_COLLECTION_CALLBACK_EFFECT"
    ));
}
