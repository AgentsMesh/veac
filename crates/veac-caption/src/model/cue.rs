use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{CaptionNativeCue, TimeRange};

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(transparent)]
pub struct CaptionCueId(String);

impl CaptionCueId {
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        if Self::is_valid_value(&value) {
            Ok(Self(value))
        } else {
            Err(format!("invalid caption cue identifier {value:?}"))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn is_valid(&self) -> bool {
        Self::is_valid_value(&self.0)
    }

    fn is_valid_value(value: &str) -> bool {
        let Some(suffix) = value.strip_prefix("cap_") else {
            return false;
        };
        !suffix.is_empty()
            && value.len() <= 128
            && suffix
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_alphanumeric())
            && suffix
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    }
}

impl std::fmt::Display for CaptionCueId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionCue {
    pub id: CaptionCueId,
    pub range: TimeRange,
    pub text: CaptionText,
    pub speaker: Option<String>,
    pub style: Option<String>,
    pub native: Option<CaptionNativeCue>,
    pub words: Vec<CaptionWord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionText {
    pub plain: String,
    pub spans: Vec<CaptionSpan>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionSpan {
    pub range: TextRange,
    pub style: InlineStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InlineStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub color: Option<String>,
    pub voice: Option<String>,
}

impl InlineStyle {
    pub(crate) fn is_plain(&self) -> bool {
        !self.bold
            && !self.italic
            && !self.underline
            && self.color.is_none()
            && self.voice.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionWord {
    pub text: String,
    pub range: TimeRange,
    pub confidence: Option<f64>,
}
