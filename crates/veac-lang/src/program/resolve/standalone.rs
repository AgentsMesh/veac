use super::{Resolver, Scope};
use crate::authoring::Span;
use crate::program::compiler_database::DEPENDENCY_ROUTE_ATTEMPTS;
use crate::program::diagnostic::Diagnostic;
use crate::program::expression::ExecutionBudget;
use crate::program::limits::SourceBudget;
use crate::program::loader::{validate_source_id, LoadedSource, SourceLoader};
use crate::program::model::FileKind;
use crate::program::CompilerDatabase;

pub(crate) fn resolve(
    root: LoadedSource,
    loader: &dyn SourceLoader,
    execution: &ExecutionBudget,
) -> Result<Scope, Vec<Diagnostic>> {
    let database = CompilerDatabase::default();
    resolve_with_database(root, loader, execution, &database, false)
}

pub(crate) fn resolve_with_database(
    root: LoadedSource,
    loader: &dyn SourceLoader,
    execution: &ExecutionBudget,
    database: &CompilerDatabase,
    exports_only: bool,
) -> Result<Scope, Vec<Diagnostic>> {
    validate_source_id(&root.id).map_err(|message| vec![source_id_error(&root.id, message)])?;
    for _ in 0..DEPENDENCY_ROUTE_ATTEMPTS {
        let (scope, retry) = attempt(root.clone(), loader, execution, database, exports_only)?;
        if !retry {
            return Ok(scope);
        }
    }
    Err(super::dependency_routes::query_changed(&root.id))
}

fn attempt(
    root: LoadedSource,
    loader: &dyn SourceLoader,
    execution: &ExecutionBudget,
    database: &CompilerDatabase,
    exports_only: bool,
) -> Result<(Scope, bool), Vec<Diagnostic>> {
    let mut source_budget = SourceBudget::default();
    source_budget
        .add(&root.id, &root.source, Span::default())
        .map_err(|error| vec![error])?;
    let (file, admission) = database.reuse_parse_with_route_admission(&root.id, &root.source)?;
    if !matches!(file.kind, FileKind::Module) {
        return Err(vec![Diagnostic::new(
            "PROGRAM_FORMAT_EXPECTED_MODULE",
            &root.id,
            "standalone module validation requires a module source",
            Span::default(),
        )]);
    }
    let mut resolver = Resolver::new(loader, source_budget, execution, database);
    resolver.route_admissions.insert(root.id.clone(), admission);
    resolver
        .sources
        .insert(root.id.clone(), root.source.clone());
    resolver.active.push(root.id);
    let result = resolver
        .scope(&file, exports_only)
        .map_err(|error| vec![error]);
    resolver.active.pop();
    result.map(|scope| (scope, resolver.retry_required))
}

fn source_id_error(path: &str, message: String) -> Diagnostic {
    Diagnostic::new("PROGRAM_SOURCE_ID", path, message, Span::default())
}
