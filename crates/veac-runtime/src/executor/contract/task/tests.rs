use std::path::{Path, PathBuf};

use veac_codegen::emitter::{
    BackendAction, BackendCommand, BackendOutput, BackendPackagePaths, BackendPhase,
    BackendProduct, BackendTask,
};

use super::{package, validate};

#[test]
fn task_shape_rejects_incompatible_phase_output_and_action_pairs() {
    let mut task = video_task("declared.mp4");
    task.phase = BackendPhase::FirstPass;
    rejects(task, "phase and product");

    let mut task = video_task("declared.mp4");
    task.product = BackendProduct::ImageSequence;
    rejects(task, "requires a sequence output");

    let mut task = video_task("declared.mp4");
    task.output = BackendOutput::Files { paths: vec![] };
    rejects(task, "file list may not be empty");

    let mut task = video_task("declared.mp4");
    task.product = BackendProduct::HlsVod;
    rejects(task, "requires one typed package");

    let mut task = video_task("declared.mp4");
    task.action = BackendAction::WriteFile {
        path: "declared.mp4".into(),
        content: vec![],
    };
    rejects(task, "write-file task");
}

#[test]
fn ffmpeg_outputs_must_match_their_typed_declarations() {
    let mut sequence = video_task("wrong.png");
    sequence.product = BackendProduct::ImageSequence;
    sequence.output = BackendOutput::ImageSequence {
        pattern: "frame-%04d.png".into(),
    };
    rejects(sequence, "image output path");

    let mut pass = video_task("pass.null");
    pass.phase = BackendPhase::FirstPass;
    pass.product = BackendProduct::RenderPassLog;
    pass.output = BackendOutput::File("wrong.log".into());
    command_mut(&mut pass).output_args = vec!["-passlogfile".into(), "logs/render".into()];
    rejects(pass, "first-pass output");

    let mut files = video_task("primary.bin");
    files.output = BackendOutput::Files {
        paths: vec!["primary.bin".into(), "sidecar.bin".into()],
    };
    validate(&files).unwrap();

    let mut mismatch = video_task("actual.mp4");
    mismatch.output = BackendOutput::File("declared.mp4".into());
    rejects(mismatch, "does not match");

    let mut invalid_pattern = video_task("frame.png");
    invalid_pattern.product = BackendProduct::ImageSequence;
    invalid_pattern.output = BackendOutput::ImageSequence {
        pattern: "frame.png".into(),
    };
    rejects(invalid_pattern, "exactly one");
}

#[test]
fn hls_layout_rejects_unsafe_paths_and_inexact_options() {
    let mut command = hls_command();
    command.output_path = "wrong.m3u8".into();
    assert!(package::validate(&command, Path::new("master.m3u8"), &hls_paths()).is_err());

    let command = hls_command();
    assert!(package::validate(&command, Path::new("nested/master.m3u8"), &hls_paths()).is_err());

    let mut command = hls_command();
    command.output_args.drain(0..2);
    assert!(package::validate(&command, Path::new("master.m3u8"), &hls_paths()).is_err());

    let mut command = hls_command();
    command.output_args.drain(2..4);
    assert!(package::validate(&command, Path::new("master.m3u8"), &hls_paths()).is_err());
}

fn video_task(output: &str) -> BackendTask {
    BackendTask {
        deliverable_id: veac_ir::DeliverableId::new("dlv_task_boundary").unwrap(),
        phase: BackendPhase::Single,
        product: BackendProduct::VideoMaster,
        output: BackendOutput::File(output.into()),
        action: BackendAction::Ffmpeg(command(output)),
    }
}

fn command(output: &str) -> BackendCommand {
    BackendCommand {
        preparations: vec![],
        inputs: vec![],
        filter_graph: None,
        filter_contract: None,
        maps: vec![],
        output_args: vec![],
        output_path: output.into(),
    }
}

fn command_mut(task: &mut BackendTask) -> &mut BackendCommand {
    let BackendAction::Ffmpeg(command) = &mut task.action else {
        unreachable!()
    };
    command
}

fn hls_paths() -> BackendPackagePaths {
    BackendPackagePaths {
        playlist_pattern: PathBuf::from("media-%v.m3u8"),
        segment_pattern: PathBuf::from("segment-%v-%03d.ts"),
    }
}

fn hls_command() -> BackendCommand {
    let mut command = command("media-%v.m3u8");
    command.output_args = vec![
        "-master_pl_name".into(),
        "master.m3u8".into(),
        "-hls_segment_filename".into(),
        "segment-%v-%03d.ts".into(),
        "-f".into(),
        "hls".into(),
        "-hls_playlist_type".into(),
        "vod".into(),
        "-hls_list_size".into(),
        "0".into(),
    ];
    command
}

fn rejects(task: BackendTask, expected: &str) {
    assert!(validate(&task).unwrap_err().message.contains(expected));
}
