use unicode_script::{Script, UnicodeScript};
use veac_plan::canonical::{
    HorizontalTextAlignment, TextLayout, TextOrientation, TextWritingMode, VerticalTextAlignment,
};

use super::{placed, MeasuredPiece};
use crate::emitter::text::model::PlacedPiece;

pub(super) fn place(
    columns: &[Vec<MeasuredPiece>],
    layout: TextLayout,
    mode: TextWritingMode,
    surface: (u32, u32),
    origin: (f64, f64),
) -> Vec<PlacedPiece> {
    let widths: Vec<_> = columns
        .iter()
        .map(|column| column.iter().map(|piece| piece.cross).fold(0.0, f64::max))
        .collect();
    let block_width: f64 = widths.iter().sum();
    let box_width = layout.box_width_pixels.unwrap_or(f64::from(surface.0));
    let box_height = layout.box_height_pixels.unwrap_or(f64::from(surface.1));
    let base_x = origin.0
        + match layout.horizontal_alignment {
            HorizontalTextAlignment::Left => 0.0,
            HorizontalTextAlignment::Center => (box_width - block_width) / 2.0,
            HorizontalTextAlignment::Right => box_width - block_width,
        };
    let lefts = column_lefts(&widths, mode, base_x, block_width);
    let mut output = Vec::new();
    for ((column, width), left) in columns.iter().zip(&widths).zip(lefts) {
        let content_height: f64 = column.iter().map(|piece| piece.advance).sum();
        let mut y = origin.1
            + match layout.vertical_alignment {
                VerticalTextAlignment::Top => 0.0,
                VerticalTextAlignment::Middle => (box_height - content_height) / 2.0,
                VerticalTextAlignment::Bottom => box_height - content_height,
            };
        for piece in column {
            let rotation = orientation(&piece.text, layout.orientation);
            output.push(placed(
                piece,
                left + width / 2.0,
                y + piece.advance / 2.0,
                rotation,
            ));
            y += piece.advance;
        }
    }
    output
}

fn column_lefts(widths: &[f64], mode: TextWritingMode, base: f64, total: f64) -> Vec<f64> {
    let mut output = Vec::with_capacity(widths.len());
    let mut cursor = if mode == TextWritingMode::VerticalRl {
        base + total
    } else {
        base
    };
    for width in widths {
        if mode == TextWritingMode::VerticalRl {
            cursor -= width;
            output.push(cursor);
        } else {
            output.push(cursor);
            cursor += width;
        }
    }
    output
}

fn orientation(text: &str, policy: TextOrientation) -> f64 {
    match policy {
        TextOrientation::Upright => 0.0,
        TextOrientation::Sideways => 90.0,
        TextOrientation::Mixed if text.chars().any(cjk_upright) => 0.0,
        TextOrientation::Mixed => 90.0,
    }
}

fn cjk_upright(value: char) -> bool {
    matches!(
        value.script(),
        Script::Han | Script::Hiragana | Script::Katakana | Script::Hangul | Script::Bopomofo
    ) || matches!(value, '\u{3000}'..='\u{303f}' | '\u{ff01}'..='\u{ff60}')
}
