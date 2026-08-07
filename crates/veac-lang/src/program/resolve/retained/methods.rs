use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;
use crate::program::expression::FunctionMap;
use crate::program::model::SurfaceFile;
use crate::program::{MethodDefinition, MethodRegistry};

pub(super) fn compiled(
    file: &SurfaceFile,
    functions: &FunctionMap,
    methods: &MethodRegistry,
    mut bytes: usize,
    limit: usize,
) -> Result<usize, Diagnostic> {
    for method in methods
        .definitions()
        .filter(|method| method.owner_source() == file.path && method.body().is_some())
    {
        let span = span(file, method);
        let payload = functions
            .lookup_by_id(method.signature().function_id())
            .expect("compiled local method must exist")
            .retained_bytes()
            .ok_or_else(|| super::error(&file.path, span))?;
        bytes = bytes
            .checked_add(payload)
            .filter(|next| *next <= limit)
            .ok_or_else(|| super::error(&file.path, span))?;
    }
    Ok(bytes)
}

fn span(file: &SurfaceFile, method: &MethodDefinition) -> Span {
    file.implementations
        .iter()
        .flat_map(|implementation| &implementation.methods)
        .find(|declaration| declaration.name == method.signature().name())
        .map(|declaration| declaration.span)
        .unwrap_or_default()
}
