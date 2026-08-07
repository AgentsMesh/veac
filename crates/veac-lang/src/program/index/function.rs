use crate::source_edit::{BodySite, SourceNodeRef};

use super::super::diagnostic::Diagnostic;
use super::super::model::SurfaceFile;
use super::SourceIndex;

pub(super) fn index(index: &mut SourceIndex, file: &SurfaceFile) -> Result<(), Diagnostic> {
    for function in &file.functions {
        let target = SourceNodeRef::function(&file.path, &function.name);
        index.register(&file.path, target.clone(), function.span)?;
        index.insert_body(
            &file.path,
            target.clone(),
            BodySite::FunctionBody,
            &function.body.source,
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
