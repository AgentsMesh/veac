use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use veac_ir::{
    AssCueSettings, CaptionNativeCue, CaptionNativeId, WebVttCueSettings, WebVttRegionId,
    WebVttTextAlign, WebVttVertical,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum CaptionDocumentNative {
    WebVtt { header: WebVttHeader },
    Ass { info: AssScriptInfo },
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebVttHeader {
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssScriptInfo {
    pub title: Option<String>,
    pub script_type: Option<AssScriptType>,
    pub wrap_style: Option<u8>,
    pub scaled_border_and_shadow: Option<bool>,
    pub play_res_x: Option<u32>,
    pub play_res_y: Option<u32>,
    pub ycbcr_matrix: Option<AssYcbcrMatrix>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssScriptType {
    V4Plus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssYcbcrMatrix {
    None,
    Tv601,
    Pc601,
    Tv709,
    Pc709,
    Tv240m,
    Pc240m,
    TvFcc,
    PcFcc,
}
