use std::path::Path;
use std::path::PathBuf;

use crate::diagnostic;
use crate::error::CliResult;

mod inputs;
mod source;

pub(crate) use inputs::build_inputs;
pub(crate) use source::{
    prepare_source_graph, read_source_graph, source_graph_revision, source_loader_unlocked,
};

pub(crate) fn check(
    path: &Path,
    inputs: Option<&Path>,
    inline_inputs: &[String],
    package_roots: &[PathBuf],
    revision: u64,
) -> CliResult<veac_ir::ProjectEnvelope> {
    build_graph(path, inputs, inline_inputs, package_roots, revision)
        .map(|(envelope, _, _)| envelope)
}

pub(crate) fn build_graph(
    path: &Path,
    inputs: Option<&Path>,
    inline_inputs: &[String],
    package_roots: &[PathBuf],
    revision: u64,
) -> CliResult<(
    veac_ir::ProjectEnvelope,
    veac_lang::program::BuiltProgram,
    std::path::PathBuf,
)> {
    let (root, program) = read_source_graph(path, package_roots, |_location, entry, loader| {
        let prepared = veac_lang::program::prepare_with_loader(entry, loader)
            .map_err(|errors| diagnostic::program(path, errors))?;
        let manifest = build_inputs(inputs, inline_inputs, &prepared)?;
        prepared
            .execute_with_inputs(&manifest)
            .map_err(|errors| diagnostic::program(path, errors))
    })?;
    let mut envelope = program.envelope().clone();
    envelope.project.revision = revision;
    veac_ir::validate(&envelope).map_err(|errors| diagnostic::ir(errors.diagnostics()))?;
    Ok((envelope, program, root))
}

pub(crate) fn format(path: &Path, package_roots: &[PathBuf]) -> CliResult<(String, String)> {
    read_source_graph(path, package_roots, |_location, entry, loader| {
        let source = entry.source.clone();
        let formatted = veac_lang::program::format_source_with_loader(entry, loader)
            .map_err(|errors| diagnostic::program(path, errors))?;
        Ok((source, formatted))
    })
    .map(|(_, formatted)| formatted)
}
