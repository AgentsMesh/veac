use std::path::Path;

use veac_build::{
    CancellationToken, ProducedProjectOutput, ProjectArtifactInput, ProjectBackendError,
    ProjectComputation, ProjectFileSnapshot, ProjectSourceGraphRevision,
};
use veac_runtime::executor::BundleExecutor;

use super::{failed, inputs, CliProjectBackend, ProjectResultExt};

mod hydrate;
mod outputs;
mod source_graph;

pub(super) fn execute(
    backend: &CliProjectBackend,
    workspace: &Path,
    computation: &ProjectComputation,
    snapshot: &ProjectFileSnapshot,
    source_graph: &ProjectSourceGraphRevision,
    artifacts: &[ProjectArtifactInput],
    cancellation: &CancellationToken,
) -> Result<Vec<ProducedProjectOutput>, ProjectBackendError> {
    let prepared = source_graph::prepare(
        &backend.source_root,
        &backend.packages,
        snapshot,
        source_graph,
    )?;
    let bound = inputs::bind(&prepared, computation, artifacts, &backend.material_root)?;
    let built = prepared
        .execute_with_inputs(&bound.manifest)
        .project_context("target source execution failed")?;
    let mut envelope = built.envelope().clone();
    let config = select_config(&envelope)?;
    let required = veac_plan::required_material_ids_one(&envelope, &config)
        .project_context("target material reachability failed")?;
    let material_paths = hydrate::materials(&mut envelope, &required, &bound.materials)?;
    veac_ir::validate(&envelope).project_context("target canonical IR is invalid")?;
    let plan =
        veac_plan::resolve_one(&envelope, &config).project_context("target planning failed")?;
    let mut bindings = crate::planning::input_bindings(&plan, &material_paths)
        .project_context("target execution binding failed")?;
    let produced = outputs::bind(workspace, computation, &built, &plan, &mut bindings)?;
    let bundle = veac_codegen::emitter::emit_all(&plan, &bindings)
        .project_context("target code generation failed")?;
    if cancellation.is_cancelled() {
        return Err(ProjectBackendError::cancelled(
            "project execution was cancelled before render",
        ));
    }
    BundleExecutor::new(backend.ffmpeg.clone())
        .execute(&bundle, &backend.store)
        .project_context("target render failed")?;
    if cancellation.is_cancelled() {
        return Err(ProjectBackendError::cancelled(
            "project execution was cancelled during render",
        ));
    }
    source_graph::verify(
        &backend.source_root,
        &backend.packages,
        snapshot,
        source_graph,
    )?;
    Ok(produced)
}

fn select_config(
    envelope: &veac_ir::ProjectEnvelope,
) -> Result<veac_ir::RenderConfigId, ProjectBackendError> {
    match envelope.project.render_configs.as_slice() {
        [config] => Ok(config.id.clone()),
        [] => Err(failed("target source produced no render configuration")),
        _ => Err(failed(
            "target source must reduce to exactly one render configuration",
        )),
    }
}

#[cfg(test)]
#[path = "render/tests.rs"]
mod tests;
