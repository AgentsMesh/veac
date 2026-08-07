use super::*;
use crate::program::expression::core::verify::test_support::{raw, raw_with_types};
use crate::program::expression::{
    CoreTerminator, InputId, PrimitiveType, TypeEnvironment, ValueId, MAX_EXPRESSION_NODES,
};

#[test]
fn collection_rejects_unnamed_and_non_dense_inputs() {
    let types = [("seed".to_owned(), PrimitiveType::Integer.into())]
        .into_iter()
        .collect::<TypeEnvironment>();
    let (mut unnamed, _) = raw_with_types("seed", &types);
    unnamed.inputs[0].name.clear();
    assert!(Definitions::collect(&unnamed)
        .err()
        .unwrap()
        .message()
        .contains("dense and named"));

    let (mut non_dense, _) = raw_with_types("seed", &types);
    non_dense.inputs[0].id = InputId::new(2);
    assert!(Definitions::collect(&non_dense).is_err());
}

#[test]
fn collection_enforces_input_and_value_budgets_before_allocation() {
    let types = [("seed".to_owned(), PrimitiveType::Integer.into())]
        .into_iter()
        .collect::<TypeEnvironment>();
    let (mut inputs, _) = raw_with_types("seed", &types);
    let template = inputs.inputs[0].clone();
    inputs.inputs = (0..=MAX_EXPRESSION_NODES)
        .map(|index| {
            let mut input = template.clone();
            input.id = InputId::new(index as u32);
            input
        })
        .collect();
    assert!(Definitions::collect(&inputs)
        .err()
        .unwrap()
        .message()
        .contains("too many declared inputs"));

    let (mut values, _) = raw("1");
    let template = values.blocks[0].instructions[0].clone();
    values.blocks[0].instructions = vec![template; MAX_EXPRESSION_NODES + 1];
    assert_eq!(
        Definitions::collect(&values).err().unwrap().code(),
        "EXPRESSION_NODE_LIMIT"
    );
}

#[test]
fn collection_rejects_invalid_spans_and_sparse_value_ids() {
    let (mut span, _) = raw("1");
    let CoreTerminator::Return { span: value, .. } = &mut span.blocks[0].terminator else {
        unreachable!()
    };
    *value = std::ops::Range { start: 2, end: 1 };
    assert!(Definitions::collect(&span)
        .err()
        .unwrap()
        .message()
        .contains("invalid source span"));

    let (mut sparse, _) = raw("1");
    sparse.blocks[0].instructions[0].id = ValueId::new(9);
    assert!(Definitions::collect(&sparse)
        .err()
        .unwrap()
        .message()
        .contains("value IDs must be dense"));
}
