use super::value_type_bytes;
use std::mem::size_of;

use crate::program::expression::{
    compile_expression, compile_functions, CoreType, CoreTypeEntry, CoreTypeId, DependencyMask,
    ExpressionContext, FunctionDefinition, FunctionParameter, InputId, TypeEnvironment, ValueType,
};

#[test]
fn retained_core_counts_shape_and_leaf_input_masks() {
    let types = [("input".to_owned(), ValueType::parse("scalar").unwrap())]
        .into_iter()
        .collect::<TypeEnvironment>();
    let compiled = compile_expression("input", &types, &ExpressionContext::empty()).unwrap();
    let mut program = compiled.core().clone();

    let metadata = &mut program.blocks[0].instructions[0].metadata;
    let mut both_origins = DependencyMask::input_shape(InputId::new(0));
    both_origins.union(&DependencyMask::input_leaf(InputId::new(0)));
    metadata.shape_dependencies = both_origins.clone();
    metadata.leaf_dependencies = both_origins;
    let with_all_matrix_cells = program.retained_bytes().unwrap();

    let metadata = &mut program.blocks[0].instructions[0].metadata;
    metadata.shape_dependencies = DependencyMask::default();
    metadata.leaf_dependencies = DependencyMask::default();
    let without_dependencies = program.retained_bytes().unwrap();

    assert_eq!(
        with_all_matrix_cells - without_dependencies,
        4 * size_of::<InputId>()
    );
}

#[test]
fn recursive_value_type_payload_is_counted_logically() {
    let unit = size_of::<ValueType>();
    assert_eq!(value_type_bytes(&ValueType::parse("int").unwrap()), Some(0));
    assert_eq!(
        value_type_bytes(&ValueType::parse("list<list<int>>").unwrap()),
        Some(2 * unit)
    );
    assert_eq!(
        value_type_bytes(&ValueType::parse("range<int>").unwrap()),
        Some(unit)
    );
    assert_eq!(
        value_type_bytes(&ValueType::parse("(int, list<int>)").unwrap()),
        Some(3 * unit)
    );
}

#[test]
fn retained_core_counts_type_table_entries_and_recursive_payload() {
    let primitive = program_for_type("int");
    let list = program_for_type("list<int>");
    assert_eq!(
        list.retained_bytes().unwrap() - primitive.retained_bytes().unwrap(),
        size_of::<CoreTypeEntry>() + size_of::<ValueType>()
    );
}

#[test]
fn retained_function_counts_recursive_signature_types() {
    let primitive = function_for_type("int");
    let list = function_for_type("list<int>");
    assert_eq!(
        list.retained_bytes().unwrap() - primitive.retained_bytes().unwrap(),
        size_of::<CoreTypeEntry>() + 3 * size_of::<ValueType>()
    );
}

#[test]
fn internal_core_type_has_only_its_entry_payload() {
    let mut program = program_for_type("map<text, int>");
    let before = program.retained_bytes().unwrap();
    let map_type = program.result_type;
    let id = CoreTypeId::new(program.types.entries.len() as u32);
    program.types.entries.push(CoreTypeEntry {
        id,
        kind: CoreType::MapBuilder { map_type },
    });
    assert_eq!(
        program.retained_bytes().unwrap() - before,
        size_of::<CoreTypeEntry>()
    );
}

fn program_for_type(value_type: &str) -> super::super::CoreProgram {
    let types = [("input".to_owned(), ValueType::parse(value_type).unwrap())]
        .into_iter()
        .collect::<TypeEnvironment>();
    compile_expression("input", &types, &ExpressionContext::empty())
        .unwrap()
        .core()
        .clone()
}

#[path = "tests/closure.rs"]
mod closure;
#[path = "tests/dependency.rs"]
mod dependency;
#[path = "tests/vectors.rs"]
mod vectors;

fn function_for_type(value_type: &str) -> std::sync::Arc<super::super::CompiledFunction> {
    let value_type = ValueType::parse(value_type).unwrap();
    let definition = FunctionDefinition::new(
        "identity",
        vec![FunctionParameter::new("value", value_type.clone())],
        value_type,
        "{value}",
    );
    compile_functions(&ExpressionContext::empty(), &[definition])
        .unwrap()
        .functions()
        .lookup("identity")
        .unwrap()
        .clone()
}
