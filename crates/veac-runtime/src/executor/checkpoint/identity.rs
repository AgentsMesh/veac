use serde_json::{json, Value};
use veac_artifact::{
    artifact_key, ArtifactDependency, ArtifactDescriptor, ArtifactKind, ContentDigest,
    ProducerFingerprint,
};
use veac_codegen::emitter::{
    BackendAction, BackendCommand, BackendOutput, BackendPreparation, BackendProduct, BackendTask,
};

use crate::executor::{output, process, FfmpegFingerprint};
use crate::RuntimeError;

mod filter;

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
    let mut dependencies = vec![ArtifactDependency {
        role: "plan".to_owned(),
        identity: plan.clone(),
    }];
    if let Some(identity) = predecessor {
        dependencies.push(ArtifactDependency {
            role: "previous_pass".to_owned(),
            identity: identity.clone(),
        });
    }
    dependencies.push(ArtifactDependency {
        role: "resources".to_owned(),
        identity: resources.clone(),
    });
    let descriptor = ArtifactDescriptor::new(
        ArtifactKind::RenderCheckpoint,
        producer,
        dependencies,
        json!({
            "contract_version": 2,
            "deliverable_id": value.deliverable_id.as_str(),
            "phase": value.phase.as_str(),
            "product": value.product.as_str(),
            "task_digest": digest(task_value(value)?)?.value,
        }),
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
    ArtifactDescriptor::new(
        kind(product),
        task.descriptor.producer.clone(),
        vec![ArtifactDependency {
            role: "task".to_owned(),
            identity: task.key.clone(),
        }],
        json!({"index": index, "path": path}),
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
            Ok(ProducerFingerprint {
                name: "ffmpeg".to_owned(),
                version: value.version.clone(),
                configuration: value.configuration.clone(),
            })
        }
        BackendAction::WriteFile { .. } => Ok(ProducerFingerprint {
            name: "veac-runtime".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            configuration: ContentDigest::sha256(b"write-file-checkpoint-v1"),
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

fn kind(product: BackendProduct) -> ArtifactKind {
    match product {
        BackendProduct::VideoMaster => ArtifactKind::VideoMaster,
        BackendProduct::RenderPassLog => ArtifactKind::RenderCheckpoint,
        BackendProduct::ImageSequence => ArtifactKind::ImageSequenceFrame,
        BackendProduct::CaptionSidecar => ArtifactKind::CaptionSidecar,
        BackendProduct::AudioStem => ArtifactKind::AudioStem,
        BackendProduct::AudioFile => ArtifactKind::AudioFile,
        BackendProduct::AnimatedImage => ArtifactKind::AnimatedImage,
        BackendProduct::StillImage => ArtifactKind::StillImage,
        BackendProduct::HlsVod => ArtifactKind::AdaptivePackage,
        BackendProduct::VideoWaveform => ArtifactKind::VideoWaveform,
        BackendProduct::Vectorscope => ArtifactKind::Vectorscope,
        BackendProduct::Histogram => ArtifactKind::Histogram,
    }
}

fn artifact_error(error: veac_artifact::ArtifactError) -> RuntimeError {
    RuntimeError::new(format!("cannot identify render checkpoint: {error}"))
}
