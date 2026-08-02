use std::path::Path;
use std::time::{Duration, Instant};

use serde::Serialize;
use veac_artifact::{ArtifactStore, ContentDigest, DigestAlgorithm};

use crate::arguments::ArtifactCommand;
use crate::{CliError, CliResult};

#[derive(Serialize)]
struct Inspection<'a> {
    descriptor: &'a veac_artifact::ArtifactDescriptor,
    record: &'a veac_artifact::ArtifactRecord,
}

pub(crate) fn run(command: ArtifactCommand) -> CliResult {
    let deadline = Instant::now()
        .checked_add(Duration::from_secs(
            veac_artifact::MAX_MEDIA_DERIVATION_WALL_SECONDS,
        ))
        .ok_or_else(|| CliError::resource_limit("ARTIFACT_FAILED", "deadline overflowed"))?;
    run_until(command, deadline)
}

fn run_until(command: ArtifactCommand, deadline: Instant) -> CliResult {
    match command {
        ArtifactCommand::Inspect { store, key, output } => {
            inspect(&store, &key, output.as_deref(), deadline)
        }
        ArtifactCommand::Materialize {
            store,
            key,
            destination,
        } => materialize(&store, &key, &destination, deadline),
        ArtifactCommand::Remove { store, key } => remove(&store, &key, deadline),
    }
}

fn materialize(store: &Path, key: &str, destination: &Path, deadline: Instant) -> CliResult {
    let store = ArtifactStore::new(store);
    let key = digest(key)?;
    let artifact =
        artifact(store.open_while(&key, || Instant::now() < deadline))?.ok_or_else(|| {
            CliError::new(
                "ARTIFACT_NOT_FOUND",
                format!("artifact {} was not found", key.value),
            )
        })?;
    reject_store_output(store.root(), Some(destination))?;
    let path =
        veac_artifact::materialize_while(&artifact, destination, || Instant::now() < deadline)
            .map_err(|error| artifact_error("ARTIFACT_MATERIALIZE_FAILED", error))?;
    println!("Materialized artifact: {}", path.display());
    Ok(())
}

fn inspect(store: &Path, key: &str, output: Option<&Path>, deadline: Instant) -> CliResult {
    let store = ArtifactStore::new(store);
    let key = digest(key)?;
    let cached =
        artifact(store.open_while(&key, || Instant::now() < deadline))?.ok_or_else(|| {
            CliError::new(
                "ARTIFACT_NOT_FOUND",
                format!("artifact {} was not found", key.value),
            )
        })?;
    let bytes = super::workflow_io::canonical(&Inspection {
        descriptor: cached.descriptor(),
        record: cached.record(),
    })?;
    reject_store_output(store.root(), output)?;
    super::workflow_io::write(&bytes, output, &[])
}

fn remove(store: &Path, key: &str, deadline: Instant) -> CliResult {
    let store = ArtifactStore::new(store);
    let key = digest(key)?;
    if !artifact(store.remove_while(&key, || Instant::now() < deadline))? {
        return Err(CliError::new(
            "ARTIFACT_NOT_FOUND",
            "artifact does not exist",
        ));
    }
    println!("Removed artifact: {}", key.value);
    Ok(())
}

fn digest(value: &str) -> CliResult<ContentDigest> {
    let digest = ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: value.to_owned(),
    };
    artifact(digest.validate()).map(|_| digest)
}

fn artifact<T>(value: veac_artifact::ArtifactResult<T>) -> CliResult<T> {
    value.map_err(|error| artifact_error("ARTIFACT_FAILED", error))
}

fn artifact_error(code: &str, error: veac_artifact::ArtifactError) -> CliError {
    if error.kind == veac_artifact::ArtifactErrorKind::ResourceLimit {
        CliError::resource_limit(code, error.to_string())
    } else {
        CliError::new(code, error.to_string())
    }
}

#[cfg(test)]
#[path = "artifact/tests.rs"]
mod tests;

fn reject_store_output(store: &Path, output: Option<&Path>) -> CliResult {
    let Some(output) = output else {
        return Ok(());
    };
    let root = std::fs::canonicalize(store)
        .map_err(|error| CliError::new("ARTIFACT_FAILED", error.to_string()))?;
    let parent = output.parent().unwrap_or(Path::new("."));
    let parent = std::fs::canonicalize(parent)
        .map_err(|error| CliError::new("ARTIFACT_FAILED", error.to_string()))?;
    if parent.starts_with(root) {
        Err(CliError::new(
            "ARTIFACT_OUTPUT_CONFLICT",
            "artifact command output cannot be inside the artifact store",
        ))
    } else {
        Ok(())
    }
}
