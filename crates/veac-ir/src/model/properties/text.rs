use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{MaterialId, TextAnimation, TextLayout, TextPath};

use super::{Color, Shadow};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextStyle {
    pub font: FontRef,
    pub fallback_fonts: Vec<FontRef>,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub size_pixels: f64,
    pub color: Color,
    pub tracking_pixels: f64,
    pub line_height: f64,
    pub layout: TextLayout,
    pub path: Option<TextPath>,
    pub background: Option<TextBackground>,
    pub outline: Option<TextOutline>,
    pub shadow: Option<Shadow>,
    pub spans: Vec<TextSpan>,
    pub animation: Option<TextAnimation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum FontRef {
    Family { family: String },
    Material { material_id: MaterialId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextSpan {
    /// Half-open range in Unicode scalar values, not UTF-8 bytes.
    pub start: u32,
    pub end: u32,
    pub font: Option<FontRef>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub size_pixels: Option<f64>,
    pub color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextBackground {
    pub color: Color,
    pub padding_pixels: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextOutline {
    pub color: Color,
    pub width_pixels: f64,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font: FontRef::Family {
                family: "sans-serif".to_owned(),
            },
            fallback_fonts: Vec::new(),
            font_weight: FontWeight::Normal,
            font_style: FontStyle::Normal,
            size_pixels: 48.0,
            color: Color {
                red: 255,
                green: 255,
                blue: 255,
                alpha: 255,
            },
            tracking_pixels: 0.0,
            line_height: 1.0,
            layout: TextLayout::default(),
            path: None,
            background: None,
            outline: None,
            shadow: None,
            spans: Vec::new(),
            animation: None,
        }
    }
}

impl TextStyle {
    pub fn font_refs(&self) -> impl Iterator<Item = &FontRef> {
        std::iter::once(&self.font)
            .chain(&self.fallback_fonts)
            .chain(self.spans.iter().filter_map(|span| span.font.as_ref()))
    }
}
