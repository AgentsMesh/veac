use sha2::{Digest, Sha256};

use super::PreparedSourceGraph;
use crate::program::SourceAuthority;

const HASH_DOMAIN: &[u8] = b"veac-prepared-source-graph-v1\0";
const NODE_TAG: u8 = 1;
const ROUTE_TAG: u8 = 2;

/// Identity of a frozen source graph, including ownership and import routing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PreparedSourceGraphRevision {
    sha256: String,
}

impl PreparedSourceGraphRevision {
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

impl PreparedSourceGraph {
    pub fn complete_revision(&self) -> PreparedSourceGraphRevision {
        let mut hash = Sha256::new();
        hash.update(HASH_DOMAIN);
        framed(&mut hash, self.root_module.as_bytes());
        hash.update((self.sources.len() as u64).to_be_bytes());
        for (id, source) in &self.sources {
            hash.update([NODE_TAG]);
            framed(&mut hash, id.as_bytes());
            hash.update([authority_tag(self.authority(id))]);
            framed(&mut hash, source.as_bytes());
        }
        hash.update((self.resolutions.len() as u64).to_be_bytes());
        for ((importer, requested), resolved) in &self.resolutions {
            hash.update([ROUTE_TAG]);
            framed(&mut hash, importer.as_bytes());
            framed(&mut hash, requested.as_bytes());
            framed(&mut hash, resolved.as_bytes());
        }
        PreparedSourceGraphRevision {
            sha256: format!("{:x}", hash.finalize()),
        }
    }
}

fn authority_tag(authority: SourceAuthority) -> u8 {
    match authority {
        SourceAuthority::Project => 0,
        SourceAuthority::ReadOnlyDependency => 1,
    }
}

fn framed(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_be_bytes());
    hash.update(bytes);
}
