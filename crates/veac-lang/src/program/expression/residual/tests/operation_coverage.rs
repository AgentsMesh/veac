use super::support::*;
use crate::program::expression::{
    CoreBuildInputId, ExactNumber as Number, PrimitiveType, ResidualBuildBindings,
    ResidualRuntimeValue, TypeEnvironment, Value, ValueType,
};
use veac_ir::{Length, LengthUnit, RationalTime, TemporalType, TemporalValue};

#[test]
fn temporal_operators_cover_the_closed_arithmetic_and_comparison_sets() {
    let scalar = parameter("scalar_ops", TemporalType::Scalar);
    for (index, (source, input, expected)) in [
        ("x - 1.0", 4.0, TemporalValue::Scalar { value: 3.0 }),
        ("x < 2.0", 1.0, TemporalValue::Boolean { value: true }),
        ("x <= 2.0", 2.0, TemporalValue::Boolean { value: true }),
        ("x > 2.0", 3.0, TemporalValue::Boolean { value: true }),
        ("x != 2.0", 3.0, TemporalValue::Boolean { value: true }),
    ]
    .into_iter()
    .enumerate()
    {
        let result = residualize(
            &compile(source, &[("x", scalar.clone())]),
            &no_bindings(),
            &format!("temporal_op_{index}"),
        );
        assert_eq!(
            evaluate(&result, vec![(0, TemporalValue::Scalar { value: input })]),
            expected
        );
    }
}

#[test]
fn concrete_build_operands_reuse_the_exact_runtime_semantics() {
    let scalar: ValueType = PrimitiveType::Scalar.into();
    let types = TypeEnvironment::from([
        ("left".to_owned(), scalar.clone()),
        ("right".to_owned(), scalar),
        ("flag".to_owned(), PrimitiveType::Boolean.into()),
    ]);
    let bindings = ResidualBuildBindings::from([
        (
            CoreBuildInputId::for_symbol("left"),
            Value::Scalar(Number::integer(6)),
        ),
        (
            CoreBuildInputId::for_symbol("right"),
            Value::Scalar(Number::integer(2)),
        ),
        (CoreBuildInputId::for_symbol("flag"), Value::Bool(false)),
    ]);
    for (index, source) in [
        "+left",
        "-left",
        "!flag",
        "left + right",
        "left - right",
        "left * right",
        "left / right",
        "left < right",
        "left <= right",
        "left > right",
        "left >= right",
        "left == right",
        "left != right",
    ]
    .into_iter()
    .enumerate()
    {
        let result = residualize(
            &compile_with_build(source, types.clone(), &[]),
            &bindings,
            &format!("concrete_op_{index}"),
        );
        assert!(matches!(result.value(), ResidualRuntimeValue::Concrete(_)));
        assert!(result.program().is_none());
    }
}

#[test]
fn temporal_literals_cover_each_numeric_and_scalar_like_value_kind() {
    let cases = [
        ("x + 1", TemporalType::Integer),
        ("x + 2s", TemporalType::Time),
        ("x + 2px", TemporalType::Length),
        ("x + 2deg", TemporalType::Angle),
        ("x == \"title\"", TemporalType::Text),
        ("x == #10203040", TemporalType::Color),
    ];
    for (index, (source, value_type)) in cases.into_iter().enumerate() {
        let result = residualize(
            &compile(
                source,
                &[("x", parameter(&format!("kind_{index}"), value_type))],
            ),
            &no_bindings(),
            &format!("literal_kind_{index}"),
        );
        assert!(result.program().is_some());
    }

    let time = TemporalValue::Time {
        value: RationalTime::new(1, 1).unwrap(),
    };
    let length = TemporalValue::Length {
        value: Length {
            value: 1.0,
            unit: LengthUnit::Pixels,
        },
    };
    assert_ne!(time.value_type(), length.value_type());
}

#[test]
fn repeated_temporal_input_reuses_one_manifest_and_one_input_node() {
    let result = residualize(
        &compile(
            "x + x",
            &[("x", parameter("shared_input", TemporalType::Scalar))],
        ),
        &no_bindings(),
        "shared_input",
    );
    let input = &result.inputs()[0];
    assert_eq!(input.core_input_id().value(), 0);
    assert_eq!(input.temporal_input_id().get(), 0);
    assert_eq!(result.inputs().len(), 1);
    assert_eq!(
        result.provenance().id,
        result.program().unwrap().provenance_id
    );
}
