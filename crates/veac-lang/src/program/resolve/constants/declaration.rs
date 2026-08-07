use std::collections::BTreeSet;

use crate::program::diagnostic::Diagnostic;
use crate::program::expression::TypeEnvironment;
use crate::program::model::{Scope, SurfaceFile};

pub(in crate::program::resolve) fn types(
    file: &SurfaceFile,
    scope: &Scope,
) -> Result<TypeEnvironment, Diagnostic> {
    let mut names = BTreeSet::new();
    let mut types = TypeEnvironment::new();
    for declaration in &file.constants {
        if !names.insert(declaration.name.as_str()) || scope.values.contains_key(&declaration.name)
        {
            return Err(super::duplicate(file, declaration));
        }
        let value_type = super::super::type_annotations::resolve_in(
            &file.path,
            scope.types.as_ref(),
            &declaration.type_syntax,
        )?;
        types.insert(declaration.name.clone(), value_type);
    }
    Ok(types)
}
