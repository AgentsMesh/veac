use std::collections::HashMap;

use subtitler::model::{AssStyle, Subtitle};

use crate::{
    time::range_to_millis, CaptionCue, CaptionEnvelope, CaptionError, CaptionFormat, ExportResult,
    LossReport,
};

use super::{losses, markup};

pub(super) fn export(
    value: &CaptionEnvelope,
    format: CaptionFormat,
) -> Result<ExportResult, CaptionError> {
    let mut report = LossReport::default();
    losses::document(&value.document, format, &mut report);
    let mut subtitles = Vec::with_capacity(value.document.cues.len());
    for (index, cue) in value.document.cues.iter().enumerate() {
        losses::cue(cue, format, index, &mut report);
        subtitles.push(to_subtitle(cue, format, index, &mut report)?);
    }
    let content = match format {
        CaptionFormat::Srt => subtitler::srt::to_string(&subtitles),
        CaptionFormat::WebVtt => {
            let header = value
                .document
                .settings
                .get("webvtt.header")
                .map(String::as_str);
            subtitler::vtt::to_string(&subtitles, header)
        }
        CaptionFormat::Ass => {
            let styles: Vec<_> = value.document.styles.iter().map(to_ass_style).collect();
            let defaults = [AssStyle::default_style()];
            let styles = if styles.is_empty() {
                &defaults[..]
            } else {
                &styles
            };
            subtitler::ass::to_string(&HashMap::new(), styles, &subtitles)
        }
    };
    Ok(ExportResult {
        content,
        loss_report: report,
    })
}

fn to_subtitle(
    cue: &CaptionCue,
    format: CaptionFormat,
    index: usize,
    report: &mut LossReport,
) -> Result<Subtitle, CaptionError> {
    let (start, end) = range_to_millis(cue.range)?;
    if format == CaptionFormat::Ass && (start % 10 != 0 || end % 10 != 0) {
        return Err(CaptionError::time(
            "ASS timestamps must be exact centiseconds",
        ));
    }
    let mut subtitle = Subtitle::new(start, end, &markup::render(cue, format, report));
    subtitle.index = Some(index + 1);
    match format {
        CaptionFormat::Srt => {}
        CaptionFormat::WebVtt => {
            subtitle.settings = cue.settings.get("webvtt.settings").cloned();
        }
        CaptionFormat::Ass => {
            subtitle.style = cue.style.clone();
            subtitle.actor = cue.speaker.clone();
            subtitle.is_comment = cue
                .settings
                .get("ass.comment")
                .is_some_and(|value| value == "true");
        }
    }
    Ok(subtitle)
}

fn to_ass_style(value: &crate::CaptionStyle) -> AssStyle {
    AssStyle {
        name: value.id.clone(),
        fontname: value.font_family.clone(),
        fontsize: value.font_size_pixels,
        primary_color: value.foreground_color.clone(),
        secondary_color: value.secondary_color.clone(),
        outline_color: value.outline_color.clone(),
        back_color: value.background_color.clone(),
        bold: value.bold,
        italic: value.italic,
        underline: value.underline,
        strikeout: value.strikeout,
        scale_x: value.scale_x_percent,
        scale_y: value.scale_y_percent,
        spacing: value.letter_spacing_pixels,
        angle: value.rotation_degrees,
        border_style: value.border_style,
        outline: value.outline_pixels,
        shadow: value.shadow_pixels,
        alignment: value.alignment,
        margin_l: value.margin_left,
        margin_r: value.margin_right,
        margin_v: value.margin_vertical,
        encoding: value.encoding,
    }
}
