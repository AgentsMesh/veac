use std::collections::BTreeSet;

use crate::program::expression::{
    self, CoreProgram, ExpressionContext, ExpressionError, TypeEnvironment,
};
use crate::program::syntax_document::{SyntaxDocument, SyntaxSlice};

pub(super) fn names(
    document: &SyntaxDocument,
    slice: &SyntaxSlice,
    declarations: &TypeEnvironment,
    context: &ExpressionContext,
) -> Result<BTreeSet<String>, ExpressionError> {
    let mut names = expression::referenced_value_symbols_slice(document, slice, &|name| {
        declarations.contains_key(name)
    })?;
    let compiled = expression::compile_expression_slice(document, slice, declarations, context)?;
    collect_inputs(compiled.core(), declarations, &mut names);
    let mut pending = compiled.core().called_functions();
    let mut visited = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !visited.insert(id) {
            continue;
        }
        let function = context
            .functions()
            .registry()
            .get(id)
            .expect("verified function call target is registered");
        collect_inputs(function.body(), declarations, &mut names);
        pending.extend(function.body().called_functions());
    }
    Ok(names)
}

fn collect_inputs(
    program: &CoreProgram,
    declarations: &TypeEnvironment,
    names: &mut BTreeSet<String>,
) {
    names.extend(
        program
            .inputs()
            .iter()
            .map(|input| input.name())
            .filter(|name| declarations.contains_key(*name))
            .map(str::to_owned),
    );
}
