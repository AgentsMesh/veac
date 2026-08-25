use super::*;
use crate::program::expression::{
    compile_functions, BuildInputSlot, CoreBuildInputId, FunctionMap, FunctionOrigin,
    FunctionParameter, PrimitiveType, ValueType,
};

fn time() -> ValueType {
    ValueType::primitive(PrimitiveType::Time)
}

fn defaulted(source: &str, span: std::ops::Range<usize>) -> FunctionDefinition {
    FunctionDefinition::new(
        "render",
        vec![FunctionParameter::new("duration", time())
            .with_default("1s", Some(FunctionOrigin::new(source, span)))],
        time(),
        "{ duration }",
    )
}

#[test]
fn namespace_admission_changes_the_query_key() {
    let first = ExpressionContext::empty();
    let mut functions = FunctionMap::new();
    functions.register_namespace("vendor");
    let changed = first.clone().with_functions(functions);
    assert_ne!(
        FunctionQueryKey::new(&first, &[]),
        FunctionQueryKey::new(&changed, &[])
    );
}

#[test]
fn local_default_origin_changes_the_query_key() {
    let context = ExpressionContext::empty();
    assert_ne!(
        FunctionQueryKey::new(&context, &[defaulted("one.veac", 4..7)]),
        FunctionQueryKey::new(&context, &[defaulted("two.veac", 14..17)])
    );
}

#[test]
fn imported_default_origin_changes_the_query_key() {
    let first =
        compile_functions(&ExpressionContext::empty(), &[defaulted("one.veac", 4..7)]).unwrap();
    let changed = compile_functions(
        &ExpressionContext::empty(),
        &[defaulted("two.veac", 14..17)],
    )
    .unwrap();
    assert_ne!(
        FunctionQueryKey::new(&first, &[]),
        FunctionQueryKey::new(&changed, &[])
    );
}

#[test]
fn provisional_symbol_types_change_the_query_key() {
    let first = ExpressionContext::empty().with_provisional_values(
        [("value".to_owned(), PrimitiveType::Integer.into())]
            .into_iter()
            .collect(),
    );
    let changed = ExpressionContext::empty().with_provisional_values(
        [("value".to_owned(), PrimitiveType::Time.into())]
            .into_iter()
            .collect(),
    );
    assert_ne!(
        FunctionQueryKey::new(&first, &[]),
        FunctionQueryKey::new(&changed, &[])
    );
}

#[test]
fn build_input_binding_names_change_the_query_key() {
    let slot = BuildInputSlot::new(CoreBuildInputId::for_symbol("slot"), "slot".into(), time());
    let context = |name: &str| {
        ExpressionContext::empty()
            .with_build_inputs([(name.to_owned(), slot.clone())].into_iter().collect())
    };
    assert_ne!(
        FunctionQueryKey::new(&context("first"), &[]),
        FunctionQueryKey::new(&context("second"), &[])
    );
}
