use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{LocaleId, ProfileId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectLocale {
    pub id: LocaleId,
    pub language_tag: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectProfile {
    pub id: ProfileId,
    pub execution: ExecutionPolicy,
    pub proxy: ProxyPolicy,
    pub segmentation: SegmentationPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExecutionPolicy {
    Serial {},
    Parallel {
        max_tasks: u16,
    },
    ResourceAware {
        max_tasks: u16,
        cpu_threads: u16,
        memory_mib: u32,
        gpu_slots: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProxyPolicy {
    Disabled {},
    PreferExisting {},
    Require {},
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SegmentationPolicy {
    Whole {},
    Fixed { duration: super::ProjectRational },
    Automatic { max_segments: u32 },
}
