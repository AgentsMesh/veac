use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{AnnotationTarget, RationalTime};

use crate::Capability;

use super::ApplicationHeader;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnnotationTimeBinding {
    pub provider_origin: RationalTime,
    pub target_origin: RationalTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnnotationApplication {
    pub header: ApplicationHeader,
    pub capability: Capability,
    pub target: AnnotationTarget,
    pub time: Option<AnnotationTimeBinding>,
    pub annotation_id_prefix: String,
}
