use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{enabled, OtioTimeRange};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "OTIO_SCHEMA")]
pub enum OtioItem {
    #[serde(rename = "Clip.2")]
    Clip {
        name: String,
        source_range: OtioTimeRange,
        media_reference: OtioMediaReference,
        #[serde(default)]
        metadata: BTreeMap<String, Value>,
        #[serde(default)]
        effects: Vec<Value>,
        #[serde(default)]
        markers: Vec<Value>,
        #[serde(default = "enabled")]
        enabled: bool,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "Gap.1")]
    Gap {
        name: String,
        source_range: OtioTimeRange,
        #[serde(default)]
        metadata: BTreeMap<String, Value>,
        #[serde(default)]
        effects: Vec<Value>,
        #[serde(default)]
        markers: Vec<Value>,
        #[serde(default = "enabled")]
        enabled: bool,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "Transition.1")]
    Transition {
        name: String,
        in_offset: super::OtioRationalTime,
        out_offset: super::OtioRationalTime,
        #[serde(default)]
        metadata: BTreeMap<String, Value>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "OTIO_SCHEMA")]
pub enum OtioMediaReference {
    #[serde(rename = "ExternalReference.1")]
    External {
        target_url: String,
        #[serde(default)]
        available_range: Option<OtioTimeRange>,
        #[serde(default)]
        metadata: BTreeMap<String, Value>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(rename = "MissingReference.1")]
    Missing {
        #[serde(default)]
        metadata: BTreeMap<String, Value>,
        #[serde(flatten)]
        extra: BTreeMap<String, Value>,
    },
    #[serde(other)]
    Unknown,
}
