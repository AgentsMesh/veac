use crate::program::expression::core::{
    CompiledFunction, CoreInstruction, CoreProgram, CoreTerminator,
};
use crate::program::expression::ExpressionError;

pub(super) fn contract(message: &str, instruction: &CoreInstruction) -> ExpressionError {
    ExpressionError::new(
        "EXPRESSION_RUNTIME_CONTRACT",
        message,
        instruction.span.clone(),
    )
}

pub(super) fn annotate<T>(
    result: Result<T, ExpressionError>,
    function: Option<&CompiledFunction>,
) -> Result<T, ExpressionError> {
    match function {
        Some(function) => {
            result.map_err(|error| error.in_runtime_function(function.name(), function.origin()))
        }
        None => result,
    }
}

pub(super) fn program_span(program: &CoreProgram) -> std::ops::Range<usize> {
    let Some(block) = program.blocks.first() else {
        return 0..0;
    };
    block
        .instructions
        .first()
        .map(|instruction| instruction.span.clone())
        .unwrap_or_else(|| match &block.terminator {
            CoreTerminator::Return { span, .. }
            | CoreTerminator::Jump { span, .. }
            | CoreTerminator::Branch { span, .. }
            | CoreTerminator::Match { span, .. } => span.clone(),
            CoreTerminator::ForEach(value) => value.provenance().loop_span().clone(),
        })
}
