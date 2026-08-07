use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::TimeRange;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionCueSemantics {
    pub spans: Vec<CaptionMarkupSpan>,
    pub style: Option<CaptionStyleReference>,
    pub native: Option<CaptionNativeCue>,
    pub words: Vec<CaptionWordTiming>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum CaptionNativeCue {
    Srt {
        index: u64,
    },
    WebVtt {
        identifier: Option<CaptionNativeId>,
        settings: Option<WebVttCueSettings>,
    },
    Ass {
        settings: AssCueSettings,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssCueSettings {
    pub comment: bool,
    pub layer: Option<u32>,
    pub margin_left: Option<u32>,
    pub margin_right: Option<u32>,
    pub margin_vertical: Option<u32>,
    pub effect: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionMarkupSpan {
    pub range: CaptionTextRange,
    pub style: CaptionInlineStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionTextRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionInlineStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub color: Option<String>,
    pub voice: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebVttCueSettings {
    pub line: Option<String>,
    pub position: Option<String>,
    pub size: Option<String>,
    pub align: Option<WebVttTextAlign>,
    pub vertical: Option<WebVttVertical>,
    pub region: Option<WebVttRegionId>,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(transparent)]
pub struct WebVttRegionId(pub String);

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(transparent)]
pub struct CaptionStyleReference(pub String);

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(transparent)]
pub struct CaptionNativeId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WebVttTextAlign {
    Start,
    Center,
    End,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WebVttVertical {
    Rl,
    Lr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionWordTiming {
    pub text: String,
    pub range: TimeRange,
    pub confidence: Option<f64>,
}
