use super::Iterable;
use crate::program::expression::core::CoreInstruction;
use crate::program::expression::{
    compile_expression, MapKeyType, PrimitiveType, TypeEnvironment, Value, ValueType,
};

fn instruction() -> CoreInstruction {
    compile_expression(
        "0",
        &TypeEnvironment::new(),
        &crate::program::expression::ExpressionContext::empty(),
    )
    .unwrap()
    .core()
    .blocks()[0]
        .instructions()[0]
        .clone()
}

#[test]
fn iterable_closes_list_range_and_map_order_without_panics() {
    let instruction = instruction();
    let integer = ValueType::primitive(PrimitiveType::Integer);
    let sources = [
        (
            Value::list(integer.clone(), vec![Value::Integer(2), Value::Integer(3)]).unwrap(),
            vec![Value::Integer(2), Value::Integer(3)],
            false,
        ),
        (
            Value::range(4, 0, -2).unwrap(),
            vec![Value::Integer(4), Value::Integer(2)],
            false,
        ),
        (
            Value::map(
                MapKeyType::Text,
                integer,
                vec![
                    (Value::Text("b".into()), Value::Integer(2)),
                    (Value::Text("a".into()), Value::Integer(1)),
                ],
            )
            .unwrap(),
            vec![
                Value::tuple(vec![Value::Text("a".into()), Value::Integer(1)]).unwrap(),
                Value::tuple(vec![Value::Text("b".into()), Value::Integer(2)]).unwrap(),
            ],
            true,
        ),
    ];
    for (source, expected, is_map) in sources {
        let mut iterable = Iterable::new(source, instruction.span()).unwrap();
        assert_eq!(iterable.count(), expected.len() as u64);
        assert_eq!(iterable.is_map(), is_map);
        for value in expected {
            assert_eq!(iterable.next(instruction.span()).unwrap(), Some(value));
        }
        assert_eq!(iterable.next(instruction.span()).unwrap(), None);
    }
}

#[test]
fn non_iterable_runtime_value_is_a_closed_contract_failure() {
    let instruction = instruction();
    let error = Iterable::new(Value::Bool(false), instruction.span())
        .err()
        .unwrap();
    assert_eq!(error.code(), "EXPRESSION_RUNTIME_CONTRACT");
    assert_eq!(error.span(), instruction.span());
}
