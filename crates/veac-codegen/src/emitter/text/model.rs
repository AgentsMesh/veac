use std::ops::Range;

use veac_plan::canonical::{Color, FontStyle, FontWeight};

#[derive(Debug, Clone, PartialEq)]
pub(super) struct GlyphStyle {
    pub font: usize,
    pub font_name: String,
    pub font_alias: String,
    pub weight: FontWeight,
    pub font_style: FontStyle,
    pub size: f64,
    pub color: Color,
    pub tracking: f64,
    pub line_height: f64,
}

#[derive(Debug, Clone)]
pub(super) struct StyledRun {
    pub range: Range<usize>,
    pub style: usize,
}

#[derive(Debug, Clone)]
pub(super) struct StyledText {
    pub text: String,
    pub styles: Vec<GlyphStyle>,
    pub runs: Vec<StyledRun>,
}

#[derive(Debug, Clone)]
pub(super) struct LinePiece {
    pub text: String,
    pub style: GlyphStyle,
}

#[derive(Debug, Clone)]
pub(super) struct RenderLine {
    pub pieces: Vec<LinePiece>,
    pub width: f64,
    pub height: f64,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub(super) struct AnimatedPiece {
    pub text: String,
    pub style: GlyphStyle,
    pub unit: usize,
}

#[derive(Debug, Clone)]
pub(super) struct AnimatedLine {
    pub pieces: Vec<AnimatedPiece>,
    pub width: f64,
    pub height: f64,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub(super) struct PlacedPiece {
    pub text: String,
    pub style: GlyphStyle,
    pub unit: usize,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation_degrees: f64,
    pub anchor_x: f64,
    pub anchor_y: f64,
}

pub(super) enum AssLayout {
    Lines(Vec<AnimatedLine>),
    Placed(Vec<PlacedPiece>),
}
