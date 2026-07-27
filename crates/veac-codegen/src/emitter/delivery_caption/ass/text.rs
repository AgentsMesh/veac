use std::collections::BTreeSet;
use std::fmt::Write;

use unicode_segmentation::UnicodeSegmentation;
use veac_plan::canonical::FontStyle;
use veac_plan::{ResolvedText, ResolvedTextSpan};

use super::super::failure::Failure;
use super::font::FontCatalog;
use super::style::{ass_alpha, ass_rgb, weight, Style};
use crate::emitter::time;

pub(super) fn render(
    content: &ResolvedText,
    base: &Style,
    fonts: &mut FontCatalog<'_>,
) -> Result<String, Failure> {
    validate_spans(content)?;
    let characters: Vec<_> = content.text.chars().collect();
    let mut output = String::new();
    let mut cursor = 0_usize;
    for span in &content.style.spans {
        output.push_str(&escape(&characters[cursor..span.start as usize]));
        let tags = tags(span, fonts)?;
        let value = escape(&characters[span.start as usize..span.end as usize]);
        if tags.is_empty() {
            output.push_str(&value);
        } else {
            let _ = write!(output, "{{{tags}}}{value}{{{}}}", base.base_tags());
        }
        cursor = span.end as usize;
    }
    output.push_str(&escape(&characters[cursor..]));
    Ok(output)
}

pub(super) fn speaker(value: Option<&str>) -> Result<&str, Failure> {
    let value = value.unwrap_or("");
    if value
        .chars()
        .any(|item| matches!(item, ',' | '\r' | '\n' | '\0'))
    {
        Err(Failure::unsupported(
            "CAPTION_ASS_SPEAKER",
            "caption speaker cannot be represented safely in the ASS Name field",
        ))
    } else {
        Ok(value)
    }
}

fn tags(span: &ResolvedTextSpan, fonts: &mut FontCatalog<'_>) -> Result<String, Failure> {
    let mut output = String::new();
    if let Some(font) = &span.font {
        let _ = write!(output, "\\fn{}", fonts.name(font)?);
    }
    if let Some(value) = span.font_weight {
        let _ = write!(output, "\\b{}", weight(value));
    }
    if let Some(value) = span.font_style {
        if value == FontStyle::Oblique {
            return Err(Failure::unsupported(
                "CAPTION_ASS_SPAN_OBLIQUE",
                "ASS sidecars cannot preserve an oblique rich-text span",
            ));
        }
        let _ = write!(output, "\\i{}", u8::from(value == FontStyle::Italic));
    }
    if let Some(value) = span.size_pixels {
        if !value.is_finite() || value <= 0.0 {
            return Err(invalid("rich-text span size is invalid"));
        }
        let _ = write!(output, "\\fs{}", time::number(value));
    }
    if let Some(value) = span.color {
        let _ = write!(
            output,
            "\\1c{}\\1a{}",
            ass_rgb(value),
            ass_alpha(value, 1.0)
        );
    }
    Ok(output)
}

fn validate_spans(content: &ResolvedText) -> Result<(), Failure> {
    let length = content.text.chars().count() as u32;
    let mut previous_end = 0_u32;
    let mut grapheme_boundaries = BTreeSet::from([0_u32]);
    let mut scalar = 0_u32;
    for grapheme in content.text.graphemes(true) {
        scalar += grapheme.chars().count() as u32;
        grapheme_boundaries.insert(scalar);
    }
    for span in &content.style.spans {
        if span.start >= span.end || span.start < previous_end || span.end > length {
            return Err(invalid("rich-text span range is invalid or overlapping"));
        }
        if !grapheme_boundaries.contains(&span.start) || !grapheme_boundaries.contains(&span.end) {
            return Err(invalid(
                "rich-text span splits an extended grapheme cluster",
            ));
        }
        previous_end = span.end;
    }
    Ok(())
}

fn escape(value: &[char]) -> String {
    let mut output = String::new();
    for character in value {
        match *character {
            '\\' => output.push_str("\\\\"),
            '{' => output.push_str("\\{"),
            '}' => output.push_str("\\}"),
            '\n' => output.push_str("\\N"),
            '\r' => {}
            value => output.push(value),
        }
    }
    output
}

fn invalid(message: &str) -> Failure {
    Failure::invalid("CAPTION_ASS_SPAN_INVALID", message)
}
