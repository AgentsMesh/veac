use std::path::PathBuf;

use veac_artifact::ArtifactRecord;
use veac_codegen::emitter::BackendPhase;
use veac_ir::DeliverableId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskExecution {
    pub deliverable_id: DeliverableId,
    pub phase: BackendPhase,
    pub cache_hit: bool,
    pub outputs: Vec<PathBuf>,
    pub output_records: Vec<ArtifactRecord>,
    pub checkpoint: ArtifactRecord,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BundleExecution {
    pub tasks: Vec<TaskExecution>,
}
