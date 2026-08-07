mod aggregate_verifier;
mod closure_verifier;
mod for_each;
mod input_type_order;
mod internal_escape;
mod local_mutation;
mod lowering;
mod map_protocol;
mod nominal_corruption;
pub(in crate::program::expression::core) mod nominal_fixture;
mod nominal_input_corruption;
mod nominal_verifier;
mod non_escaping;
mod range_verifier;
mod structural_verifier;
mod temporal_attachment;
mod trusted_input;
mod type_table;
mod verifier;

use super::{CoreProgram, FunctionRegistry};
use crate::program::expression::{
    compile_expression, ExpressionContext, ExpressionError, FunctionMap, TypeEnvironment,
};

fn raw(source: &str) -> (CoreProgram, FunctionMap) {
    raw_with_types(source, &TypeEnvironment::new())
}

fn raw_with_types(source: &str, types: &TypeEnvironment) -> (CoreProgram, FunctionMap) {
    let functions = FunctionMap::new();
    let context = ExpressionContext::empty().with_functions(functions.clone());
    let program = compile_expression(source, types, &context)
        .unwrap()
        .core()
        .clone();
    (program, functions)
}

fn verify_error(program: CoreProgram, functions: &FunctionMap) -> ExpressionError {
    super::verify(program, functions.registry(), &[])
        .expect_err("mutated Core must fail verification")
}

fn empty_registry() -> FunctionRegistry {
    FunctionRegistry::default()
}
