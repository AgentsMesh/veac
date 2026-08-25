use crate::source_edit::{BodySite, SourceNodeRef};

use super::super::diagnostic::Diagnostic;
use super::super::model::SurfaceFile;
use super::SourceIndex;

pub(super) fn index(index: &mut SourceIndex, file: &SurfaceFile) -> Result<(), Diagnostic> {
    for function in &file.functions {
        let target = SourceNodeRef::function(&file.path, &function.name);
        index.register(&file.path, target.clone(), function.syntax.span)?;
        super::parameter_default::index(index, file, &target, &function.parameters)?;
        index.insert_body(
            &file.path,
            target.clone(),
            BodySite::FunctionBody,
            file.syntax.slice_text(&function.body.syntax),
            function.body.span,
        )?;
        super::expression::index(index, file, target.clone(), &function.body)?;
        super::component_animation::index(index, file, target, &function.body)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "function_tests.rs"]
mod tests;
