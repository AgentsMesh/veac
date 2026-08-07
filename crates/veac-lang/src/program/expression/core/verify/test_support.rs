impl super::VerifiedCoreProgram {
    pub(in crate::program::expression) fn core_mut(&mut self) -> &mut super::CoreProgram {
        &mut self.core
    }
}

pub(super) fn raw(source: &str) -> (super::CoreProgram, crate::program::expression::FunctionMap) {
    raw_with_types(source, &crate::program::expression::TypeEnvironment::new())
}

pub(super) fn raw_with_types(
    source: &str,
    types: &crate::program::expression::TypeEnvironment,
) -> (super::CoreProgram, crate::program::expression::FunctionMap) {
    let functions = crate::program::expression::FunctionMap::new();
    let context =
        crate::program::expression::ExpressionContext::empty().with_functions(functions.clone());
    let program = crate::program::expression::compile_expression(source, types, &context)
        .unwrap()
        .core()
        .clone();
    (program, functions)
}
