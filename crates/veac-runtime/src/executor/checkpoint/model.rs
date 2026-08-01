use std::path::PathBuf;

use veac_artifact::{ArtifactRecord, DeliveryPackageInventory};

pub(in crate::executor) struct ResumeHit {
    pub record: ArtifactRecord,
    pub paths: Vec<PathBuf>,
    pub output_records: Vec<ArtifactRecord>,
}

#[derive(Debug)]
pub(in crate::executor) struct StoredCheckpoint {
    pub record: ArtifactRecord,
    pub output_records: Vec<ArtifactRecord>,
}

pub(super) fn package_record_matches(
    record: &ArtifactRecord,
    inventory: &DeliveryPackageInventory,
) -> bool {
    record.content == inventory.tree && record.size_bytes == inventory.size_bytes()
}
