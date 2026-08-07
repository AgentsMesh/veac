use serde_json::{json, Value};
use veac_artifact::{
    artifact_key, ArtifactDependency, ArtifactDependencyRole, ArtifactDescriptor,
    ArtifactParameters, ContentDigest, ProducerFingerprint, RenderCheckpointParameters,
    RenderOutputParameters, RenderTaskParameters,
};
use veac_codegen::emitter::{
    BackendAction, BackendCommand, BackendOutput, BackendPreparation, BackendProduct, BackendTask,
};

use crate::executor::{output, process, FfmpegFingerprint};
use crate::RuntimeError;

mod filter;
mod typed;

pub(in crate::executor) struct TaskIdentity {
    pub descriptor: ArtifactDescriptor,
    pub key: ContentDigest,
}

pub(super) fn task(
    value: &BackendTask,
    plan: &ContentDigest,
    resources: &ContentDigest,
    ffmpeg: Option<&FfmpegFingerprint>,
    predecessor: Option<&ContentDigest>,
) -> Result<TaskIdentity, RuntimeError> {
    let producer = producer(value, ffmpeg)?;
    let mut dependencies = vec![ArtifactDependency::new(
        ArtifactDependencyRole::Plan,
        plan.clone(),
    )];
    if let Some(identity) = predecessor {
        dependencies.push(ArtifactDependency::new(
            ArtifactDependencyRole::PreviousPass,
            identity.clone(),
        ));
    }
    dependencies.push(ArtifactDependency::new(
        ArtifactDependencyRole::Resources,
        resources.clone(),
    ));
    let descriptor = ArtifactDescriptor::new(
        producer,
        dependencies,
        ArtifactParameters::RenderCheckpoint(RenderCheckpointParameters::Task(
            RenderTaskParameters {
                contract_version: 2,
                deliverable_id: value.deliverable_id.clone(),
                phase: typed::phase(value.phase),
                product: typed::product(value.product),
                task_digest: digest(task_value(value)?)?,
            },
        )),
    );
    let key = artifact_key(&descriptor).map_err(artifact_error)?;
    Ok(TaskIdentity { descriptor, key })
}

pub(super) fn output(
    task: &TaskIdentity,
    product: BackendProduct,
    path: &str,
    index: usize,
) -> ArtifactDescriptor {
    let value = RenderOutputParameters::new(index, path);
    ArtifactDescriptor::new(
        task.descriptor.producer.clone(),
        vec![ArtifactDependency::new(
            ArtifactDependencyRole::Task,
            task.key.clone(),
        )],
        typed::output(product, value),
    )
}

fn producer(
    task: &BackendTask,
    ffmpeg: Option<&FfmpegFingerprint>,
) -> Result<ProducerFingerprint, RuntimeError> {
    match &task.action {
        BackendAction::Ffmpeg(_) => {
            let value = ffmpeg.ok_or_else(|| {
                RuntimeError::new("FFmpeg task is missing its producer fingerprint")
            })?;
            crate::workflow::media_artifact_producer(value)
        }
        BackendAction::WriteFile { .. } => Ok(ProducerFingerprint {
            name: "veac-runtime".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            configuration: crate::artifact_backend_identity(),
        }),
    }
}

fn task_value(task: &BackendTask) -> Result<Value, RuntimeError> {
    let action = match &task.action {
        BackendAction::Ffmpeg(command) => ffmpeg_value(command),
        BackendAction::WriteFile { path, content } => json!({
            "type": "write_file",
            "path": output::path_string(path),
            "content": ContentDigest::sha256(content),
            "size": content.len(),
        }),
    };
    let declared = match &task.output {
        BackendOutput::File(path) => json!({
            "type": "file",
            "path": output::path_string(path),
        }),
        BackendOutput::Files { paths } => json!({
            "type": "files",
            "paths": paths.iter().map(|path| output::path_string(path)).collect::<Vec<_>>(),
        }),
        BackendOutput::ImageSequence { pattern } => json!({
            "type": "image_sequence",
            "pattern": output::path_string(pattern),
        }),
        BackendOutput::Package {
            root,
            entrypoint,
            paths,
        } => json!({
            "type": "package",
            "root": output::path_string(root),
            "entrypoint": output::path_string(entrypoint),
            "playlist_pattern": output::path_string(&paths.playlist_pattern),
            "segment_pattern": output::path_string(&paths.segment_pattern),
        }),
    };
    Ok(json!({"action": action, "output": declared}))
}

fn ffmpeg_value(command: &BackendCommand) -> Value {
    json!({
        "type": "ffmpeg",
        "arguments": process::arguments(command),
        "filter_contract": filter::value(command),
        "preparations": command.preparations.iter().map(preparation_value).collect::<Vec<_>>(),
    })
}

fn preparation_value(value: &BackendPreparation) -> Value {
    json!({
        "command": ffmpeg_value(&value.command),
        "outputs": value.outputs.iter().map(|path| output::path_string(path)).collect::<Vec<_>>(),
    })
}

fn digest(value: Value) -> Result<ContentDigest, RuntimeError> {
    serde_json_canonicalizer::to_vec(&value)
        .map(ContentDigest::sha256)
        .map_err(|error| RuntimeError::new(format!("cannot hash backend task: {error}")))
}

fn artifact_error(error: veac_artifact::ArtifactError) -> RuntimeError {
    RuntimeError::new(format!("cannot identify render checkpoint: {error}"))
}
