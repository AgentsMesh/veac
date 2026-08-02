use crate::{CaptionCue, CaptionDocument, CaptionFormat, LossReport};

pub(super) fn document(document: &CaptionDocument, format: CaptionFormat, report: &mut LossReport) {
    if document.language.is_some() {
        report.document("language", "target format has no document language field");
    }
    if format != CaptionFormat::Ass && !document.styles.is_empty() {
        report.document("styles", "target format has no reusable style definitions");
    }
    for key in document.settings.keys() {
        let supported = format == CaptionFormat::WebVtt && key == "webvtt.header";
        if !supported {
            report.document(
                &format!("settings.{key}"),
                "document setting is not emitted by target format",
            );
        }
    }
}

pub(super) fn cue(
    cue: &CaptionCue,
    format: CaptionFormat,
    output_index: usize,
    report: &mut LossReport,
) {
    if !cue.words.is_empty() {
        report.cue(
            &cue.id,
            "words",
            "target format has no canonical word-timing field",
        );
    }
    if cue.speaker.is_some() && format == CaptionFormat::Srt {
        report.cue(&cue.id, "speaker", "SRT has no speaker field");
    }
    if cue.style.is_some() && format != CaptionFormat::Ass {
        report.cue(&cue.id, "style", "target format has no ASS-style reference");
    }
    for (key, value) in &cue.settings {
        let supported = match format {
            CaptionFormat::Srt => false,
            CaptionFormat::WebVtt => key == "webvtt.settings",
            CaptionFormat::Ass => key == "ass.comment" && (value == "true" || value == "false"),
        };
        if !supported {
            let reason = if key == "srt.index" && value == &(output_index + 1).to_string() {
                "SRT cue index is deterministically regenerated"
            } else {
                "cue setting is not emitted by target format"
            };
            report.cue(&cue.id, &format!("settings.{key}"), reason);
        }
    }
}
