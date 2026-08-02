use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextLayout {
    pub box_width_pixels: Option<f64>,
    pub box_height_pixels: Option<f64>,
    pub wrap: TextWrap,
    pub overflow: TextOverflow,
    pub horizontal_alignment: HorizontalTextAlignment,
    pub vertical_alignment: VerticalTextAlignment,
    pub writing_mode: TextWritingMode,
    pub orientation: TextOrientation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TextWrap {
    None,
    Word,
    Character,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TextOverflow {
    Visible,
    Clip,
    Ellipsis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HorizontalTextAlignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerticalTextAlignment {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TextWritingMode {
    HorizontalTb,
    VerticalRl,
    VerticalLr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TextOrientation {
    /// CJK stays upright while Latin and similar scripts rotate clockwise.
    Mixed,
    Upright,
    Sideways,
}

impl Default for TextLayout {
    fn default() -> Self {
        Self {
            box_width_pixels: None,
            box_height_pixels: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Visible,
            horizontal_alignment: HorizontalTextAlignment::Center,
            vertical_alignment: VerticalTextAlignment::Middle,
            writing_mode: TextWritingMode::HorizontalTb,
            orientation: TextOrientation::Mixed,
        }
    }
}
