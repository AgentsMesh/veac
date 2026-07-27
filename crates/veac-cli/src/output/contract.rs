use std::path::{Path, PathBuf};

use veac_ir::{AudioStemFormat, CaptionSidecarFormat, Deliverable, DeliverableKind, ImageFormat};

use crate::error::{CliError, CliResult};

pub(super) fn validate(
    deliverable: &Deliverable,
    candidate: &Path,
    protected: &[PathBuf],
) -> CliResult {
    validate_name(deliverable, candidate)?;
    if protected
        .iter()
        .any(|path| consumes(candidate, deliverable, path))
    {
        return Err(CliError::new(
            "OUTPUT_OVERWRITES_INPUT",
            format!("output {} aliases a project input", candidate.display()),
        ));
    }
    Ok(())
}

pub(super) fn validate_name(deliverable: &Deliverable, candidate: &Path) -> CliResult {
    let name = candidate
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let valid = match &deliverable.kind {
        DeliverableKind::Video(settings) => {
            veac_ir::output_file_compatible(name, settings.container)
        }
        DeliverableKind::ImageSequence(settings) => {
            veac_ir::ImageSequencePattern::parse(name).is_some()
                && image_extension(name, settings.format)
        }
        DeliverableKind::CaptionSidecar(settings) => extension(
            name,
            match settings.format {
                CaptionSidecarFormat::Srt => "srt",
                CaptionSidecarFormat::WebVtt => "vtt",
                CaptionSidecarFormat::Ass => "ass",
            },
        ),
        DeliverableKind::AudioStem(settings) => extension(
            name,
            match settings.format {
                AudioStemFormat::Wav => "wav",
                AudioStemFormat::Flac => "flac",
            },
        ),
        DeliverableKind::Scope(settings) => image_extension(name, settings.format),
    };
    if valid {
        Ok(())
    } else {
        Err(CliError::new(
            "OUTPUT_FORMAT_MISMATCH",
            format!(
                "output {} does not match deliverable {}",
                candidate.display(),
                deliverable.id
            ),
        ))
    }
}

fn consumes(candidate: &Path, deliverable: &Deliverable, protected: &Path) -> bool {
    if !matches!(deliverable.kind, DeliverableKind::ImageSequence(_)) {
        return candidate == protected;
    }
    if candidate.parent() != protected.parent() {
        return false;
    }
    let pattern = candidate.file_name().unwrap().to_string_lossy();
    let value = protected.file_name().unwrap().to_string_lossy();
    veac_ir::ImageSequencePattern::parse(&pattern).is_some_and(|pattern| pattern.matches(&value))
}

fn image_extension(value: &str, format: ImageFormat) -> bool {
    extension(
        value,
        match format {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Tiff => "tiff",
            ImageFormat::Exr => "exr",
        },
    )
}

fn extension(value: &str, expected: &str) -> bool {
    value
        .rsplit_once('.')
        .is_some_and(|(_, value)| value.eq_ignore_ascii_case(expected))
}
