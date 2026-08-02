use std::path::Path;

use crate::diagnostic;
use crate::error::CliResult;
use crate::fs;

pub(crate) fn compile(path: &Path, revision: u64) -> CliResult<veac_ir::ProjectEnvelope> {
    let source = fs::read_utf8(path, "source")?;
    let document = veac_lang::authoring::parse(&source)
        .map_err(|errors| diagnostic::authoring(path, &source, errors))?;
    let mut envelope = veac_lang::authoring::lower_document(&document)
        .map_err(|errors| diagnostic::authoring(path, &source, errors))?;
    envelope.project.revision = revision;
    Ok(envelope)
}

pub(crate) fn format(path: &Path) -> CliResult<(String, String)> {
    let source = fs::read_utf8(path, "source")?;
    let document = veac_lang::authoring::parse(&source)
        .map_err(|errors| diagnostic::authoring(path, &source, errors))?;
    Ok((source, veac_lang::authoring::format_document(&document)))
}
