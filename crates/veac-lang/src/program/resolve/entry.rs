use super::{Resolution, Resolver};
use crate::program::compiler_database::DEPENDENCY_ROUTE_ATTEMPTS;
use crate::program::diagnostic::Diagnostic;
use crate::program::expression::ExecutionBudget;
use crate::program::limits::SourceBudget;
use crate::program::loader::{validate_source_id, LoadedSource, SourceLoader};
use crate::program::model::FileKind;
use crate::program::CompilerDatabase;

pub(crate) fn resolve_with_database(
    root: LoadedSource,
    loader: &dyn SourceLoader,
    execution: &ExecutionBudget,
    contract: &crate::program::EntryContract,
    database: &CompilerDatabase,
) -> Result<Resolution, Vec<Diagnostic>> {
    validate_source_id(&root.id).map_err(|message| {
        vec![Diagnostic::new(
            "PROGRAM_SOURCE_ID",
            &root.id,
            message,
            crate::authoring::Span::default(),
        )]
    })?;
    for _ in 0..DEPENDENCY_ROUTE_ATTEMPTS {
        let (resolution, retry) = attempt(root.clone(), loader, execution, contract, database)?;
        if !retry {
            return Ok(resolution);
        }
    }
    Err(super::dependency_routes::query_changed(&root.id))
}

fn attempt(
    root: LoadedSource,
    loader: &dyn SourceLoader,
    execution: &ExecutionBudget,
    contract: &crate::program::EntryContract,
    database: &CompilerDatabase,
) -> Result<(Resolution, bool), Vec<Diagnostic>> {
    let mut source_budget = SourceBudget::default();
    source_budget
        .add(&root.id, &root.source, crate::authoring::Span::default())
        .map_err(|error| vec![error])?;
    let (entry, admission) = database.reuse_parse_with_route_admission(&root.id, &root.source)?;
    if !matches!(entry.kind, FileKind::Entry) {
        return Err(vec![Diagnostic::new(
            "PROGRAM_ENTRY_MODULE",
            &root.id,
            "entry path contains a module instead of a project",
            crate::authoring::Span::default(),
        )]);
    }
    contract
        .validate_surface(&entry)
        .map_err(|error| vec![error])?;
    let mut resolver = Resolver::new(loader, source_budget, execution, database);
    resolver.route_admissions.insert(root.id.clone(), admission);
    resolver
        .sources
        .insert(root.id.clone(), root.source.clone());
    resolver.active.push(root.id.clone());
    let result = resolver.entry_scope(&entry, contract.preludes());
    resolver.active.pop();
    let scope = result.map_err(|error| vec![error])?;
    Ok((
        Resolution {
            entry: (*entry).clone(),
            scope,
        },
        resolver.retry_required,
    ))
}
