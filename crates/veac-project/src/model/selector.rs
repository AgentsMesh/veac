use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{AxisId, AxisValue, LocaleId, ProfileId, TargetId};

pub type MatrixAssignment = BTreeMap<AxisId, AxisValue>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TargetRef {
    pub target: TargetId,
    pub profile: Option<ProfileId>,
    pub selector: InstanceSelector,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum InstanceSelector {
    Same {},
    Exact {
        locale: Option<LocaleId>,
        axes: MatrixAssignment,
    },
    AllMatching {},
}
