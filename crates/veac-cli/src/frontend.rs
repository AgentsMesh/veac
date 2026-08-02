use std::path::Path;

use crate::diagnostic;
use crate::error::CliResult;
use crate::fs;

pub(crate) fn compile(path: &Path, revision: u64) -> CliResult<veac_ir::ProjectEnvelope> {
    compile_graph(path, revision).map(|(envelope, _, _)| envelope)
}

pub(crate) fn compile_graph(
    path: &Path,
    revision: u64,
) -> CliResult<(
    veac_ir::ProjectEnvelope,
    veac_lang::program::CompiledProgram,
    std::path::PathBuf,
)> {
    let (root, program) = veac_lang::program::compile_path_with_root(path)
        .map_err(|errors| diagnostic::program(path, errors))?;
    let mut envelope = veac_lang::authoring::lower_document(program.document())
        .map_err(|errors| diagnostic::authoring(path, program.expanded_source(), errors))?;
    envelope.project.revision = revision;
    veac_ir::validate(&envelope).map_err(|errors| diagnostic::ir(errors.diagnostics()))?;
    Ok((envelope, program, root))
}

pub(crate) fn format(path: &Path) -> CliResult<(String, String)> {
    let source = fs::read_utf8(path, "source")?;
    let core_errors = match veac_lang::authoring::parse(&source) {
        Ok(document) => return Ok((source, veac_lang::authoring::format_document(&document))),
        Err(errors) => errors,
    };
    match veac_lang::program::compile_path(path) {
        Ok(_) => Ok((source.clone(), source)),
        Err(errors)
            if errors
                .as_slice()
                .iter()
                .all(|error| error.code == "PROGRAM_ENTRY_MODULE") =>
        {
            let label = path.file_name().unwrap_or_default().to_string_lossy();
            veac_lang::program::check_source(&label, &source)
                .map(|_| (source.clone(), source))
                .map_err(|errors| diagnostic::program(path, errors))
        }
        Err(errors) => {
            let label = path.file_name().unwrap_or_default().to_string_lossy();
            if veac_lang::program::check_source(&label, &source).is_ok() {
                Err(diagnostic::program(path, errors))
            } else {
                Err(diagnostic::authoring(path, &source, core_errors))
            }
        }
    }
}
