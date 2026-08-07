use veac_ir::CaptionNativeCue;

use crate::{CaptionCue, CaptionDocument, CaptionDocumentNative, CaptionFormat, LossReport};

pub(super) fn document(document: &CaptionDocument, format: CaptionFormat, report: &mut LossReport) {
    if document.language.is_some() {
        report.document("language", "target format has no document language field");
    }
    if format != CaptionFormat::Ass && !document.styles.is_empty() {
        report.document("styles", "target format has no reusable style definitions");
    }
    match &document.native {
        Some(CaptionDocumentNative::WebVtt { header }) if format != CaptionFormat::WebVtt => {
            if header.description.is_some() {
                report.document(
                    "native.webvtt.header.description",
                    "target format has no WebVTT header description",
                );
            }
        }
        Some(CaptionDocumentNative::Ass { info }) if format != CaptionFormat::Ass => {
            ass_info(info, report);
        }
        _ => {}
    }
}

pub(super) fn cue(cue: &CaptionCue, format: CaptionFormat, report: &mut LossReport) {
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
    match &cue.native {
        Some(CaptionNativeCue::Srt { .. }) if format != CaptionFormat::Srt => report.cue(
            &cue.id,
            "native.srt.index",
            "target format has no SRT cue index",
        ),
        Some(CaptionNativeCue::WebVtt {
            identifier,
            settings,
        }) if format != CaptionFormat::WebVtt => {
            if identifier.is_some() {
                report.cue(
                    &cue.id,
                    "native.webvtt.identifier",
                    "target format has no WebVTT cue identifier",
                );
            }
            if settings.is_some() {
                report.cue(
                    &cue.id,
                    "native.webvtt.settings",
                    "target format has no WebVTT cue settings",
                );
            }
        }
        Some(CaptionNativeCue::Ass { settings }) if format != CaptionFormat::Ass => {
            ass_cue(cue, settings, report);
        }
        _ => {}
    }
}

fn ass_info(info: &crate::AssScriptInfo, report: &mut LossReport) {
    let values = [
        ("title", info.title.is_some()),
        ("script_type", info.script_type.is_some()),
        ("wrap_style", info.wrap_style.is_some()),
        (
            "scaled_border_and_shadow",
            info.scaled_border_and_shadow.is_some(),
        ),
        ("play_res_x", info.play_res_x.is_some()),
        ("play_res_y", info.play_res_y.is_some()),
        ("ycbcr_matrix", info.ycbcr_matrix.is_some()),
    ];
    for (field, present) in values {
        if present {
            report.document(
                &format!("native.ass.info.{field}"),
                "target format has no ASS Script Info field",
            );
        }
    }
}

fn ass_cue(cue: &CaptionCue, value: &crate::AssCueSettings, report: &mut LossReport) {
    let values = [
        ("comment", value.comment),
        ("layer", value.layer.is_some()),
        ("margin_left", value.margin_left.is_some()),
        ("margin_right", value.margin_right.is_some()),
        ("margin_vertical", value.margin_vertical.is_some()),
        ("effect", value.effect.is_some()),
    ];
    for (field, present) in values {
        if present {
            report.cue(
                &cue.id,
                &format!("native.ass.{field}"),
                "target format has no ASS event field",
            );
        }
    }
}
