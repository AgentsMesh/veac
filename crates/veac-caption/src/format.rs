use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CaptionFormat {
    Srt,
    WebVtt,
    Ass,
}

impl fmt::Display for CaptionFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Srt => "SRT",
            Self::WebVtt => "WebVTT",
            Self::Ass => "ASS",
        })
    }
}
