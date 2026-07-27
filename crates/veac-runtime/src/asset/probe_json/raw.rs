use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub(super) struct FfprobeOutput {
    #[serde(default)]
    pub streams: Vec<FfprobeStream>,
    pub format: Option<FfprobeFormat>,
}

#[derive(Deserialize)]
pub(super) struct FfprobeFormat {
    pub format_name: Option<String>,
    pub duration: Option<String>,
    pub tags: Option<FfprobeFormatTags>,
}

#[derive(Deserialize)]
pub(super) struct FfprobeFormatTags {
    #[serde(alias = "MAJOR_BRAND")]
    pub major_brand: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct FfprobeStream {
    pub index: Option<u32>,
    pub codec_type: Option<String>,
    pub codec_name: Option<String>,
    pub time_base: Option<String>,
    pub avg_frame_rate: Option<String>,
    pub r_frame_rate: Option<String>,
    pub start_time: Option<String>,
    pub duration: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub pix_fmt: Option<String>,
    pub profile: Option<String>,
    pub level: Option<i32>,
    pub sample_aspect_ratio: Option<String>,
    pub sample_rate: Option<String>,
    pub channels: Option<u16>,
    pub channel_layout: Option<String>,
    #[serde(default)]
    pub disposition: FfprobeDisposition,
    pub tags: Option<FfprobeTags>,
    #[serde(default)]
    pub side_data_list: Vec<FfprobeSideData>,
}

#[derive(Default, Deserialize)]
pub(super) struct FfprobeDisposition {
    #[serde(default, rename = "default")]
    pub is_default: u8,
    #[serde(default)]
    pub attached_pic: u8,
    #[serde(default, alias = "timed_thumbnail")]
    pub timed_thumbnails: u8,
}

#[derive(Deserialize)]
pub(super) struct FfprobeTags {
    #[serde(alias = "ROTATE")]
    pub rotate: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct FfprobeSideData {
    pub rotation: Option<Value>,
}
