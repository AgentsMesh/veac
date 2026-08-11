use super::{Resolution, Resolver};
use crate::program::diagnostic::Diagnostic;
use crate::program::expression::ExecutionBudget;
use crate::program::limits::SourceBudget;
use crate::program::loader::{validate_source_id, LoadedSource, SourceLoader};
use crate::program::model::FileKind;
use crate::program::parser;

pub(crate) fn resolve(
    root: LoadedSource,
    loader: &dyn SourceLoader,
    execution: &ExecutionBudget,
    contract: &crate::program::EntryContract,
) -> Result<Resolution, Vec<Diagnostic>> {
    validate_source_id(&root.id).map_err(|message| {
        vec![Diagnostic::new(
            "PROGRAM_SOURCE_ID",
            &root.id,
            message,
            crate::authoring::Span::default(),
        )]
    })?;
    let mut source_budget = SourceBudget::default();
    source_budget
        .add(&root.id, &root.source, crate::authoring::Span::default())
        .map_err(|error| vec![error])?;
    let entry = parser::parse_executable(&root.id, &root.source)?;
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
    let mut resolver = Resolver::new(loader, source_budget, execution);
    resolver
        .sources
        .insert(root.id.clone(), root.source.clone());
    resolver.active.push(root.id.clone());
    let result = resolver.entry_scope(&entry, contract.preludes());
    resolver.active.pop();
    let scope = result.map_err(|error| vec![error])?;
    Ok(Resolution {
        entry,
        scope,
        sources: resolver.sources,
    })
}
