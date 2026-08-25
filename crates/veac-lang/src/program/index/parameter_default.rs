use crate::program::diagnostic::Diagnostic;
use crate::program::model::{FunctionParameterDecl, SurfaceFile};
use crate::source_edit::{ExpressionSite, SourceNodeRef};

use super::SourceIndex;

pub(super) fn index(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: &SourceNodeRef,
    parameters: &[FunctionParameterDecl],
) -> Result<(), Diagnostic> {
    for parameter in parameters {
        let Some(default) = &parameter.default else {
            continue;
        };
        index.insert(
            &file.path,
            target.clone(),
            ExpressionSite::ParameterDefault {
                parameter: parameter.name.clone(),
            },
            file.syntax.slice_text(&default.syntax),
            default.span,
        )?;
    }
    Ok(())
}
