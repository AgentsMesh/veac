mod bindings;
mod item;

pub use bindings::*;
pub use item::*;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtioTimeline {
    #[serde(rename = "OTIO_SCHEMA")]
    pub schema: String,
    pub name: String,
    pub tracks: OtioStack,
    #[serde(default)]
    pub global_start_time: Option<OtioRationalTime>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtioStack {
    #[serde(rename = "OTIO_SCHEMA")]
    pub schema: String,
    pub name: String,
    pub children: Vec<OtioTrack>,
    #[serde(default)]
    pub source_range: Option<OtioTimeRange>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
    #[serde(default)]
    pub effects: Vec<Value>,
    #[serde(default)]
    pub markers: Vec<Value>,
    #[serde(default = "enabled")]
    pub enabled: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtioTrack {
    #[serde(rename = "OTIO_SCHEMA")]
    pub schema: String,
    pub name: String,
    pub kind: String,
    pub children: Vec<OtioItem>,
    #[serde(default)]
    pub source_range: Option<OtioTimeRange>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
    #[serde(default)]
    pub effects: Vec<Value>,
    #[serde(default)]
    pub markers: Vec<Value>,
    #[serde(default = "enabled")]
    pub enabled: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtioRationalTime {
    #[serde(rename = "OTIO_SCHEMA")]
    pub schema: String,
    pub value: f64,
    pub rate: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtioTimeRange {
    #[serde(rename = "OTIO_SCHEMA")]
    pub schema: String,
    pub start_time: OtioRationalTime,
    pub duration: OtioRationalTime,
}

pub(crate) const fn enabled() -> bool {
    true
}
