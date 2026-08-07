use veac_ir::AssCueSettings;

use crate::{CaptionError, CaptionFormat};

pub(crate) fn vtt_ids(input: &str) -> Vec<Option<String>> {
    let normalized = input.replace("\r\n", "\n");
    normalized
        .split("\n\n")
        .filter_map(|block| {
            let mut lines = block.lines().map(str::trim).filter(|line| !line.is_empty());
            let first = lines.next()?;
            if first.starts_with("WEBVTT")
                || first.starts_with("NOTE")
                || first.starts_with("STYLE")
                || first.starts_with("REGION")
            {
                return None;
            }
            if first.contains("-->") {
                Some(None)
            } else {
                lines
                    .next()
                    .filter(|line| line.contains("-->"))
                    .map(|_| Some(first.to_owned()))
            }
        })
        .collect()
}

pub(crate) fn ass_extras(input: &str) -> Result<Vec<AssCueSettings>, CaptionError> {
    input
        .lines()
        .filter_map(|line| {
            let value = line
                .trim()
                .strip_prefix("Dialogue:")
                .or_else(|| line.trim().strip_prefix("Comment:"))?
                .trim();
            let fields: Vec<_> = value.splitn(10, ',').collect();
            Some(parse_ass_fields(&fields))
        })
        .collect()
}

fn parse_ass_fields(fields: &[&str]) -> Result<AssCueSettings, CaptionError> {
    if fields.len() != 10 {
        return Err(ass_error("event does not contain ten fields"));
    }
    Ok(AssCueSettings {
        comment: false,
        layer: nonzero_u32(fields[0], "layer")?,
        margin_left: nonzero_u32(fields[5], "left margin")?,
        margin_right: nonzero_u32(fields[6], "right margin")?,
        margin_vertical: nonzero_u32(fields[7], "vertical margin")?,
        effect: nonempty(fields[8]),
    })
}

fn nonzero_u32(value: &str, field: &str) -> Result<Option<u32>, CaptionError> {
    let value = value
        .trim()
        .parse::<u32>()
        .map_err(|_| ass_error(&format!("invalid {field}")))?;
    Ok((value != 0).then_some(value))
}

fn nonempty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn ass_error(message: &str) -> CaptionError {
    CaptionError::parse(CaptionFormat::Ass, message)
}

pub(crate) fn has_unsupported_vtt_markup(input: &str) -> bool {
    ["<c.", "<ruby", "<rt", "<lang", "<00:"]
        .iter()
        .any(|tag| input.contains(tag))
}

pub(crate) fn has_unsupported_ass_override(text: &str) -> bool {
    text.split('{')
        .skip(1)
        .filter_map(|part| part.split_once('}').map(|value| value.0))
        .any(|block| {
            block.split('\\').filter(|tag| !tag.is_empty()).any(|tag| {
                !["b", "i", "u", "c", "r"]
                    .iter()
                    .any(|prefix| tag.starts_with(prefix))
            })
        })
}
