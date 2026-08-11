use std::path::{Component, Path, PathBuf};

use veac_build::{ProducedProjectOutput, ProjectBackendError, ProjectComputation};
use veac_ir::{Deliverable, DeliverableKind, DeliverableTarget};
use veac_project::{MediaType, ProjectOutput};

use super::super::{failed, ProjectResultExt};

pub(super) fn bind(
    workspace: &Path,
    computation: &ProjectComputation,
    built: &veac_lang::program::BuiltProgram,
    plan: &veac_plan::ResolvedRenderPlan,
    bindings: &mut veac_artifact::ExecutionBindings,
) -> Result<Vec<ProducedProjectOutput>, ProjectBackendError> {
    if computation.outputs.len() != plan.output.deliverables.len() {
        return Err(failed(
            "project outputs do not match target source deliverables",
        ));
    }
    let mut produced = Vec::with_capacity(computation.outputs.len());
    for output in &computation.outputs {
        let Some(deliverable) =
            built.deliverable_by_logical_key(&plan.output.render_config_id, output.id().as_str())
        else {
            return Err(failed(format!(
                "target omitted project output logical key '{}'",
                output.id()
            )));
        };
        compatible(output, deliverable)?;
        let relative = relative_path(output, deliverable)?;
        let destination = workspace.join(&relative);
        let Some(parent) = destination.parent() else {
            return Err(failed("target output has no parent directory"));
        };
        std::fs::create_dir_all(parent).project_context("cannot create target output directory")?;
        bindings
            .bind_output(deliverable.id.clone(), destination)
            .project_context("cannot bind target output")?;
        produced.push(ProducedProjectOutput {
            output: output.id().clone(),
            relative_path: relative,
            semantics: veac_build::ProjectOutputSemantics::Opaque,
        });
    }
    Ok(produced)
}

fn relative_path(
    output: &ProjectOutput,
    deliverable: &Deliverable,
) -> Result<PathBuf, ProjectBackendError> {
    let DeliverableTarget::File { name } = &deliverable.target else {
        return Err(failed(
            "project runtime currently requires file target deliverables",
        ));
    };
    let name = Path::new(name);
    if name.components().count() != 1
        || !matches!(name.components().next(), Some(Component::Normal(_)))
    {
        return Err(failed("target deliverable name must be one path segment"));
    }
    Ok(Path::new("outputs").join(output.id().as_str()).join(name))
}

fn compatible(
    output: &ProjectOutput,
    deliverable: &Deliverable,
) -> Result<(), ProjectBackendError> {
    if is_compatible(output, deliverable) {
        Ok(())
    } else {
        Err(failed(format!(
            "project output '{}' kind differs from target deliverable",
            output.id()
        )))
    }
}

fn is_compatible(output: &ProjectOutput, deliverable: &Deliverable) -> bool {
    match output {
        ProjectOutput::Media {
            media_type: MediaType::Video,
            ..
        } => matches!(deliverable.kind, DeliverableKind::Video(_)),
        ProjectOutput::Media {
            media_type: MediaType::Audio,
            ..
        } => matches!(
            deliverable.kind,
            DeliverableKind::AudioStem(_) | DeliverableKind::AudioFile(_)
        ),
        ProjectOutput::Media {
            media_type: MediaType::Image,
            ..
        } => matches!(
            deliverable.kind,
            DeliverableKind::StillImage(_)
                | DeliverableKind::AnimatedImage(_)
                | DeliverableKind::Scope(_)
        ),
        ProjectOutput::Data { .. } => {
            matches!(deliverable.kind, DeliverableKind::CaptionSidecar(_))
        }
        ProjectOutput::Directory { .. } => false,
    }
}
