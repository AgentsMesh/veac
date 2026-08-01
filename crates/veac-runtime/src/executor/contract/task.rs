use std::path::Path;

use veac_codegen::emitter::{
    BackendAction, BackendOutput, BackendPhase, BackendProduct, BackendTask,
};
use veac_ir::ImageSequencePattern;

use super::invalid;
use crate::executor::output;
use crate::RuntimeError;

mod package;

pub(super) fn validate(task: &BackendTask) -> Result<(), RuntimeError> {
    match (task.phase, task.product) {
        (BackendPhase::FirstPass, BackendProduct::RenderPassLog)
        | (BackendPhase::Single | BackendPhase::SecondPass, BackendProduct::VideoMaster)
        | (BackendPhase::Single, BackendProduct::ImageSequence)
        | (BackendPhase::Single, BackendProduct::CaptionSidecar)
        | (BackendPhase::Single, BackendProduct::AudioStem)
        | (BackendPhase::Single, BackendProduct::AudioFile)
        | (BackendPhase::Single, BackendProduct::AnimatedImage)
        | (BackendPhase::Single, BackendProduct::StillImage)
        | (BackendPhase::Single, BackendProduct::HlsVod)
        | (BackendPhase::Single, BackendProduct::VideoWaveform)
        | (BackendPhase::Single, BackendProduct::Vectorscope)
        | (BackendPhase::Single, BackendProduct::Histogram) => {}
        _ => return invalid("backend task phase and product are incompatible"),
    }
    match (&task.output, task.product) {
        (BackendOutput::ImageSequence { pattern }, BackendProduct::ImageSequence) => {
            validate_pattern(pattern)?
        }
        (BackendOutput::ImageSequence { .. }, _) | (_, BackendProduct::ImageSequence) => {
            return invalid("image sequence product requires a sequence output pattern")
        }
        (BackendOutput::Files { paths }, _) if paths.is_empty() => {
            return invalid("backend file list may not be empty")
        }
        (BackendOutput::Package { .. }, BackendProduct::HlsVod) => {}
        (BackendOutput::Package { .. }, _) | (_, BackendProduct::HlsVod) => {
            return invalid("HLS product requires one typed package output")
        }
        _ => {}
    }
    validate_action(task)
}

fn validate_action(task: &BackendTask) -> Result<(), RuntimeError> {
    match (&task.action, &task.output) {
        (BackendAction::WriteFile { path, .. }, BackendOutput::File(output))
            if path == output && task.product == BackendProduct::CaptionSidecar =>
        {
            Ok(())
        }
        (BackendAction::WriteFile { .. }, _) => {
            invalid("write-file task must target its caption sidecar output")
        }
        (BackendAction::Ffmpeg(command), BackendOutput::ImageSequence { pattern })
            if command.output_path == *pattern =>
        {
            Ok(())
        }
        (BackendAction::Ffmpeg(_), BackendOutput::ImageSequence { .. }) => {
            invalid("FFmpeg image output path must equal its declared pattern")
        }
        (BackendAction::Ffmpeg(_), BackendOutput::File(path))
            if task.phase == BackendPhase::FirstPass =>
        {
            let expected = output::appended(&output::passlog_prefix(task)?, "-0.log");
            if *path == expected {
                Ok(())
            } else {
                invalid("first-pass output must be the FFmpeg passlog")
            }
        }
        (BackendAction::Ffmpeg(command), BackendOutput::File(path))
            if command.output_path == *path =>
        {
            Ok(())
        }
        (BackendAction::Ffmpeg(command), BackendOutput::Files { paths })
            if paths.first() == Some(&command.output_path) =>
        {
            Ok(())
        }
        (
            BackendAction::Ffmpeg(command),
            BackendOutput::Package {
                entrypoint, paths, ..
            },
        ) => package::validate(command, entrypoint, paths),
        (BackendAction::Ffmpeg(_), _) => {
            invalid("FFmpeg command output does not match its declared output")
        }
    }
}

fn validate_pattern(pattern: &Path) -> Result<(), RuntimeError> {
    let name = pattern
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if ImageSequencePattern::parse(name).is_some() {
        Ok(())
    } else {
        invalid("image sequence output requires exactly one %d or %0Nd placeholder")
    }
}
