use super::super::diagnostic::Diagnostics;
use super::super::expression::ExecutionBudget;
use super::super::loader::{LoadedSource, SourceLoader};
use super::super::model::FileKind;
use super::super::{executable, parser, resolve};

pub(super) fn validate(
    source: &LoadedSource,
    loader: &dyn SourceLoader,
) -> Result<FileKind, Diagnostics> {
    let file = parser::parse_executable(&source.id, &source.source).map_err(Diagnostics)?;
    let kind = file.kind;
    validate_as(source.clone(), loader, kind.clone())?;
    Ok(kind)
}

pub(super) fn validate_as(
    source: LoadedSource,
    loader: &dyn SourceLoader,
    kind: FileKind,
) -> Result<(), Diagnostics> {
    match kind {
        FileKind::Entry => executable::prepare_with_loader(source, loader).map(|_| ()),
        FileKind::Module => resolve::standalone_module(source, loader, &ExecutionBudget::default())
            .map(|_| ())
            .map_err(Diagnostics),
    }
}
