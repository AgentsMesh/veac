use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;
use veac_plan::canonical::{
    HorizontalTextAlignment, TextLayout, TextOverflow, VerticalTextAlignment,
};

use super::error::TextError;
use super::fonts::FontBook;
use super::layout::{self, ShapedLine};
use super::model::{LinePiece, RenderLine, StyledText};

pub(super) fn compose(
    styled: &StyledText,
    shaped: &[ShapedLine],
    layout: TextLayout,
    surface: (u32, u32),
    origin: (f64, f64),
    fonts: &mut FontBook,
) -> Result<Vec<RenderLine>, TextError> {
    let box_width = layout.box_width_pixels.unwrap_or(f64::from(surface.0));
    let box_height = layout.box_height_pixels.unwrap_or(f64::from(surface.1));
    let mut selected = shaped.len();
    let mut vertical_overflow = false;
    if layout.overflow == TextOverflow::Ellipsis {
        selected = shaped
            .iter()
            .take_while(|line| line.top < box_height)
            .count()
            .max(1)
            .min(shaped.len());
        vertical_overflow = selected < shaped.len();
    }
    let mut lines = Vec::with_capacity(selected);
    for (index, line) in shaped.iter().take(selected).enumerate() {
        let mut pieces = pieces(styled, line.range.clone());
        let ellipsis = layout.overflow == TextOverflow::Ellipsis
            && (line.width > box_width || vertical_overflow && index + 1 == selected);
        let width = if ellipsis {
            ellipsize(&mut pieces, box_width, fonts)?
        } else {
            line.width
        };
        lines.push(RenderLine {
            pieces,
            width,
            height: line.height,
            x: 0.0,
            y: line.top,
        });
    }
    align(&mut lines, layout, box_width, box_height, origin);
    Ok(lines)
}

fn pieces(styled: &StyledText, range: Range<usize>) -> Vec<LinePiece> {
    let mut pieces = Vec::new();
    for run in &styled.runs {
        let start = run.range.start.max(range.start);
        let end = run.range.end.min(range.end);
        if start < end {
            pieces.push(LinePiece {
                text: styled.text[start..end].to_owned(),
                style: styled.styles[run.style].clone(),
            });
        }
    }
    if pieces.is_empty() {
        pieces.push(LinePiece {
            text: String::new(),
            style: styled.styles[0].clone(),
        });
    }
    pieces
}

fn ellipsize(
    pieces: &mut Vec<LinePiece>,
    width: f64,
    fonts: &mut FontBook,
) -> Result<f64, TextError> {
    trim_end(pieces);
    let mut ellipsis_style = pieces
        .last()
        .map(|piece| piece.style.clone())
        .ok_or_else(|| TextError::invalid("ellipsis has no text style"))?;
    if !fonts.covers(ellipsis_style.font, "…") {
        let font = fonts.first_covering("…").ok_or_else(|| {
            TextError::new(
                "TEXT_GLYPH_MISSING",
                "authored font stack has no ellipsis glyph",
            )
        })?;
        ellipsis_style.font = font;
        ellipsis_style.font_name = fonts.entries[font].ass_name.clone();
        ellipsis_style.font_alias = fonts.entries[font].alias.clone();
    }
    pieces.push(LinePiece {
        text: "…".to_owned(),
        style: ellipsis_style,
    });
    while layout::measure(pieces, fonts) > width && visible_graphemes(pieces) > 1 {
        let ellipsis = pieces.pop().expect("ellipsis exists");
        pop_grapheme(pieces);
        pieces.push(ellipsis);
    }
    Ok(layout::measure(pieces, fonts))
}

fn trim_end(pieces: &mut Vec<LinePiece>) {
    loop {
        let Some(last) = pieces.last_mut() else {
            return;
        };
        let trimmed = last.text.trim_end_matches(char::is_whitespace).len();
        last.text.truncate(trimmed);
        if !last.text.is_empty() || pieces.len() == 1 {
            return;
        }
        pieces.pop();
    }
}

fn pop_grapheme(pieces: &mut Vec<LinePiece>) {
    while let Some(last) = pieces.last_mut() {
        if let Some((start, _)) = last.text.grapheme_indices(true).next_back() {
            last.text.truncate(start);
            if last.text.is_empty() && pieces.len() > 1 {
                pieces.pop();
            }
            return;
        }
        if pieces.len() == 1 {
            return;
        }
        pieces.pop();
    }
}

fn visible_graphemes(pieces: &[LinePiece]) -> usize {
    pieces
        .iter()
        .map(|piece| piece.text.graphemes(true).count())
        .sum()
}

fn align(
    lines: &mut [RenderLine],
    layout: TextLayout,
    width: f64,
    height: f64,
    origin: (f64, f64),
) {
    let block_height = lines.last().map_or(0.0, |line| line.y + line.height);
    let vertical = match layout.vertical_alignment {
        VerticalTextAlignment::Top => 0.0,
        VerticalTextAlignment::Middle => (height - block_height) / 2.0,
        VerticalTextAlignment::Bottom => height - block_height,
    };
    for line in lines {
        let horizontal = match layout.horizontal_alignment {
            HorizontalTextAlignment::Left => 0.0,
            HorizontalTextAlignment::Center => (width - line.width) / 2.0,
            HorizontalTextAlignment::Right => width - line.width,
        };
        line.x = origin.0 + horizontal;
        line.y += origin.1 + vertical;
    }
}
