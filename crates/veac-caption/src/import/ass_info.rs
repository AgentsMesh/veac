use std::collections::HashMap;

use crate::{AssScriptInfo, AssScriptType, AssYcbcrMatrix, CaptionError, CaptionFormat};

pub(super) fn parse(
    values: &HashMap<String, String>,
) -> Result<(AssScriptInfo, Vec<String>), CaptionError> {
    let mut info = AssScriptInfo::default();
    let mut unknown = Vec::new();
    for (key, value) in values {
        match key.to_ascii_lowercase().as_str() {
            "title" => set(&mut info.title, text(value, "Title")?, "Title")?,
            "scripttype" => set(&mut info.script_type, script_type(value)?, "ScriptType")?,
            "wrapstyle" => set(&mut info.wrap_style, wrap_style(value)?, "WrapStyle")?,
            "scaledborderandshadow" => set(
                &mut info.scaled_border_and_shadow,
                boolean(value, "ScaledBorderAndShadow")?,
                "ScaledBorderAndShadow",
            )?,
            "playresx" => set(
                &mut info.play_res_x,
                positive(value, "PlayResX")?,
                "PlayResX",
            )?,
            "playresy" => set(
                &mut info.play_res_y,
                positive(value, "PlayResY")?,
                "PlayResY",
            )?,
            "ycbcr matrix" | "ycbcrmatrix" => {
                set(&mut info.ycbcr_matrix, matrix(value)?, "YCbCr Matrix")?
            }
            _ => unknown.push(key.clone()),
        }
    }
    unknown.sort();
    Ok((info, unknown))
}

fn set<T>(slot: &mut Option<T>, value: T, field: &str) -> Result<(), CaptionError> {
    if slot.replace(value).is_some() {
        Err(invalid(field, "is duplicated"))
    } else {
        Ok(())
    }
}

fn text(value: &str, field: &str) -> Result<String, CaptionError> {
    let value = value.trim();
    if value.is_empty() {
        Err(invalid(field, "must be nonempty"))
    } else {
        Ok(value.to_owned())
    }
}

fn script_type(value: &str) -> Result<AssScriptType, CaptionError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "v4.00+" => Ok(AssScriptType::V4Plus),
        _ => Err(invalid("ScriptType", "must be v4.00+")),
    }
}

fn wrap_style(value: &str) -> Result<u8, CaptionError> {
    let value = value
        .trim()
        .parse::<u8>()
        .map_err(|_| invalid("WrapStyle", "must be an integer"))?;
    if value <= 3 {
        Ok(value)
    } else {
        Err(invalid("WrapStyle", "must be in 0..=3"))
    }
}

fn boolean(value: &str, field: &str) -> Result<bool, CaptionError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "yes" | "true" => Ok(true),
        "no" | "false" => Ok(false),
        _ => Err(invalid(field, "must be yes or no")),
    }
}

fn positive(value: &str, field: &str) -> Result<u32, CaptionError> {
    let value = value
        .trim()
        .parse::<u32>()
        .map_err(|_| invalid(field, "must be a positive integer"))?;
    if value == 0 {
        Err(invalid(field, "must be greater than zero"))
    } else {
        Ok(value)
    }
}

fn matrix(value: &str) -> Result<AssYcbcrMatrix, CaptionError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Ok(AssYcbcrMatrix::None),
        "tv.601" => Ok(AssYcbcrMatrix::Tv601),
        "pc.601" => Ok(AssYcbcrMatrix::Pc601),
        "tv.709" => Ok(AssYcbcrMatrix::Tv709),
        "pc.709" => Ok(AssYcbcrMatrix::Pc709),
        "tv.240m" => Ok(AssYcbcrMatrix::Tv240m),
        "pc.240m" => Ok(AssYcbcrMatrix::Pc240m),
        "tv.fcc" => Ok(AssYcbcrMatrix::TvFcc),
        "pc.fcc" => Ok(AssYcbcrMatrix::PcFcc),
        _ => Err(invalid("YCbCr Matrix", "has an unsupported value")),
    }
}

fn invalid(field: &str, reason: &str) -> CaptionError {
    CaptionError::parse(CaptionFormat::Ass, format!("ASS {field} {reason}"))
}
