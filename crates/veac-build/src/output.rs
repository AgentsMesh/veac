use std::{collections::BTreeMap, fmt::Debug, marker::PhantomData};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_artifact::ContentDigest;

use crate::{BuildError, BuildResult, NodeId, PortName};

#[derive(Debug, PartialEq, Eq)]
pub struct InputSlot<T> {
    name: PortName,
    marker: PhantomData<fn(T)>,
}

impl<T> InputSlot<T> {
    pub fn new(name: impl Into<String>) -> BuildResult<Self> {
        Ok(Self {
            name: PortName::new(name)?,
            marker: PhantomData,
        })
    }

    pub fn name(&self) -> &PortName {
        &self.name
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct OutputSlot<T> {
    name: PortName,
    marker: PhantomData<fn() -> T>,
}

impl<T> OutputSlot<T> {
    pub fn new(name: impl Into<String>) -> BuildResult<Self> {
        Ok(Self {
            name: PortName::new(name)?,
            marker: PhantomData,
        })
    }

    pub fn name(&self) -> &PortName {
        &self.name
    }
}

pub struct OutputRef<T> {
    pub(crate) node: NodeId,
    pub(crate) output: PortName,
    pub(crate) marker: PhantomData<fn() -> T>,
}

impl<T> Clone for OutputRef<T> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            output: self.output.clone(),
            marker: PhantomData,
        }
    }
}

impl<T> Debug for OutputRef<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OutputRef")
            .field("node", &self.node)
            .field("output", &self.output)
            .finish()
    }
}

impl<T> PartialEq for OutputRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node && self.output == other.output
    }
}

impl<T> Eq for OutputRef<T> {}

impl<T> OutputRef<T> {
    pub fn node(&self) -> &NodeId {
        &self.node
    }

    pub fn output(&self) -> &PortName {
        &self.output
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(
    try_from = "BTreeMap<PortName, ContentDigest>",
    into = "BTreeMap<PortName, ContentDigest>"
)]
#[schemars(with = "BTreeMap<PortName, ContentDigest>")]
pub struct ArtifactOutputs(BTreeMap<PortName, ContentDigest>);

impl ArtifactOutputs {
    pub fn try_from_iter(
        values: impl IntoIterator<Item = (PortName, ContentDigest)>,
    ) -> BuildResult<Self> {
        let mut outputs = BTreeMap::new();
        for (name, digest) in values {
            digest.validate().map_err(|error| {
                BuildError::invalid(format!("invalid output digest for '{name}': {error}"))
            })?;
            if outputs.insert(name.clone(), digest).is_some() {
                return Err(BuildError::invalid(format!(
                    "output '{name}' is declared more than once"
                )));
            }
        }
        Ok(Self(outputs))
    }

    pub fn one<T>(slot: &OutputSlot<T>, digest: ContentDigest) -> BuildResult<Self> {
        Self::try_from_iter([(slot.name.clone(), digest)])
    }

    pub fn get(&self, name: &PortName) -> Option<&ContentDigest> {
        self.0.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PortName, &ContentDigest)> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl TryFrom<BTreeMap<PortName, ContentDigest>> for ArtifactOutputs {
    type Error = BuildError;

    fn try_from(value: BTreeMap<PortName, ContentDigest>) -> Result<Self, Self::Error> {
        Self::try_from_iter(value)
    }
}

impl From<ArtifactOutputs> for BTreeMap<PortName, ContentDigest> {
    fn from(value: ArtifactOutputs) -> Self {
        value.0
    }
}
