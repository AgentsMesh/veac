mod active;
mod components;
mod constants;
mod names;
mod presets;
mod retained;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::diagnostic::Diagnostic;
use super::expand::definition::Budget as DefinitionBudget;
use super::limits::SourceBudget;
use super::loader::{validate_source_id, LoadedSource, SourceLoader};
use super::model::{ComponentCatalog, FileKind, Scope, SurfaceFile};
use super::parser;
pub(crate) struct Resolution {
    pub entry: SurfaceFile,
    pub scope: Scope,
    pub component_catalog: ComponentCatalog,
    pub sources: BTreeMap<String, String>,
}
pub(crate) fn entry(
    root: LoadedSource,
    loader: &dyn SourceLoader,
) -> Result<Resolution, Vec<Diagnostic>> {
    validate_source_id(&root.id).map_err(|message| {
        vec![Diagnostic::new(
            "PROGRAM_SOURCE_ID",
            &root.id,
            message,
            crate::authoring::Span::default(),
        )]
    })?;
    let mut budget = SourceBudget::default();
    budget
        .add(&root.id, &root.source, crate::authoring::Span::default())
        .map_err(|error| vec![error])?;
    let entry = parser::parse(&root.id, &root.source)?;
    if !matches!(entry.kind, FileKind::Entry) {
        return Err(vec![Diagnostic::new(
            "PROGRAM_ENTRY_MODULE",
            &root.id,
            "entry path contains a module instead of a project",
            crate::authoring::Span::default(),
        )]);
    }
    let mut resolver = Resolver::new(loader, budget);
    resolver
        .sources
        .insert(root.id.clone(), root.source.clone());
    resolver.active.push(root.id.clone());
    let result = resolver.scope(&entry, false);
    resolver.active.pop();
    let scope = result.map_err(|error| vec![error])?;
    Ok(Resolution {
        entry,
        scope,
        component_catalog: resolver.component_catalog,
        sources: resolver.sources,
    })
}
struct Resolver<'a> {
    loader: &'a dyn SourceLoader,
    cache: BTreeMap<String, Arc<Scope>>,
    active: Vec<String>,
    sources: BTreeMap<String, String>,
    component_catalog: ComponentCatalog,
    budget: SourceBudget,
    definitions: DefinitionBudget,
    retained: retained::Budget,
}

impl<'a> Resolver<'a> {
    fn new(loader: &'a dyn SourceLoader, budget: SourceBudget) -> Self {
        Self {
            loader,
            cache: BTreeMap::new(),
            active: Vec::new(),
            sources: BTreeMap::new(),
            component_catalog: ComponentCatalog::new(),
            budget,
            definitions: DefinitionBudget::default(),
            retained: retained::Budget::default(),
        }
    }

    fn scope(&mut self, file: &SurfaceFile, exports_only: bool) -> Result<Scope, Diagnostic> {
        let mut scope = Scope::default();
        self.imports(file, &mut scope)?;
        let exported_values = constants::resolve(file, &mut scope, &mut self.retained)?;
        let exported_presets =
            presets::resolve(file, &mut scope, &mut self.definitions, &mut self.retained)?;
        let exported_components = components::resolve(
            file,
            &mut scope,
            &mut self.component_catalog,
            &mut self.definitions,
            &mut self.retained,
        )?;
        if exports_only {
            Arc::make_mut(&mut scope.values).retain(|name, _| exported_values.contains(name));
            Arc::make_mut(&mut scope.presets).retain(|key, _| exported_presets.contains(key));
            scope
                .components
                .retain(|name, _| exported_components.contains(name));
        }
        Ok(scope)
    }

    fn imports(&mut self, file: &SurfaceFile, scope: &mut Scope) -> Result<(), Diagnostic> {
        let mut aliases = BTreeSet::new();
        for import in &file.imports {
            if !aliases.insert(import.alias.clone()) {
                return Err(Diagnostic::new(
                    "PROGRAM_IMPORT_ALIAS",
                    &file.path,
                    format!("import alias `{}` is duplicated", import.alias),
                    import.span,
                ));
            }
            let imported = self.module(&file.path, &import.path, import.span)?;
            names::namespace(
                &file.path,
                &import.alias,
                imported.as_ref(),
                scope,
                import.span,
                &mut self.retained,
            )?;
        }
        Ok(())
    }

    fn module(
        &mut self,
        importer: &str,
        requested: &str,
        span: crate::authoring::Span,
    ) -> Result<Arc<Scope>, Diagnostic> {
        let loaded = self
            .loader
            .load(importer, requested)
            .map_err(|message| Diagnostic::new("PROGRAM_IMPORT_LOAD", importer, message, span))?;
        validate_source_id(&loaded.id)
            .map_err(|message| Diagnostic::new("PROGRAM_SOURCE_ID", importer, message, span))?;
        if self
            .sources
            .get(&loaded.id)
            .is_some_and(|source| source != &loaded.source)
        {
            return Err(Diagnostic::new(
                "PROGRAM_SOURCE_ID_COLLISION",
                importer,
                format!("source ID `{}` resolved to different contents", loaded.id),
                span,
            ));
        }
        if let Some(scope) = self.cache.get(&loaded.id) {
            return Ok(Arc::clone(scope));
        }
        active::check(&self.active, importer, &loaded.id, span)?;
        self.budget.add(&loaded.id, &loaded.source, span)?;
        self.active.push(loaded.id.clone());
        self.sources
            .insert(loaded.id.clone(), loaded.source.clone());
        let result = self.load_module(importer, &loaded, span);
        self.active.pop();
        let scope = Arc::new(result?);
        self.cache.insert(loaded.id, Arc::clone(&scope));
        Ok(scope)
    }

    fn load_module(
        &mut self,
        importer: &str,
        loaded: &LoadedSource,
        span: crate::authoring::Span,
    ) -> Result<Scope, Diagnostic> {
        let file = parser::parse(&loaded.id, &loaded.source).map_err(|errors| errors[0].clone())?;
        if !matches!(file.kind, FileKind::Module) {
            return Err(Diagnostic::new(
                "PROGRAM_IMPORT_PROJECT",
                importer,
                "imported file must contain a module",
                span,
            ));
        }
        self.scope(&file, true)
    }
}

#[cfg(test)]
mod tests;
