use super::support::*;
use crate::program::expression::{ResidualRuntimeValue, Value};
use veac_ir::{RationalTime, TemporalNodeKind, TemporalType, TemporalValue};

#[test]
fn concrete_expression_stays_concrete() {
    let result = residualize(&compile("1 + 2", &[]), &no_bindings(), "concrete");
    assert_eq!(
        result.value(),
        &ResidualRuntimeValue::Concrete(Value::Integer(3))
    );
    assert!(result.program().is_none());
    assert!(result.inputs().is_empty());
}

#[test]
fn temporal_input_has_a_typed_manifest_and_input_node() {
    let identity = parameter("gain", TemporalType::Scalar);
    let result = residualize(
        &compile("gain", &[("gain", identity.clone())]),
        &no_bindings(),
        "input",
    );
    let program = result.program().unwrap();
    assert_eq!(program.inputs.len(), 1);
    assert_eq!(program.result_type, TemporalType::Scalar);
    assert!(matches!(
        program.nodes[0].kind,
        TemporalNodeKind::Input { .. }
    ));
    assert_eq!(result.inputs()[0].identity(), &identity);
    assert_eq!(
        evaluate(&result, vec![(0, TemporalValue::Scalar { value: 0.75 })]),
        TemporalValue::Scalar { value: 0.75 }
    );
}

#[test]
fn unary_arithmetic_and_comparison_preserve_temporal_semantics() {
    let scalar = parameter("x", TemporalType::Scalar);
    let result = residualize(
        &compile("-x + 2.0", &[("x", scalar.clone())]),
        &no_bindings(),
        "arithmetic",
    );
    assert_eq!(
        evaluate(&result, vec![(0, TemporalValue::Scalar { value: 0.5 })]),
        TemporalValue::Scalar { value: 1.5 }
    );

    let compared = residualize(
        &compile("x >= 2.0", &[("x", scalar)]),
        &no_bindings(),
        "compare",
    );
    assert_eq!(
        evaluate(&compared, vec![(0, TemporalValue::Scalar { value: 2.0 })]),
        TemporalValue::Boolean { value: true }
    );
}

#[test]
fn positive_and_boolean_equality_lower_without_semantic_drift() {
    let scalar = parameter("x", TemporalType::Scalar);
    let positive = residualize(&compile("+x", &[("x", scalar)]), &no_bindings(), "positive");
    assert_eq!(positive.program().unwrap().nodes.len(), 1);

    let flag = parameter("flag", TemporalType::Boolean);
    let equal = residualize(
        &compile("!flag == true", &[("flag", flag)]),
        &no_bindings(),
        "equal",
    );
    assert_eq!(
        evaluate(&equal, vec![(0, TemporalValue::Boolean { value: false })]),
        TemporalValue::Boolean { value: true }
    );
}

#[test]
fn percent_is_normalized_before_temporal_time_scaling() {
    let time = parameter("clock", TemporalType::Time);
    let multiplied = residualize(
        &compile("50% * clock", &[("clock", time.clone())]),
        &no_bindings(),
        "percent_multiply",
    );
    let divided = residualize(
        &compile("clock / 50%", &[("clock", time)]),
        &no_bindings(),
        "percent_divide",
    );
    let input = TemporalValue::Time {
        value: RationalTime::new(8, 1).unwrap(),
    };
    assert_eq!(
        evaluate(&multiplied, vec![(0, input.clone())]),
        TemporalValue::Time {
            value: RationalTime::new(4, 1).unwrap()
        }
    );
    assert_eq!(
        evaluate(&divided, vec![(0, input)]),
        TemporalValue::Time {
            value: RationalTime::new(16, 1).unwrap()
        }
    );
}

#[test]
fn temporal_program_is_deterministic_across_random_access_values() {
    let expression = compile(
        "x * 2.0 + 1.0",
        &[("x", parameter("x", TemporalType::Scalar))],
    );
    let result = residualize(&expression, &no_bindings(), "random_access");
    let digest = result.program().unwrap().content_sha256.clone();
    for (input, expected) in [(0.0, 1.0), (2.5, 6.0), (-4.0, -7.0)] {
        assert_eq!(
            evaluate(&result, vec![(0, TemporalValue::Scalar { value: input })]),
            TemporalValue::Scalar { value: expected }
        );
        assert_eq!(result.program().unwrap().content_sha256, digest);
    }
}
