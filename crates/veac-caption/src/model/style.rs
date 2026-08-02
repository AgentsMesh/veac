use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionStyle {
    pub id: String,
    pub font_family: String,
    pub font_size_pixels: u32,
    pub foreground_color: String,
    pub secondary_color: String,
    pub outline_color: String,
    pub background_color: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikeout: bool,
    pub scale_x_percent: f64,
    pub scale_y_percent: f64,
    pub letter_spacing_pixels: f64,
    pub rotation_degrees: f64,
    pub border_style: u32,
    pub outline_pixels: f64,
    pub shadow_pixels: f64,
    pub alignment: u32,
    pub margin_left: i32,
    pub margin_right: i32,
    pub margin_vertical: i32,
    pub encoding: i32,
}
