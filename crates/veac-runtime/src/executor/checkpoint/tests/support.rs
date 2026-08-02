use std::path::Path;

use veac_artifact::{artifact_key, ArtifactRecord, ContentDigest};
use veac_codegen::emitter::{
    BackendAction, BackendCommand, BackendOutput, BackendPhase, BackendProduct, BackendTask,
};

use super::super::{identity, manifest::CheckpointOutput, TaskIdentity};
use crate::executor::FfmpegFingerprint;

pub(super) fn digest(value: u8) -> ContentDigest {
    ContentDigest::sha256([value])
}

pub(super) fn fingerprint() -> FfmpegFingerprint {
    FfmpegFingerprint {
        version: "ffmpeg test".to_owned(),
        configuration: digest(9),
    }
}

pub(super) fn task(output: BackendOutput, product: BackendProduct) -> BackendTask {
    let path = match &output {
        BackendOutput::File(path) => path.clone(),
        BackendOutput::Files { paths } => paths[0].clone(),
        BackendOutput::ImageSequence { pattern } => pattern.clone(),
        BackendOutput::Package { paths, .. } => paths.playlist_pattern.clone(),
    };
    BackendTask {
        deliverable_id: veac_ir::DeliverableId::new("dlv_checkpoint_test").unwrap(),
        phase: BackendPhase::Single,
        product,
        output,
        action: BackendAction::Ffmpeg(BackendCommand {
            preparations: vec![],
            inputs: vec![],
            filter_graph: None,
            filter_contract: None,
            maps: vec![],
            output_args: vec!["-f".to_owned(), "data".to_owned()],
            output_path: path,
        }),
    }
}

pub(super) fn write_task(path: &Path, content: &[u8]) -> BackendTask {
    BackendTask {
        deliverable_id: veac_ir::DeliverableId::new("dlv_checkpoint_write").unwrap(),
        phase: BackendPhase::Single,
        product: BackendProduct::CaptionSidecar,
        output: BackendOutput::File(path.to_path_buf()),
        action: BackendAction::WriteFile {
            path: path.to_path_buf(),
            content: content.to_vec(),
        },
    }
}

pub(super) fn identity_for(task: &BackendTask) -> TaskIdentity {
    let ffmpeg = matches!(task.action, BackendAction::Ffmpeg(_)).then(fingerprint);
    super::super::identity(task, &digest(1), &digest(2), ffmpeg.as_ref(), None).unwrap()
}

pub(super) fn entry(path: &str) -> CheckpointOutput {
    let task = write_task(Path::new(path), b"caption");
    let identity = identity_for(&task);
    let descriptor = identity::output(&identity, BackendProduct::CaptionSidecar, path, 0);
    CheckpointOutput::File {
        path: path.to_owned(),
        record: ArtifactRecord {
            key: artifact_key(&descriptor).unwrap(),
            content: digest(3),
            size_bytes: 7,
        },
        descriptor,
    }
}

pub(super) fn deadline() -> std::time::Instant {
    std::time::Instant::now() + std::time::Duration::from_secs(10)
}
