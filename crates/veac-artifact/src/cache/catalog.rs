use super::{io, verified, ArtifactStore, VerifiedArtifact};
use crate::{
    ArtifactDependency, ArtifactError, ArtifactErrorKind, ArtifactKind, ArtifactResult,
    MAX_ARTIFACT_PAYLOAD_BYTES, MAX_IN_MEMORY_ARTIFACT_BYTES,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactCatalogQuery {
    kind: ArtifactKind,
    dependencies: Vec<ArtifactDependency>,
}

impl ArtifactCatalogQuery {
    pub fn new(
        kind: ArtifactKind,
        mut dependencies: Vec<ArtifactDependency>,
    ) -> ArtifactResult<Self> {
        if dependencies.is_empty() {
            return invalid("artifact catalog queries require at least one exact dependency");
        }
        for dependency in &dependencies {
            if dependency.role.is_empty() {
                return invalid("artifact catalog dependency roles cannot be empty");
            }
            dependency.identity.validate()?;
        }
        dependencies.sort_by(|left, right| {
            (&left.role, &left.identity.value).cmp(&(&right.role, &right.identity.value))
        });
        if dependencies.windows(2).any(|pair| pair[0] == pair[1]) {
            return invalid("artifact catalog dependencies must be unique");
        }
        Ok(Self { kind, dependencies })
    }

    pub fn kind(&self) -> ArtifactKind {
        self.kind
    }

    pub fn dependencies(&self) -> &[ArtifactDependency] {
        &self.dependencies
    }
}

impl ArtifactStore {
    pub fn catalog(&self, query: &ArtifactCatalogQuery) -> ArtifactResult<Vec<VerifiedArtifact>> {
        catalog_with_budgets(
            self,
            query,
            MAX_IN_MEMORY_ARTIFACT_BYTES,
            MAX_ARTIFACT_PAYLOAD_BYTES,
        )
    }
}

fn catalog_with_budgets(
    store: &ArtifactStore,
    query: &ArtifactCatalogQuery,
    mut metadata_budget: u64,
    mut payload_budget: u64,
) -> ArtifactResult<Vec<VerifiedArtifact>> {
    let mut matches = Vec::new();
    for key in io::catalog_keys(store.root())? {
        let inspected = verified::inspect_key(store, &key, metadata_budget)?.ok_or_else(|| {
            ArtifactError::new(
                ArtifactErrorKind::CorruptCache,
                "artifact disappeared while cataloging the store",
            )
        })?;
        metadata_budget -= inspected.metadata_bytes();
        let descriptor = &inspected.descriptor;
        if descriptor.kind == query.kind
            && query
                .dependencies
                .iter()
                .all(|dependency| descriptor.dependencies.contains(dependency))
        {
            if inspected.record.size_bytes > payload_budget {
                return catalog_payload_limit();
            }
            let size = inspected.record.size_bytes;
            matches.push(verified::finish_inspected(inspected, payload_budget)?);
            payload_budget -= size;
        }
    }
    Ok(matches)
}

fn catalog_payload_limit<T>() -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::ResourceLimit,
        "artifact catalog payload verification exceeds the byte budget",
    ))
}

fn invalid<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::InvalidContract,
        message,
    ))
}

#[cfg(test)]
#[path = "catalog/tests.rs"]
mod tests;
