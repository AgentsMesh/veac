mod path;
mod vertical;

use std::collections::BTreeMap;

use unicode_segmentation::UnicodeSegmentation;
use veac_plan::canonical::TextWritingMode;
use veac_plan::ResolvedTextStyle;

use super::animation;
use super::error::TextError;
use super::fonts::FontBook;
use super::layout;
use super::model::{AnimatedLine, GlyphStyle, LinePiece, PlacedPiece};

const MAX_PLACED_GRAPHEMES: usize = 2_048;

#[derive(Clone)]
pub(super) struct MeasuredPiece {
    pub text: String,
    pub style: GlyphStyle,
    pub unit: usize,
    pub advance: f64,
    pub cross: f64,
}

pub(super) fn required(style: &ResolvedTextStyle) -> bool {
    style.path.is_some()
        || style.layout.writing_mode != TextWritingMode::HorizontalTb
        || animation::has_transform(style.animation.as_ref())
        || style
            .animation
            .as_ref()
            .is_some_and(|animation| animation.highlight.is_some())
}

pub(super) fn place(
    lines: &[AnimatedLine],
    style: &ResolvedTextStyle,
    fonts: &mut FontBook,
    surface: (u32, u32),
    origin: (f64, f64),
) -> Result<Vec<PlacedPiece>, TextError> {
    let measured = measure(lines, fonts);
    let mut placed = if let Some(text_path) = &style.path {
        if style.layout.writing_mode != TextWritingMode::HorizontalTb {
            return Err(TextError::new(
                "TEXT_PATH_WRITING_MODE",
                "text path requires horizontal-tb writing mode",
            ));
        }
        path::place(&measured, text_path, surface)?
    } else {
        match style.layout.writing_mode {
            TextWritingMode::HorizontalTb => horizontal(lines, &measured),
            mode => vertical::place(&measured, style.layout, mode, surface, origin),
        }
    };
    if placed.len() > MAX_PLACED_GRAPHEMES {
        return Err(TextError::new(
            "TEXT_LAYOUT_UNIT_LIMIT",
            format!(
                "text requires {} placed graphemes; limit is {MAX_PLACED_GRAPHEMES}",
                placed.len()
            ),
        ));
    }
    assign_anchors(&mut placed);
    Ok(placed)
}

fn measure(lines: &[AnimatedLine], fonts: &mut FontBook) -> Vec<Vec<MeasuredPiece>> {
    let mut output = Vec::with_capacity(lines.len());
    for line in lines {
        let mut measured = Vec::new();
        for piece in &line.pieces {
            for text in piece.text.graphemes(true) {
                let source = LinePiece {
                    text: text.to_owned(),
                    style: piece.style.clone(),
                };
                measured.push(MeasuredPiece {
                    text: text.to_owned(),
                    style: piece.style.clone(),
                    unit: piece.unit,
                    advance: layout::measure(&[source], fonts).max(0.01),
                    cross: (piece.style.size * piece.style.line_height).max(0.01),
                });
            }
        }
        output.push(measured);
    }
    output
}

fn horizontal(lines: &[AnimatedLine], measured: &[Vec<MeasuredPiece>]) -> Vec<PlacedPiece> {
    let mut output = Vec::new();
    for (line, pieces) in lines.iter().zip(measured) {
        let mut x = line.x;
        for piece in pieces {
            output.push(placed(
                piece,
                x + piece.advance / 2.0,
                line.y + line.height / 2.0,
                0.0,
            ));
            x += piece.advance;
        }
    }
    output
}

pub(super) fn placed(piece: &MeasuredPiece, x: f64, y: f64, rotation: f64) -> PlacedPiece {
    PlacedPiece {
        text: piece.text.clone(),
        style: piece.style.clone(),
        unit: piece.unit,
        x,
        y,
        width: piece.advance,
        height: piece.cross,
        rotation_degrees: rotation,
        anchor_x: 0.0,
        anchor_y: 0.0,
    }
}

fn assign_anchors(pieces: &mut [PlacedPiece]) {
    let mut bounds: BTreeMap<usize, (f64, f64, f64, f64)> = BTreeMap::new();
    for piece in pieces.iter() {
        let value = bounds.entry(piece.unit).or_insert((
            piece.x - piece.width / 2.0,
            piece.y - piece.height / 2.0,
            piece.x + piece.width / 2.0,
            piece.y + piece.height / 2.0,
        ));
        value.0 = value.0.min(piece.x - piece.width / 2.0);
        value.1 = value.1.min(piece.y - piece.height / 2.0);
        value.2 = value.2.max(piece.x + piece.width / 2.0);
        value.3 = value.3.max(piece.y + piece.height / 2.0);
    }
    for piece in pieces {
        if let Some(value) = bounds.get(&piece.unit) {
            piece.anchor_x = (value.0 + value.2) / 2.0;
            piece.anchor_y = (value.1 + value.3) / 2.0;
        }
    }
}
