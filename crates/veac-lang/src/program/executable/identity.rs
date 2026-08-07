use std::collections::BTreeMap;

use crate::program::diagnostic::Diagnostic;
use crate::program::expression::{CompiledFunction, ProgramIdentity};
use crate::source_edit::{source_graph_revision, SourceModule};

pub(super) fn read(
    main: &CompiledFunction,
    sources: &BTreeMap<String, String>,
    root_module: &str,
    declared_inputs_sha256: String,
    span: crate::authoring::Span,
) -> Result<ProgramIdentity, Diagnostic> {
    let modules = sources
        .iter()
        .map(|(path, source)| SourceModule::utf8(path, source))
        .collect::<Vec<_>>();
    let revision = source_graph_revision(&modules).map_err(|error| {
        Diagnostic::new(
            "PROGRAM_EXECUTABLE_IDENTITY",
            root_module,
            format!("executable source identity failed: {error}"),
            span,
        )
    })?;
    Ok(ProgramIdentity {
        core_version: main.body().version(),
        domain_opset: main.body().domain_opset().raw(),
        domain_registry_sha256: main.body().domain_registry_digest().to_string(),
        main_content_sha256: main.content_digest().to_string(),
        source_graph_sha256: revision.source_graph_sha256,
        declared_inputs_sha256,
    })
}
