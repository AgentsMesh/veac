use subtitler::model::{AssStyle, SubtitleFile};
use veac_ir::CaptionNativeCue;

use crate::{
    validate, CaptionDocumentNative, CaptionEnvelope, CaptionError, CaptionFormat, CaptionStyle,
    ImportOptions, ImportResult,
};

use super::{ass_info, common, native};

pub(super) fn import_ass(
    input: &str,
    options: &ImportOptions,
) -> Result<ImportResult, CaptionError> {
    let file = subtitler::ass::parse_content(input)
        .map_err(|error| CaptionError::parse(CaptionFormat::Ass, error))?;
    let SubtitleFile::Ass(data) = file else {
        return Err(CaptionError::parse(
            CaptionFormat::Ass,
            "parser returned another format",
        ));
    };
    let markers = input
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            line.starts_with("Dialogue:") || line.starts_with("Comment:")
        })
        .count();
    if markers == 0 || markers != data.subtitles.len() {
        return Err(CaptionError::parse(
            CaptionFormat::Ass,
            "one or more event records are malformed",
        ));
    }
    let extras = native::ass_extras(input)?;
    let styles = data.styles.iter().map(convert_style).collect();
    let (info, unknown_info) = ass_info::parse(&data.info)?;
    let mut result = common::build(
        &data.subtitles,
        styles,
        Some(CaptionDocumentNative::Ass { info }),
        CaptionFormat::Ass,
        options,
        |index, subtitle| {
            let mut settings = extras.get(index).cloned().unwrap_or_default();
            if subtitle.is_comment {
                settings.comment = true;
            }
            Ok((None, Some(CaptionNativeCue::Ass { settings })))
        },
    )?;
    for key in unknown_info {
        result.loss_report.document(
            &format!("native.ass.info.{key}"),
            "ASS Script Info field is outside the closed contract",
        );
    }
    for (index, subtitle) in data.subtitles.iter().enumerate() {
        if native::has_unsupported_ass_override(&subtitle.text) {
            result.loss_report.cue(
                &result.document.cues[index].id,
                "text.spans",
                "ASS override contains an unsupported command",
            );
        }
    }
    validate(&CaptionEnvelope::new(result.document.clone()))?;
    Ok(result)
}

fn convert_style(value: &AssStyle) -> CaptionStyle {
    CaptionStyle {
        id: value.name.clone(),
        font_family: value.fontname.clone(),
        font_size_pixels: value.fontsize,
        foreground_color: value.primary_color.clone(),
        secondary_color: value.secondary_color.clone(),
        outline_color: value.outline_color.clone(),
        background_color: value.back_color.clone(),
        bold: value.bold,
        italic: value.italic,
        underline: value.underline,
        strikeout: value.strikeout,
        scale_x_percent: value.scale_x,
        scale_y_percent: value.scale_y,
        letter_spacing_pixels: value.spacing,
        rotation_degrees: value.angle,
        border_style: value.border_style,
        outline_pixels: value.outline,
        shadow_pixels: value.shadow,
        alignment: value.alignment,
        margin_left: value.margin_l,
        margin_right: value.margin_r,
        margin_vertical: value.margin_v,
        encoding: value.encoding,
    }
}
