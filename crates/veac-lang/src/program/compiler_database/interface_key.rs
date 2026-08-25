use std::mem::size_of_val;
use std::sync::Arc;

use super::revision::CompilerSourceRevision;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct InterfaceQueryKey {
    root_id: String,
    sources: Arc<[(String, CompilerSourceRevision)]>,
    resolutions: Arc<[(String, String, String)]>,
    compiler: CompilerIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CompilerIdentity {
    language_schema: u32,
    core_version: u16,
    domain_opset: u16,
    domain_digest: [u8; 32],
}

impl InterfaceQueryKey {
    pub(super) fn new(
        root_id: &str,
        sources: impl IntoIterator<Item = (String, CompilerSourceRevision)>,
        resolutions: impl IntoIterator<Item = (String, String, String)>,
    ) -> Self {
        let domain = crate::program::DomainOperationRegistry::shared();
        Self {
            root_id: root_id.to_owned(),
            sources: sources.into_iter().collect(),
            resolutions: resolutions.into_iter().collect(),
            compiler: CompilerIdentity {
                language_schema: crate::vocabulary::LANGUAGE_SPEC_SCHEMA_VERSION,
                core_version: crate::program::expression::CORE_VERSION,
                domain_opset: domain.version().raw(),
                domain_digest: *domain.digest().as_bytes(),
            },
        }
    }

    pub(super) fn retained_bytes(&self) -> usize {
        let sources = self
            .sources
            .iter()
            .fold(size_of_val(self.sources.as_ref()), |bytes, (id, _)| {
                bytes.saturating_add(id.len())
            });
        let resolutions = self.resolutions.iter().fold(
            size_of_val(self.resolutions.as_ref()),
            |bytes, (importer, requested, resolved)| {
                bytes
                    .saturating_add(importer.len())
                    .saturating_add(requested.len())
                    .saturating_add(resolved.len())
            },
        );
        self.root_id
            .len()
            .saturating_mul(2)
            .saturating_add(sources)
            .saturating_add(resolutions)
    }
}
