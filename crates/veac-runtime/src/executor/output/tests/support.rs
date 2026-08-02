use std::path::{Path, PathBuf};

use veac_codegen::emitter::{
    BackendAction, BackendCommand, BackendOutput, BackendPhase, BackendProduct, BackendTask,
};

pub(super) fn task(output: BackendOutput, phase: BackendPhase) -> BackendTask {
    let path = match &output {
        BackendOutput::File(path) => path.clone(),
        BackendOutput::Files { paths } => paths[0].clone(),
        BackendOutput::ImageSequence { pattern } => pattern.clone(),
        BackendOutput::Package { paths, .. } => paths.playlist_pattern.clone(),
    };
    BackendTask {
        deliverable_id: veac_ir::DeliverableId::new("dlv_output_test").unwrap(),
        phase,
        product: if matches!(output, BackendOutput::ImageSequence { .. }) {
            BackendProduct::ImageSequence
        } else {
            BackendProduct::VideoMaster
        },
        output,
        action: BackendAction::Ffmpeg(command(&path)),
    }
}

pub(super) fn command(path: &Path) -> BackendCommand {
    BackendCommand {
        preparations: vec![],
        inputs: vec![],
        filter_graph: None,
        filter_contract: None,
        maps: vec![],
        output_args: vec![],
        output_path: path.to_path_buf(),
    }
}

pub(super) fn appended(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_owned();
    value.push(suffix);
    value.into()
}
