use std::path::{Path, PathBuf};

use veac_artifact::ContentDigest;
use veac_codegen::emitter::{
    BackendCommand, MAX_FILTER_GRAPH_BYTES, MAX_INLINE_FILTER_GRAPH_BYTES,
};

use super::super::directory::{Directory, EntryIdentity};
use crate::RuntimeError;

const NAME: &str = "filter-complex.ffscript";

pub(super) struct StagedScript {
    pub path: PathBuf,
    identity: EntryIdentity,
    content: ContentDigest,
}

pub(super) fn stage(
    root: &Path,
    directory: &Directory,
    command: &BackendCommand,
) -> Result<Option<StagedScript>, RuntimeError> {
    let Some(graph) = command
        .filter_graph
        .as_ref()
        .filter(|value| value.len() > MAX_INLINE_FILTER_GRAPH_BYTES)
    else {
        return Ok(None);
    };
    if graph.len() > MAX_FILTER_GRAPH_BYTES {
        return Err(RuntimeError::new(
            "FFmpeg filter graph exceeds the backend graph limit",
        ));
    }
    let identity = directory.write_all_sync(NAME, graph.as_bytes())?;
    Ok(Some(StagedScript {
        path: root.join(NAME),
        identity,
        content: ContentDigest::sha256(graph.as_bytes()),
    }))
}

pub(super) fn finish(
    directory: &Directory,
    script: Option<StagedScript>,
    execution: Result<(), RuntimeError>,
) -> Result<(), RuntimeError> {
    let Some(script) = script else {
        return execution;
    };
    let cleanup = verify_and_remove(directory, &script);
    match (execution, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(error)) => Err(error.context("cannot clean staged FFmpeg filter script")),
        (Err(error), Err(cleanup)) => Err(RuntimeError {
            kind: error.kind,
            message: format!("{error}; cannot clean staged FFmpeg filter script: {cleanup}"),
        }),
    }
}

fn verify_and_remove(directory: &Directory, script: &StagedScript) -> Result<(), RuntimeError> {
    directory.require(NAME, script.identity)?;
    let (bytes, identity) = directory.read_bounded(NAME, MAX_FILTER_GRAPH_BYTES as u64)?;
    if identity != script.identity || ContentDigest::sha256(&bytes) != script.content {
        return Err(RuntimeError::new(
            "staged FFmpeg filter script changed during execution",
        ));
    }
    directory.remove_bound(NAME, script.identity)
}
