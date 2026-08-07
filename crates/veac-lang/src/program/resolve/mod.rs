mod active;
mod constants;
mod entry;
mod functions;
mod inputs;
mod methods;
mod names;
mod retained;
mod standalone;
mod type_annotations;
mod types;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::diagnostic::Diagnostic;
use super::expression::ExecutionBudget;
use super::limits::SourceBudget;
use super::loader::{validate_source_id, LoadedSource, SourceLoader};
use super::model::{FileKind, Scope, SurfaceFile};
use super::parser;

pub(crate) use entry::resolve as executable_entry;
pub(crate) use standalone::resolve as standalone_module;

pub(crate) struct Resolution {
    pub entry: SurfaceFile,
    pub scope: Scope,
    pub sources: BTreeMap<String, String>,
}
struct Resolver<'a> {
    loader: &'a dyn SourceLoader,
    cache: BTreeMap<String, Arc<Scope>>,
    active: Vec<String>,
    sources: BTreeMap<String, String>,
    budget: SourceBudget,
    retained: retained::Budget,
    execution: &'a ExecutionBudget,
}

impl<'a> Resolver<'a> {
    fn new(
        loader: &'a dyn SourceLoader,
        budget: SourceBudget,
        execution: &'a ExecutionBudget,
    ) -> Self {
        Self {
            loader,
            cache: BTreeMap::new(),
            active: Vec::new(),
            sources: BTreeMap::new(),
            budget,
            retained: retained::Budget::default(),
            execution,
        }
    }

    fn scope(&mut self, file: &SurfaceFile, exports_only: bool) -> Result<Scope, Diagnostic> {
        let mut scope = Scope::default();
        self.imports(file, &mut scope)?;
        let exported_types = types::resolve(file, &mut scope, &mut self.retained)?;
        inputs::resolve(file, &mut scope)?;
        methods::declare(file, &mut scope, &mut self.retained)?;
        types::validate_exports(file, &scope, &exported_types)?;
        let constant_types = constants::declaration_types(file, &scope)?;
        let provisional_functions = functions::provisional(file, &scope, &constant_types)?;
        let exported_values = constants::resolve(
            file,
            &mut scope,
            &mut self.retained,
            self.execution,
            provisional_functions,
            constant_types,
        )?;
        let exported_functions = functions::resolve(file, &mut scope, &mut self.retained)?;
        if exports_only {
            types::retain_exports(file, &mut scope, &exported_types)?;
            methods::retain_exports(&mut scope);
            functions::retain_exports(&mut scope, &exported_functions);
            Arc::make_mut(&mut scope.values).retain(|name, _| exported_values.contains(name));
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
