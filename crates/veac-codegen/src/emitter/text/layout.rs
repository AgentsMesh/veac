use std::ops::Range;

use cosmic_text::fontdb::{Style, Weight};
use cosmic_text::{Attrs, Buffer, Family, LineIter, Metrics, Shaping, Wrap};
use veac_plan::canonical::{FontStyle, FontWeight, TextWrap};

use super::fonts::FontBook;
use super::model::{GlyphStyle, LinePiece, StyledRun, StyledText};

#[derive(Debug, Clone)]
pub(super) struct ShapedLine {
    pub range: Range<usize>,
    pub width: f64,
    pub top: f64,
    pub height: f64,
}

pub(super) fn shape(
    styled: &StyledText,
    fonts: &mut FontBook,
    width: Option<f64>,
    wrap: TextWrap,
) -> Vec<ShapedLine> {
    let mut buffer = buffer(styled, fonts, width, wrap);
    let paragraphs = paragraph_ranges(&styled.text);
    let raw: Vec<_> = buffer
        .layout_runs()
        .map(|run| {
            let start = run
                .glyphs
                .iter()
                .map(|glyph| glyph.start)
                .min()
                .unwrap_or(0);
            let end = run
                .glyphs
                .iter()
                .map(|glyph| glyph.end)
                .max()
                .unwrap_or(start);
            (
                run.line_i,
                start,
                end,
                run.line_w,
                run.line_top,
                run.line_height,
            )
        })
        .collect();
    let mut lines = Vec::with_capacity(raw.len());
    for (index, value) in raw.iter().enumerate() {
        let paragraph = paragraphs
            .get(value.0)
            .cloned()
            .unwrap_or(styled.text.len()..styled.text.len());
        let local_start = if index == 0 || raw[index - 1].0 != value.0 {
            0
        } else {
            value.1
        };
        let local_end = raw
            .get(index + 1)
            .filter(|next| next.0 == value.0)
            .map_or(paragraph.len(), |next| next.1);
        lines.push(ShapedLine {
            range: paragraph.start + local_start..paragraph.start + local_end,
            width: f64::from(value.3),
            top: f64::from(value.4),
            height: f64::from(value.5),
        });
    }
    buffer.set_redraw(false);
    lines
}

pub(super) fn measure(pieces: &[LinePiece], fonts: &mut FontBook) -> f64 {
    let styled = from_pieces(pieces);
    buffer(&styled, fonts, None, TextWrap::None)
        .layout_runs()
        .next()
        .map_or(0.0, |run| f64::from(run.line_w))
}

fn buffer(styled: &StyledText, fonts: &mut FontBook, width: Option<f64>, wrap: TextWrap) -> Buffer {
    let base = styled
        .styles
        .first()
        .expect("styled text always has a style");
    let default = attrs(base, 0);
    let spans: Vec<_> = styled
        .runs
        .iter()
        .map(|run| {
            (
                &styled.text[run.range.clone()],
                attrs(&styled.styles[run.style], run.style),
            )
        })
        .collect();
    let metrics = Metrics::new(base.size as f32, (base.line_height * base.size) as f32);
    let mut buffer = Buffer::new_empty(metrics);
    buffer.set_size(&mut fonts.system, width.map(|value| value as f32), None);
    buffer.set_wrap(&mut fonts.system, wrap_mode(wrap));
    if spans.is_empty() {
        buffer.set_text(&mut fonts.system, "", &default, Shaping::Advanced, None);
    } else {
        buffer.set_rich_text(&mut fonts.system, spans, &default, Shaping::Advanced, None);
    }
    buffer
}

fn attrs(style: &GlyphStyle, metadata: usize) -> Attrs<'_> {
    Attrs::new()
        .family(Family::Name(&style.font_alias))
        .weight(Weight(weight(style.weight)))
        .style(font_style(style.font_style))
        .metrics(Metrics::new(
            style.size as f32,
            (style.size * style.line_height) as f32,
        ))
        .letter_spacing((style.tracking / style.size) as f32)
        .metadata(metadata)
}

fn from_pieces(pieces: &[LinePiece]) -> StyledText {
    let mut text = String::new();
    let mut styles = Vec::new();
    let mut runs = Vec::new();
    for piece in pieces {
        let start = text.len();
        text.push_str(&piece.text);
        let style = styles
            .iter()
            .position(|value| value == &piece.style)
            .unwrap_or_else(|| {
                styles.push(piece.style.clone());
                styles.len() - 1
            });
        runs.push(StyledRun {
            range: start..text.len(),
            style,
        });
    }
    StyledText { text, styles, runs }
}

fn paragraph_ranges(text: &str) -> Vec<Range<usize>> {
    let mut ranges: Vec<_> = LineIter::new(text).map(|(range, _)| range).collect();
    if ranges.is_empty() || text.ends_with(['\r', '\n']) {
        ranges.push(text.len()..text.len());
    }
    ranges
}

fn wrap_mode(value: TextWrap) -> Wrap {
    match value {
        TextWrap::None => Wrap::None,
        TextWrap::Word => Wrap::Word,
        TextWrap::Character => Wrap::Glyph,
    }
}

fn weight(value: FontWeight) -> u16 {
    match value {
        FontWeight::Thin => 100,
        FontWeight::ExtraLight => 200,
        FontWeight::Light => 300,
        FontWeight::Normal => 400,
        FontWeight::Medium => 500,
        FontWeight::SemiBold => 600,
        FontWeight::Bold => 700,
        FontWeight::ExtraBold => 800,
        FontWeight::Black => 900,
    }
}

fn font_style(value: FontStyle) -> Style {
    match value {
        FontStyle::Normal => Style::Normal,
        FontStyle::Italic => Style::Italic,
        FontStyle::Oblique => Style::Oblique,
    }
}
