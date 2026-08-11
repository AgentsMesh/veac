use std::fmt::{Display, Formatter};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{BuildError, BuildResult};

const MAX_NODE_ID_BYTES: usize = 128;
const MAX_PORT_NAME_BYTES: usize = 64;

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(try_from = "String", into = "String")]
#[schemars(with = "String")]
pub struct NodeId(String);

impl NodeId {
    pub fn new(value: impl Into<String>) -> BuildResult<Self> {
        checked(value.into(), MAX_NODE_ID_BYTES, "node id").map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for NodeId {
    type Error = BuildError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<NodeId> for String {
    fn from(value: NodeId) -> Self {
        value.0
    }
}

impl Display for NodeId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(try_from = "String", into = "String")]
#[schemars(with = "String")]
pub struct PortName(String);

impl PortName {
    pub fn new(value: impl Into<String>) -> BuildResult<Self> {
        checked(value.into(), MAX_PORT_NAME_BYTES, "port name").map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for PortName {
    type Error = BuildError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PortName> for String {
    fn from(value: PortName) -> Self {
        value.0
    }
}

impl Display for PortName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

pub(crate) fn checked(value: String, limit: usize, subject: &str) -> BuildResult<String> {
    if value.is_empty() || value.len() > limit {
        return Err(BuildError::invalid(format!(
            "{subject} must contain between 1 and {limit} bytes"
        )));
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
    {
        return Err(BuildError::invalid(format!(
            "{subject} may only contain ASCII letters, digits, '.', '_' or '-'"
        )));
    }
    Ok(value)
}
