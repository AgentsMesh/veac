use veac_ir::{CaptionNativeCue, WebVttCueSettings};

use crate::{CaptionDocumentNative, ValidationIssue};

use super::{issue, optional_text};

pub(super) fn document(value: Option<&CaptionDocumentNative>, issues: &mut Vec<ValidationIssue>) {
    match value {
        Some(CaptionDocumentNative::WebVtt { header }) => {
            let path = "$.document.native.header.description";
            optional_text(issues, path, header.description.as_deref());
            if header
                .description
                .as_deref()
                .is_some_and(|value| !line(value, 1024))
            {
                issue(
                    issues,
                    path,
                    "NATIVE_TEXT",
                    "must be a single safe WebVTT header line",
                );
            }
        }
        Some(CaptionDocumentNative::Ass { info }) => {
            let path = "$.document.native.info";
            optional_text(issues, &format!("{path}.title"), info.title.as_deref());
            if info
                .title
                .as_deref()
                .is_some_and(|value| !line(value, 1024))
            {
                issue(
                    issues,
                    &format!("{path}.title"),
                    "NATIVE_TEXT",
                    "must be a single safe ASS Script Info line",
                );
            }
            if info.wrap_style.is_some_and(|value| value > 3) {
                issue(
                    issues,
                    &format!("{path}.wrap_style"),
                    "NATIVE_RANGE",
                    "must be in 0..=3",
                );
            }
            if info.play_res_x == Some(0) || info.play_res_y == Some(0) {
                issue(
                    issues,
                    path,
                    "NATIVE_RANGE",
                    "ASS play resolution must be greater than zero",
                );
            }
        }
        None => {}
    }
}

pub(super) fn cue(value: Option<&CaptionNativeCue>, path: &str, issues: &mut Vec<ValidationIssue>) {
    match value {
        Some(CaptionNativeCue::Srt { index: 0 }) => {
            issue(issues, path, "NATIVE_RANGE", "SRT index must be positive");
        }
        Some(CaptionNativeCue::WebVtt {
            identifier,
            settings,
        }) => {
            if identifier
                .as_ref()
                .is_some_and(|value| !line(&value.0, 1024) || value.0.contains("-->"))
            {
                issue(
                    issues,
                    path,
                    "NATIVE_TEXT",
                    "WebVTT identifier must be a safe cue identifier",
                );
            }
            if settings.as_ref().is_some_and(|value| !webvtt(value)) {
                issue(
                    issues,
                    path,
                    "NATIVE_SETTING",
                    "WebVTT cue setting contains an invalid token",
                );
            }
        }
        Some(CaptionNativeCue::Ass { settings }) => {
            if settings.effect.as_deref().is_some_and(|value| {
                !line(value, 1024) || value.contains(',') || value.trim().is_empty()
            }) {
                issue(
                    issues,
                    path,
                    "NATIVE_TEXT",
                    "ASS effect must be a safe nonempty event field",
                );
            }
        }
        _ => {}
    }
}

pub(super) fn compatible(
    document: Option<&CaptionDocumentNative>,
    cue: Option<&CaptionNativeCue>,
) -> bool {
    matches!(
        (document, cue),
        (
            Some(CaptionDocumentNative::WebVtt { .. }),
            Some(CaptionNativeCue::WebVtt { .. }) | None
        ) | (
            Some(CaptionDocumentNative::Ass { .. }),
            Some(CaptionNativeCue::Ass { .. }) | None
        ) | (None, _)
    )
}

fn webvtt(value: &WebVttCueSettings) -> bool {
    [
        value.line.as_deref(),
        value.position.as_deref(),
        value.size.as_deref(),
        value.region.as_ref().map(|value| value.0.as_str()),
    ]
    .into_iter()
    .flatten()
    .all(token)
}

fn token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 1024
        && !value
            .chars()
            .any(|ch| ch.is_control() || ch.is_whitespace())
}

fn line(value: &str, limit: usize) -> bool {
    !value.is_empty() && value.len() <= limit && !value.chars().any(char::is_control)
}
