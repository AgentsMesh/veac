use std::ffi::OsString;
use std::path::{Path, PathBuf};

use veac_artifact::ExecutionBindings;
use veac_plan::canonical::{Deliverable, PassMode, VideoDeliverable};
use veac_plan::ResolvedRenderPlan;

use super::super::{
    emit_video_delivery, BackendCommand, BackendOutput, BackendPhase, BackendProduct, BackendTask,
    CodegenErrors,
};

pub(super) fn emit(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    deliverable: &Deliverable,
    settings: &VideoDeliverable,
    tasks: &mut Vec<BackendTask>,
) -> Result<(), CodegenErrors> {
    let copies_segment = bindings.full_render_segment().is_some();
    if copies_segment {
        let command = super::super::render_segment::command(plan, bindings, deliverable, settings)?;
        tasks.push(task(
            deliverable,
            BackendPhase::Single,
            BackendProduct::VideoMaster,
            BackendOutput::File(command.output_path.clone()),
            command,
        ));
        return Ok(());
    }
    let command = emit_video_delivery(plan, bindings, deliverable, settings)?;
    let output = command.output_path.clone();
    match settings.pass_mode {
        PassMode::Single => tasks.push(task(
            deliverable,
            BackendPhase::Single,
            BackendProduct::VideoMaster,
            BackendOutput::File(output),
            command,
        )),
        PassMode::TwoPass => {
            let mut first_settings = settings.clone();
            first_settings.audio = None;
            let mut first_deliverable = deliverable.clone();
            first_deliverable.kind =
                veac_plan::canonical::DeliverableKind::Video(first_settings.clone());
            let first = emit_video_delivery(plan, bindings, &first_deliverable, &first_settings)?;
            two_pass(deliverable, first, command, tasks);
        }
    }
    Ok(())
}

fn two_pass(
    deliverable: &Deliverable,
    mut first: BackendCommand,
    command: BackendCommand,
    tasks: &mut Vec<BackendTask>,
) {
    let prefix = appended(&command.output_path, ".veac-pass");
    let log = appended(&prefix, "-0.log");
    first.output_args.extend(pass_arguments(1, &prefix));
    first
        .output_args
        .extend(["-an".to_owned(), "-f".to_owned(), "null".to_owned()]);
    first.output_path = appended(&command.output_path, ".pass1.null");
    tasks.push(task(
        deliverable,
        BackendPhase::FirstPass,
        BackendProduct::RenderPassLog,
        BackendOutput::File(log),
        first,
    ));

    let mut second = command;
    second.output_args.extend(pass_arguments(2, &prefix));
    let output = BackendOutput::File(second.output_path.clone());
    tasks.push(task(
        deliverable,
        BackendPhase::SecondPass,
        BackendProduct::VideoMaster,
        output,
        second,
    ));
}

fn task(
    deliverable: &Deliverable,
    phase: BackendPhase,
    product: BackendProduct,
    output: BackendOutput,
    command: BackendCommand,
) -> BackendTask {
    BackendTask {
        deliverable_id: deliverable.id.clone(),
        phase,
        product,
        output,
        action: super::super::BackendAction::Ffmpeg(command),
    }
}

fn pass_arguments(pass: u8, prefix: &Path) -> Vec<String> {
    vec![
        "-pass".to_owned(),
        pass.to_string(),
        "-passlogfile".to_owned(),
        prefix.to_string_lossy().into_owned(),
    ]
}

fn appended(path: &Path, suffix: &str) -> PathBuf {
    let mut value: OsString = path.as_os_str().to_owned();
    value.push(suffix);
    value.into()
}
