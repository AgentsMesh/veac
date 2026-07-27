use std::path::PathBuf;

use veac_artifact::ExecutionBindings;
use veac_plan::canonical::{AudioCodec, Deliverable, OutputFormat, VideoDeliverable};
use veac_plan::ResolvedOutput;

use super::error::{diagnostic, CodegenErrorKind};
use super::CodegenErrors;

pub(super) fn validate_binding(
    deliverable: &Deliverable,
    bindings: &ExecutionBindings,
) -> Result<(), CodegenErrors> {
    bound_path(deliverable, bindings).map(|_| ())
}

pub(super) fn bound_path(
    deliverable: &Deliverable,
    bindings: &ExecutionBindings,
) -> Result<PathBuf, CodegenErrors> {
    bindings
        .output(&deliverable.id)
        .map(PathBuf::from)
        .ok_or_else(|| {
            CodegenErrors::one(diagnostic(
                CodegenErrorKind::MissingOutputBinding,
                "OUTPUT_BINDING_MISSING",
                Some(deliverable.id.to_string()),
                "deliverable has no machine-local path binding",
            ))
        })
}

pub(super) fn arguments(output: &ResolvedOutput, settings: &VideoDeliverable) -> Vec<String> {
    let mut args = super::output_video::arguments(&settings.video, settings.hardware);
    args.extend([
        "-r".to_owned(),
        format!(
            "{}/{}",
            output.frame_rate.numerator, output.frame_rate.denominator
        ),
        "-s".to_owned(),
        format!("{}x{}", output.width, output.height),
        "-f".to_owned(),
        container(settings.container).to_owned(),
    ]);
    if settings.optimize_for_streaming {
        args.extend(["-movflags".to_owned(), "+faststart".to_owned()]);
    }
    if let Some(audio) = &settings.audio {
        args.extend([
            "-c:a".to_owned(),
            audio_encoder(audio.codec).to_owned(),
            "-ar".to_owned(),
            audio.sample_rate.to_string(),
            "-ac".to_owned(),
            audio.channels.to_string(),
        ]);
    } else {
        args.push("-an".to_owned());
    }
    args
}

fn container(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Mp4 => "mp4",
        OutputFormat::Mov => "mov",
        OutputFormat::Mkv => "matroska",
        OutputFormat::Webm => "webm",
        OutputFormat::Mxf => "mxf",
    }
}

pub(super) fn audio_encoder(codec: AudioCodec) -> &'static str {
    match codec {
        AudioCodec::Aac => "aac",
        AudioCodec::Opus => "libopus",
        AudioCodec::Flac => "flac",
        AudioCodec::PcmS16Le => "pcm_s16le",
        AudioCodec::PcmS24Le => "pcm_s24le",
        AudioCodec::PcmS32Le => "pcm_s32le",
    }
}
