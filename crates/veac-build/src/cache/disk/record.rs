use serde::{Deserialize, Serialize};
use veac_artifact::{ArtifactStore, ContentDigest};

use crate::{ArtifactOutputs, BuildError, BuildResult, NodeCacheKey, PortName};

use super::RECORD_VERSION;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ComputationRecord {
    version: u32,
    key: String,
    outputs: Vec<RecordOutput>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordOutput {
    name: String,
    artifact: ContentDigest,
}

impl ComputationRecord {
    pub(super) fn capture(
        key: &NodeCacheKey,
        outputs: &ArtifactOutputs,
        store: &ArtifactStore,
    ) -> BuildResult<Self> {
        let mut captured = Vec::with_capacity(outputs.len());
        for (name, artifact) in outputs.iter() {
            verified(store, artifact)?;
            captured.push(RecordOutput {
                name: name.as_str().to_owned(),
                artifact: artifact.clone(),
            });
        }
        Ok(Self {
            version: RECORD_VERSION,
            key: key.digest().value.clone(),
            outputs: captured,
        })
    }

    pub(super) fn open(
        self,
        key: &NodeCacheKey,
        store: &ArtifactStore,
    ) -> BuildResult<ArtifactOutputs> {
        if self.version != RECORD_VERSION || self.key != key.digest().value {
            return Err(BuildError::cache("computation record identity is invalid"));
        }
        let mut previous: Option<&str> = None;
        for output in &self.outputs {
            if previous.is_some_and(|value| value >= output.name.as_str()) {
                return Err(BuildError::cache(
                    "computation outputs are not unique and sorted",
                ));
            }
            previous = Some(&output.name);
            verified(store, &output.artifact)?;
        }
        ArtifactOutputs::try_from_iter(
            self.outputs
                .into_iter()
                .map(|output| Ok((PortName::new(output.name)?, output.artifact)))
                .collect::<BuildResult<Vec<_>>>()?,
        )
    }
}

fn verified(store: &ArtifactStore, key: &ContentDigest) -> BuildResult<()> {
    if store.open(key).map_err(artifact_error)?.is_none() {
        return Err(BuildError::cache(format!(
            "referenced artifact {} is missing",
            key.value
        )));
    }
    Ok(())
}

fn artifact_error(error: veac_artifact::ArtifactError) -> BuildError {
    BuildError::cache(format!("artifact cache failure: {error}"))
}

#[cfg(test)]
#[path = "record/tests.rs"]
mod tests;
