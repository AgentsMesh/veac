use std::collections::BTreeSet;

use unicode_segmentation::UnicodeSegmentation;
use veac_plan::{ResolvedText, ResolvedTextSpan, ResolvedTextStyle};

use super::error::TextError;
use super::fonts::FontBook;
use super::model::{GlyphStyle, StyledRun, StyledText};

pub(super) fn resolve(
    content: &ResolvedText,
    style: &ResolvedTextStyle,
    fonts: &FontBook,
) -> Result<StyledText, TextError> {
    validate_span_boundaries(content, style)?;
    let mut styles = Vec::new();
    let mut runs: Vec<StyledRun> = Vec::new();
    let mut scalar = 0_u32;
    for (byte, grapheme) in content.text.grapheme_indices(true) {
        let span = style
            .spans
            .iter()
            .find(|span| span.start <= scalar && scalar < span.end);
        let glyph = glyph_style(style, span, grapheme, fonts)?;
        let style_index = styles
            .iter()
            .position(|candidate| candidate == &glyph)
            .unwrap_or_else(|| {
                styles.push(glyph);
                styles.len() - 1
            });
        let end = byte + grapheme.len();
        if let Some(previous) = runs.last_mut().filter(|run| run.style == style_index) {
            previous.range.end = end;
        } else {
            runs.push(StyledRun {
                range: byte..end,
                style: style_index,
            });
        }
        scalar += grapheme.chars().count() as u32;
    }
    if content.text.is_empty() {
        styles.push(glyph_style(style, None, "", fonts)?);
    }
    Ok(StyledText {
        text: content.text.clone(),
        styles,
        runs,
    })
}

fn glyph_style(
    style: &ResolvedTextStyle,
    span: Option<&ResolvedTextSpan>,
    grapheme: &str,
    fonts: &FontBook,
) -> Result<GlyphStyle, TextError> {
    let primary = span
        .and_then(|span| span.font.as_ref())
        .unwrap_or(&style.font);
    let candidates = std::iter::once(primary).chain(&style.fallback_fonts);
    let font = candidates
        .map(|font| fonts.index(font))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .find(|index| fonts.covers(*index, grapheme))
        .ok_or_else(|| {
            let scalars = grapheme
                .chars()
                .map(|value| format!("U+{:04X}", u32::from(value)))
                .collect::<Vec<_>>()
                .join(" ");
            TextError::new(
                "TEXT_GLYPH_MISSING",
                format!("authored font stack has no glyph for {scalars}"),
            )
        })?;
    let entry = &fonts.entries[font];
    Ok(GlyphStyle {
        font,
        font_name: entry.ass_name.clone(),
        font_alias: entry.alias.clone(),
        weight: span
            .and_then(|span| span.font_weight)
            .unwrap_or(style.font_weight),
        font_style: span
            .and_then(|span| span.font_style)
            .unwrap_or(style.font_style),
        size: span
            .and_then(|span| span.size_pixels)
            .unwrap_or(style.size_pixels),
        color: span.and_then(|span| span.color).unwrap_or(style.color),
        tracking: style.tracking_pixels,
        line_height: style.line_height,
    })
}

fn validate_span_boundaries(
    content: &ResolvedText,
    style: &ResolvedTextStyle,
) -> Result<(), TextError> {
    let mut boundaries = BTreeSet::from([0_u32]);
    let mut scalar = 0_u32;
    for grapheme in content.text.graphemes(true) {
        scalar += grapheme.chars().count() as u32;
        boundaries.insert(scalar);
    }
    if let Some(span) = style
        .spans
        .iter()
        .find(|span| !boundaries.contains(&span.start) || !boundaries.contains(&span.end))
    {
        return Err(TextError::new(
            "TEXT_SPAN_GRAPHEME_SPLIT",
            format!(
                "rich span {}..{} splits an extended grapheme cluster",
                span.start, span.end
            ),
        ));
    }
    Ok(())
}
