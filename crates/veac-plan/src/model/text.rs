use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{
    Color, FontRef, FontStyle, FontWeight, Shadow, TextAnimation, TextBackground, TextLayout,
    TextOutline, TextPath,
};

use super::PlanInputId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedText {
    pub text: String,
    pub style: ResolvedTextStyle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedTextStyle {
    pub font: ResolvedFont,
    pub fallback_fonts: Vec<ResolvedFont>,
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
    pub spans: Vec<ResolvedTextSpan>,
    pub animation: Option<TextAnimation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedTextSpan {
    pub start: u32,
    pub end: u32,
    pub font: Option<ResolvedFont>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub size_pixels: Option<f64>,
    pub color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedFont {
    pub requested: FontRef,
    pub input_id: PlanInputId,
    pub family: Option<String>,
    pub postscript_name: Option<String>,
    pub face_index: u32,
}
