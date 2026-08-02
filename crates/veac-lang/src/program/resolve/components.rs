use std::collections::BTreeSet;
use std::sync::Arc;

use crate::program::diagnostic::Diagnostic;
use crate::program::expand;
use crate::program::model::{
    CapturedScope, ComponentCatalog, ComponentInterface, ComponentKey, ResolvedComponent, Scope,
    SurfaceFile,
};

use super::{names, retained};

pub(super) fn resolve(
    file: &SurfaceFile,
    scope: &mut Scope,
    catalog: &mut ComponentCatalog,
    budget: &mut expand::definition::Budget,
    retained: &mut retained::Budget,
) -> Result<BTreeSet<String>, Diagnostic> {
    let mut exported = BTreeSet::new();
    let mut local = Vec::new();
    for declaration in &file.components {
        let key = ComponentKey {
            path: file.path.clone(),
            name: declaration.name.clone(),
        };
        names::insert(
            &file.path,
            "component",
            &declaration.name,
            &mut scope.components,
            declaration.name.clone(),
            key.clone(),
            declaration.span,
        )?;
        local.push((key, declaration));
    }
    if local.is_empty() {
        return Ok(exported);
    }
    let source: Arc<str> = Arc::from(file.source.as_str());
    let captured = Arc::new(CapturedScope {
        values: Arc::clone(&scope.values),
        presets: Arc::clone(&scope.presets),
        components: scope.components.clone(),
    });
    for (key, declaration) in &local {
        catalog.insert(
            key.clone(),
            Arc::new(ResolvedComponent {
                declaration: (*declaration).clone(),
                interface: ComponentInterface::from_declaration(declaration),
                source_path: file.path.clone(),
                source: Arc::clone(&source),
                captured: Arc::clone(&captured),
            }),
        );
        if declaration.exported {
            exported.insert(declaration.name.clone());
        }
    }
    for (key, _) in &local {
        let component = Arc::clone(&catalog[key]);
        expand::definition::component(&component, catalog, budget)?;
    }
    retained.components(file, &captured.components)?;
    Ok(exported)
}

#[cfg(test)]
mod tests;
