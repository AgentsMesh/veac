use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::MulticamGroupId;

use super::ApplicationHeader;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MulticamSyncApplication {
    pub header: ApplicationHeader,
    pub group_id: MulticamGroupId,
}
