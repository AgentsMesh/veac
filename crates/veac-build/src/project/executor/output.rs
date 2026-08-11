use std::{collections::BTreeMap, path::Path};

use veac_artifact::{
    ArtifactDependency, ArtifactDependencyRole, ArtifactDescriptor, ArtifactParameters,
    ArtifactStore, ContentDigest, ProducedArtifactParameters, ProducerFingerprint,
    RenderOutputParameters,
};
use veac_project::{MediaType, ProjectOutput};

use crate::{
    ArtifactOutputs, CancellationToken, ExecuteRequest, ExecutionError, PortName,
    ProducedProjectOutput, ProjectAction, ProjectOutputSemantics,
};

mod path;

pub(super) fn publish(
    store: &ArtifactStore,
    request: ExecuteRequest<'_, ProjectAction>,
    workspace: &Path,
    produced: &[ProducedProjectOutput],
    cancellation: &CancellationToken,
) -> Result<ArtifactOutputs, ExecutionError> {
    let mut by_id = BTreeMap::new();
    for output in produced {
        if by_id.insert(output.output.as_str(), output).is_some() {
            return Err(ExecutionError::failed(
                "backend produced a duplicate output",
            ));
        }
    }
    let expected = &request.action.computation().outputs;
    if by_id.len() != expected.len() {
        return Err(ExecutionError::failed(
            "backend output count does not match the project contract",
        ));
    }
    let dependencies = dependencies(&request);
    let mut artifacts = Vec::with_capacity(expected.len());
    for (index, output) in expected.iter().enumerate() {
        let produced = by_id.get(output.id().as_str()).ok_or_else(|| {
            ExecutionError::failed(format!("backend omitted output '{}'", output.id()))
        })?;
        let path = match output {
            ProjectOutput::Directory { .. } => {
                let source = path::checked_directory(workspace, &produced.relative_path)?;
                let archive = workspace.join(format!(".veac-directory-{index}.artifact"));
                super::super::directory::pack(&source, &archive, cancellation)?;
                archive
            }
            _ => path::checked_file(workspace, &produced.relative_path)?,
        };
        let (content, size) = path::fingerprint(&path, cancellation)?;
        let descriptor = descriptor(
            request.action,
            request.cache_key.digest(),
            output,
            &produced.semantics,
            index,
            &dependencies,
        )?;
        let record = store
            .put_file_expected_while(&descriptor, &path, &content, size, || {
                !cancellation.is_cancelled()
            })
            .map_err(|error| {
                if cancellation.is_cancelled() {
                    ExecutionError::cancelled("cancelled while publishing project artifact")
                } else {
                    ExecutionError::failed(error.to_string())
                }
            })?;
        artifacts.push((
            PortName::new(output.id().as_str()).map_err(contract_error)?,
            record.key,
        ));
    }
    ArtifactOutputs::try_from_iter(artifacts).map_err(contract_error)
}

fn descriptor(
    action: &ProjectAction,
    task: &ContentDigest,
    output: &ProjectOutput,
    semantics: &ProjectOutputSemantics,
    index: usize,
    dependencies: &[ArtifactDependency],
) -> Result<ArtifactDescriptor, ExecutionError> {
    Ok(ArtifactDescriptor::new(
        ProducerFingerprint {
            name: format!("veac-project-runtime:{}", action.kind_name()),
            version: super::super::action::PROJECT_ACTION_VERSION.to_string(),
            configuration: task.clone(),
        },
        dependencies.to_vec(),
        parameters(action, output, semantics, index)?,
    ))
}

fn parameters(
    action: &ProjectAction,
    output: &ProjectOutput,
    semantics: &ProjectOutputSemantics,
    index: usize,
) -> Result<ArtifactParameters, ExecutionError> {
    let value = RenderOutputParameters::new(index, output.id().as_str());
    match (output, semantics) {
        (
            ProjectOutput::Directory { .. },
            ProjectOutputSemantics::Evidence {
                suite_sha256,
                report_sha256,
                outcome,
            },
        ) if matches!(action, ProjectAction::Evidence { .. }) => {
            Ok(ArtifactParameters::EvidenceBundle(
                veac_artifact::EvidenceBundleParameters::new(suite_sha256, report_sha256, *outcome),
            ))
        }
        (
            ProjectOutput::Media {
                media_type: MediaType::Video,
                ..
            },
            ProjectOutputSemantics::Opaque,
        ) => Ok(ArtifactParameters::VideoMaster(
            ProducedArtifactParameters::Render(value),
        )),
        (
            ProjectOutput::Media {
                media_type: MediaType::Audio,
                ..
            },
            ProjectOutputSemantics::Opaque,
        ) => Ok(ArtifactParameters::AudioFile(value)),
        (
            ProjectOutput::Media {
                media_type: MediaType::Image,
                ..
            },
            ProjectOutputSemantics::Opaque,
        ) => Ok(ArtifactParameters::StillImage(value)),
        (ProjectOutput::Data { .. }, ProjectOutputSemantics::Opaque) => {
            Ok(ArtifactParameters::CaptionSidecar(value))
        }
        (ProjectOutput::Directory { .. }, ProjectOutputSemantics::Opaque) => {
            Ok(ArtifactParameters::AdaptivePackage(value))
        }
        _ => Err(ExecutionError::failed(
            "backend output semantics do not match the project action and output",
        )),
    }
}

fn dependencies(request: &ExecuteRequest<'_, ProjectAction>) -> Vec<ArtifactDependency> {
    let mut values = request
        .inputs
        .iter()
        .map(|input| ArtifactDependency::new(ArtifactDependencyRole::Input, input.digest.clone()))
        .collect::<Vec<_>>();
    values.push(ArtifactDependency::new(
        ArtifactDependencyRole::Task,
        request.cache_key.digest().clone(),
    ));
    values.sort_by(|left, right| (&left.role, &left.identity).cmp(&(&right.role, &right.identity)));
    values.dedup();
    values
}

fn contract_error(error: impl std::fmt::Display) -> ExecutionError {
    ExecutionError::failed(error.to_string())
}

#[cfg(test)]
#[path = "output/tests.rs"]
mod tests;
