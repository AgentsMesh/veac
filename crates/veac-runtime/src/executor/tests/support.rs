use std::path::{Path, PathBuf};

use veac_artifact::ContentDigest;
use veac_codegen::emitter::{
    BackendAction, BackendCommand, BackendOutput, BackendPhase, BackendProduct, BackendResource,
    BackendTask,
};
use veac_ir::DeliverableId;

use crate::executor::model::RuntimeBundle as BackendBundle;

#[path = "support/fake_ffmpeg.rs"]
mod fake_ffmpeg;
pub(in crate::executor) use fake_ffmpeg::FakeFfmpeg;

pub(in crate::executor) fn video_task(id: &str, path: &Path) -> BackendTask {
    BackendTask {
        deliverable_id: deliverable_id(id),
        phase: BackendPhase::Single,
        product: BackendProduct::VideoMaster,
        output: BackendOutput::File(path.to_path_buf()),
        action: BackendAction::Ffmpeg(command(path)),
    }
}

pub(super) fn write_task(id: &str, path: &Path, content: &[u8]) -> BackendTask {
    BackendTask {
        deliverable_id: deliverable_id(id),
        phase: BackendPhase::Single,
        product: BackendProduct::CaptionSidecar,
        output: BackendOutput::File(path.to_path_buf()),
        action: BackendAction::WriteFile {
            path: path.to_path_buf(),
            content: content.to_vec(),
        },
    }
}

pub(super) fn image_task(id: &str, pattern: &Path) -> BackendTask {
    BackendTask {
        deliverable_id: deliverable_id(id),
        phase: BackendPhase::Single,
        product: BackendProduct::ImageSequence,
        output: BackendOutput::ImageSequence {
            pattern: pattern.to_path_buf(),
        },
        action: BackendAction::Ffmpeg(command(pattern)),
    }
}

pub(in crate::executor) fn two_pass_tasks(id: &str, output: &Path) -> Vec<BackendTask> {
    let prefix = appended(output, ".veac-pass");
    let log = appended(&prefix, "-0.log");
    let mut first = command(&appended(output, ".pass1.null"));
    first.output_args.extend([
        "-pass".to_owned(),
        "1".to_owned(),
        "-passlogfile".to_owned(),
        prefix.to_string_lossy().into_owned(),
        "-f".to_owned(),
        "null".to_owned(),
    ]);
    let mut second = command(output);
    second.output_args.extend([
        "-pass".to_owned(),
        "2".to_owned(),
        "-passlogfile".to_owned(),
        prefix.to_string_lossy().into_owned(),
    ]);
    vec![
        BackendTask {
            deliverable_id: deliverable_id(id),
            phase: BackendPhase::FirstPass,
            product: BackendProduct::RenderPassLog,
            output: BackendOutput::File(log),
            action: BackendAction::Ffmpeg(first),
        },
        BackendTask {
            deliverable_id: deliverable_id(id),
            phase: BackendPhase::SecondPass,
            product: BackendProduct::VideoMaster,
            output: BackendOutput::File(output.to_path_buf()),
            action: BackendAction::Ffmpeg(second),
        },
    ]
}

pub(in crate::executor) fn command(path: &Path) -> BackendCommand {
    BackendCommand {
        inputs: vec![],
        filter_graph: None,
        filter_contract: None,
        maps: vec![],
        output_args: vec!["-f".to_owned(), "data".to_owned()],
        output_path: path.to_path_buf(),
    }
}

pub(in crate::executor) fn bundle(tasks: Vec<BackendTask>) -> BackendBundle {
    BackendBundle {
        plan_identity: ContentDigest::sha256(b"test-plan"),
        substitution_proof: ContentDigest::sha256(b"test-substitution"),
        protected_resources: vec![],
        requirements: vec![],
        tasks,
    }
}

pub(in crate::executor) fn protected_resource(path: &Path) -> BackendResource {
    BackendResource {
        path: path.to_path_buf(),
        expected_identity: crate::asset::sha256_identity(path).unwrap(),
    }
}

pub(super) fn path(parent: &Path, name: &str) -> PathBuf {
    parent.join(name)
}

pub(super) fn deliverable_id(value: &str) -> DeliverableId {
    DeliverableId::new(format!("dlv_{value}")).unwrap()
}

fn appended(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_owned();
    value.push(suffix);
    value.into()
}
