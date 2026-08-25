use crate::source_edit::{BodySite, SourceNodeRef};

use super::super::diagnostic::Diagnostic;
use super::super::model::SurfaceFile;
use super::SourceIndex;

pub(super) fn index(index: &mut SourceIndex, file: &SurfaceFile) -> Result<(), Diagnostic> {
    for implementation in &file.implementations {
        let receiver = implementation.target.to_string();
        for method in &implementation.methods {
            let target = SourceNodeRef::method(&file.path, &receiver, &method.name);
            index.register(&file.path, target.clone(), method.syntax.span)?;
            super::parameter_default::index(index, file, &target, &method.parameters)?;
            index.insert_body(
                &file.path,
                target.clone(),
                BodySite::MethodBody,
                file.syntax.slice_text(&method.body.syntax),
                method.body.span,
            )?;
            super::expression::index(index, file, target.clone(), &method.body)?;
            super::component_animation::index(index, file, target, &method.body)?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "method_tests.rs"]
mod tests;
