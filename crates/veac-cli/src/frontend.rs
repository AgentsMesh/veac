use std::path::Path;

use crate::diagnostic;
use crate::error::CliResult;
use crate::fs;

mod inputs;
mod source;

pub(crate) use inputs::build_inputs;
pub(crate) use source::{prepare_source_graph, read_source_graph};

pub(crate) fn check(
    path: &Path,
    inputs: Option<&Path>,
    inline_inputs: &[String],
    revision: u64,
) -> CliResult<veac_ir::ProjectEnvelope> {
    build_graph(path, inputs, inline_inputs, revision).map(|(envelope, _, _)| envelope)
}

pub(crate) fn build_graph(
    path: &Path,
    inputs: Option<&Path>,
    inline_inputs: &[String],
    revision: u64,
) -> CliResult<(
    veac_ir::ProjectEnvelope,
    veac_lang::program::BuiltProgram,
    std::path::PathBuf,
)> {
    let (root, program) = read_source_graph(path, |location| {
        let prepared = veac_lang::program::prepare_path(location.path())
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

pub(crate) fn format(path: &Path) -> CliResult<(String, String)> {
    read_source_graph(path, |location| {
        let source = fs::read_utf8(location.path(), "source")?;
        let formatted = veac_lang::program::format_path(location.path())
            .map_err(|errors| diagnostic::program(path, errors))?;
        Ok((source, formatted))
    })
    .map(|(_, formatted)| formatted)
}
