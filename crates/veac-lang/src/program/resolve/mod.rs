mod active;
mod constants;
mod dependency_routes;
mod entry;
mod functions;
mod inputs;
mod methods;
mod module;
mod names;
mod prelude;
mod retained;
mod standalone;
mod type_annotations;
mod types;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::compiler_database::DependencyRouteAdmission;
use super::diagnostic::Diagnostic;
use super::expression::ExecutionBudget;
use super::limits::SourceBudget;
use super::loader::{LoadedSource, SourceLoader};
use super::model::{Scope, SurfaceFile};
use super::CompilerDatabase;

pub(crate) use entry::resolve_with_database as contract_entry_with_database;
pub(crate) use standalone::resolve as standalone_module;

pub(crate) fn module_interface_scope(
    root: LoadedSource,
    loader: &dyn SourceLoader,
    execution: &ExecutionBudget,
    database: &CompilerDatabase,
) -> Result<Scope, Vec<Diagnostic>> {
    standalone::resolve_with_database(root, loader, execution, database, true)
}

pub(crate) fn executable_entry_with_database(
    root: LoadedSource,
    loader: &dyn SourceLoader,
    execution: &ExecutionBudget,
    database: &CompilerDatabase,
) -> Result<Resolution, Vec<Diagnostic>> {
    entry::resolve_with_database(
        root,
        loader,
        execution,
        &crate::program::EntryContract::video(),
        database,
    )
}

pub(crate) struct Resolution {
    pub entry: SurfaceFile,
    pub scope: Scope,
}
struct Resolver<'a> {
    loader: &'a dyn SourceLoader,
    cache: BTreeMap<String, Arc<Scope>>,
    routes: BTreeMap<String, BTreeMap<String, String>>,
    route_admissions: BTreeMap<String, DependencyRouteAdmission>,
    retry_required: bool,
    active: Vec<String>,
    sources: BTreeMap<String, String>,
    budget: SourceBudget,
    retained: retained::Budget,
    execution: &'a ExecutionBudget,
    database: &'a CompilerDatabase,
}

impl<'a> Resolver<'a> {
    fn new(
        loader: &'a dyn SourceLoader,
        budget: SourceBudget,
        execution: &'a ExecutionBudget,
        database: &'a CompilerDatabase,
    ) -> Self {
        Self {
            loader,
            cache: BTreeMap::new(),
            routes: BTreeMap::new(),
            route_admissions: BTreeMap::new(),
            retry_required: false,
            active: Vec::new(),
            sources: BTreeMap::new(),
            budget,
            retained: retained::Budget::default(),
            execution,
            database,
        }
    }

    fn scope(&mut self, file: &SurfaceFile, exports_only: bool) -> Result<Scope, Diagnostic> {
        self.scope_from(file, exports_only, Scope::default())
    }

    fn scope_from(
        &mut self,
        file: &SurfaceFile,
        exports_only: bool,
        mut scope: Scope,
    ) -> Result<Scope, Diagnostic> {
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
        let admission = self
            .route_admissions
            .get(&file.path)
            .expect("parsed file has dependency route admission");
        let exported_functions = functions::resolve(
            file,
            &mut scope,
            &mut self.retained,
            self.database,
            admission,
        )?;
        if exports_only {
            types::retain_exports(file, &mut scope, &exported_types)?;
            methods::retain_exports(file, &mut scope, &exported_types);
            functions::retain_exports(&mut scope, &exported_functions);
            Arc::make_mut(&mut scope.values).retain(|name, _| exported_values.contains(name));
        }
        Ok(scope)
    }

    fn imports(&mut self, file: &SurfaceFile, scope: &mut Scope) -> Result<(), Diagnostic> {
        let mut aliases = BTreeSet::new();
        let mut routes = self.routes.remove(&file.path).unwrap_or_default();
        for import in &file.imports {
            if !aliases.insert(import.alias.clone()) {
                return Err(Diagnostic::new(
                    "PROGRAM_IMPORT_ALIAS",
                    &file.path,
                    format!("import alias `{}` is duplicated", import.alias),
                    import.span,
                ));
            }
            let (imported, resolved_source_id) =
                self.module(&file.path, &import.path, import.span)?;
            routes.insert(import.path.clone(), resolved_source_id);
            names::namespace(
                &file.path,
                &import.alias,
                imported.as_ref(),
                scope,
                import.span,
                &mut self.retained,
            )?;
        }
        self.commit_dependency_routes(file, routes);
        Ok(())
    }
}
