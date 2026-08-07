use veac_ir::{WebVttCueSettings, WebVttRegionId, WebVttTextAlign, WebVttVertical};

use crate::CaptionError;

pub(crate) fn parse(value: &str) -> Result<WebVttCueSettings, CaptionError> {
    let mut output = WebVttCueSettings {
        line: None,
        position: None,
        size: None,
        align: None,
        vertical: None,
        region: None,
    };
    for token in value.split_ascii_whitespace() {
        let (name, value) = token
            .split_once(':')
            .ok_or_else(|| invalid("setting must use name:value syntax"))?;
        match name {
            "line" if output.line.is_none() => output.line = Some(value.to_owned()),
            "position" if output.position.is_none() => output.position = Some(value.to_owned()),
            "size" if output.size.is_none() => output.size = Some(value.to_owned()),
            "align" if output.align.is_none() => output.align = Some(align(value)?),
            "vertical" if output.vertical.is_none() => output.vertical = Some(vertical(value)?),
            "region" if output.region.is_none() => {
                output.region = Some(WebVttRegionId(value.to_owned()))
            }
            "line" | "position" | "size" | "align" | "vertical" | "region" => {
                return Err(invalid("setting name is duplicated"))
            }
            _ => {
                return Err(invalid(
                    "setting name is outside the closed WebVTT contract",
                ))
            }
        }
    }
    Ok(output)
}

pub(crate) fn format(value: &WebVttCueSettings) -> String {
    let mut output = Vec::new();
    push(&mut output, "line", value.line.as_deref());
    push(&mut output, "position", value.position.as_deref());
    push(&mut output, "size", value.size.as_deref());
    push(&mut output, "align", value.align.map(align_name));
    push(&mut output, "vertical", value.vertical.map(vertical_name));
    push(
        &mut output,
        "region",
        value.region.as_ref().map(|region| region.0.as_str()),
    );
    output.join(" ")
}

fn align(value: &str) -> Result<WebVttTextAlign, CaptionError> {
    match value {
        "start" => Ok(WebVttTextAlign::Start),
        "center" | "middle" => Ok(WebVttTextAlign::Center),
        "end" => Ok(WebVttTextAlign::End),
        "left" => Ok(WebVttTextAlign::Left),
        "right" => Ok(WebVttTextAlign::Right),
        _ => Err(invalid("align value is invalid")),
    }
}

fn vertical(value: &str) -> Result<WebVttVertical, CaptionError> {
    match value {
        "rl" => Ok(WebVttVertical::Rl),
        "lr" => Ok(WebVttVertical::Lr),
        _ => Err(invalid("vertical value is invalid")),
    }
}

fn align_name(value: WebVttTextAlign) -> &'static str {
    match value {
        WebVttTextAlign::Start => "start",
        WebVttTextAlign::Center => "center",
        WebVttTextAlign::End => "end",
        WebVttTextAlign::Left => "left",
        WebVttTextAlign::Right => "right",
    }
}

fn vertical_name(value: WebVttVertical) -> &'static str {
    match value {
        WebVttVertical::Rl => "rl",
        WebVttVertical::Lr => "lr",
    }
}

fn push(output: &mut Vec<String>, name: &str, value: Option<&str>) {
    if let Some(value) = value {
        output.push(format!("{name}:{value}"));
    }
}

fn invalid(message: &str) -> CaptionError {
    CaptionError::ir(format!("invalid canonical WebVTT cue settings: {message}"))
}
