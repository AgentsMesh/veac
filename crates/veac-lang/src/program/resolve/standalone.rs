use super::{Resolver, Scope};
use crate::authoring::Span;
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
) -> Result<Scope, Vec<Diagnostic>> {
    validate_source_id(&root.id).map_err(|message| vec![source_id_error(&root.id, message)])?;
    let mut source_budget = SourceBudget::default();
    source_budget
        .add(&root.id, &root.source, Span::default())
        .map_err(|error| vec![error])?;
    let file = parser::parse_executable(&root.id, &root.source)?;
    if !matches!(file.kind, FileKind::Module) {
        return Err(vec![Diagnostic::new(
            "PROGRAM_FORMAT_EXPECTED_MODULE",
            &root.id,
            "standalone module validation requires a module source",
            Span::default(),
        )]);
    }
    let mut resolver = Resolver::new(loader, source_budget, execution);
    resolver
        .sources
        .insert(root.id.clone(), root.source.clone());
    resolver.active.push(root.id);
    let result = resolver.scope(&file, false).map_err(|error| vec![error]);
    resolver.active.pop();
    result
}

fn source_id_error(path: &str, message: String) -> Diagnostic {
    Diagnostic::new("PROGRAM_SOURCE_ID", path, message, Span::default())
}
