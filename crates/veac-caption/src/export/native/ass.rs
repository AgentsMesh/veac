use std::{collections::HashMap, fmt::Write};

use subtitler::model::{AssStyle, Subtitle};
use veac_ir::{AssCueSettings, CaptionNativeCue};

use crate::{AssScriptInfo, AssScriptType, AssYcbcrMatrix, CaptionDocument, CaptionDocumentNative};

use super::ass_timestamp;

pub(crate) fn render(
    document: &CaptionDocument,
    styles: &[AssStyle],
    subtitles: &[Subtitle],
) -> String {
    let shell = subtitler::ass::to_string(&HashMap::new(), styles, &[]);
    let styles = shell
        .split_once("[V4+ Styles]\n")
        .and_then(|(_, rest)| rest.split_once("\n[Events]\n"))
        .map_or("", |(styles, _)| styles);
    let mut output = script_info(document);
    let _ = write!(output, "\n[V4+ Styles]\n{styles}\n[Events]\n");
    output.push_str(
        "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
    );
    for (subtitle, cue) in subtitles.iter().zip(&document.cues) {
        let native = match &cue.native {
            Some(CaptionNativeCue::Ass { settings }) => settings,
            _ => &EMPTY_SETTINGS,
        };
        let _ = writeln!(
            output,
            "{}: {},{},{},{},{},{},{},{},{},{}",
            if native.comment {
                "Comment"
            } else {
                "Dialogue"
            },
            native.layer.unwrap_or(0),
            ass_timestamp(subtitle.start),
            ass_timestamp(subtitle.end),
            subtitle.style.as_deref().unwrap_or("Default"),
            subtitle.actor.as_deref().unwrap_or(""),
            native.margin_left.unwrap_or(0),
            native.margin_right.unwrap_or(0),
            native.margin_vertical.unwrap_or(0),
            native.effect.as_deref().unwrap_or(""),
            subtitle.text,
        );
    }
    output
}

fn script_info(document: &CaptionDocument) -> String {
    let mut output = "[Script Info]\n".to_owned();
    let Some(CaptionDocumentNative::Ass { info }) = &document.native else {
        output.push_str(
            "Title: <untitled>\nScriptType: v4.00+\nPlayResX: 384\nPlayResY: 288\nWrapStyle: 0\n",
        );
        return output;
    };
    write_info(&mut output, info);
    output
}

fn write_info(output: &mut String, info: &AssScriptInfo) {
    field(output, "Title", info.title.as_deref());
    field(
        output,
        "ScriptType",
        Some(info.script_type.map_or("v4.00+", |value| match value {
            AssScriptType::V4Plus => "v4.00+",
        })),
    );
    number(output, "WrapStyle", info.wrap_style);
    field(
        output,
        "ScaledBorderAndShadow",
        info.scaled_border_and_shadow
            .map(|value| if value { "yes" } else { "no" }),
    );
    number(output, "PlayResX", info.play_res_x);
    number(output, "PlayResY", info.play_res_y);
    field(output, "YCbCr Matrix", info.ycbcr_matrix.map(matrix));
}

fn matrix(value: AssYcbcrMatrix) -> &'static str {
    match value {
        AssYcbcrMatrix::None => "None",
        AssYcbcrMatrix::Tv601 => "TV.601",
        AssYcbcrMatrix::Pc601 => "PC.601",
        AssYcbcrMatrix::Tv709 => "TV.709",
        AssYcbcrMatrix::Pc709 => "PC.709",
        AssYcbcrMatrix::Tv240m => "TV.240M",
        AssYcbcrMatrix::Pc240m => "PC.240M",
        AssYcbcrMatrix::TvFcc => "TV.FCC",
        AssYcbcrMatrix::PcFcc => "PC.FCC",
    }
}

fn field(output: &mut String, name: &str, value: Option<&str>) {
    if let Some(value) = value {
        let _ = writeln!(output, "{name}: {value}");
    }
}

fn number(output: &mut String, name: &str, value: Option<impl std::fmt::Display>) {
    if let Some(value) = value {
        let _ = writeln!(output, "{name}: {value}");
    }
}

const EMPTY_SETTINGS: AssCueSettings = AssCueSettings {
    comment: false,
    layer: None,
    margin_left: None,
    margin_right: None,
    margin_vertical: None,
    effect: None,
};
