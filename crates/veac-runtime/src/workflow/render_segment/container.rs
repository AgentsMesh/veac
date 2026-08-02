use veac_ir::{MediaProbeSnapshot, OutputFormat};

use super::media_error;
use crate::workflow::WorkflowResult;

mod ebml;

pub(super) fn validate_snapshot(
    probe: &MediaProbeSnapshot,
    expected: OutputFormat,
) -> WorkflowResult<()> {
    let family = |name: &str| probe.container_format.split(',').any(|value| value == name);
    let valid = match expected {
        OutputFormat::Mp4 => {
            family("mov")
                && family("mp4")
                && probe
                    .container_brand
                    .as_deref()
                    .is_some_and(|brand| !brand.trim().eq_ignore_ascii_case("qt"))
        }
        OutputFormat::Mov => {
            family("mov")
                && probe
                    .container_brand
                    .as_deref()
                    .is_some_and(|brand| brand.trim().eq_ignore_ascii_case("qt"))
        }
        OutputFormat::Mkv | OutputFormat::Webm => family("matroska") && family("webm"),
        OutputFormat::Mxf => family("mxf"),
    };
    if !valid {
        return Err(media_error(
            "render-segment container differs from its exact delivery profile",
        ));
    }
    Ok(())
}

pub(super) fn validate_prefix(prefix: &[u8], expected: OutputFormat) -> WorkflowResult<()> {
    match expected {
        OutputFormat::Mkv => ebml::validate(prefix, ebml::DocType::Matroska),
        OutputFormat::Webm => ebml::validate(prefix, ebml::DocType::Webm),
        _ => Ok(()),
    }
}
