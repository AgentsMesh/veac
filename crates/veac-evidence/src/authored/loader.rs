use veac_lang::program::{LoadedSource, SourceAuthority, SourceLoader};

use super::{EVIDENCE_MODULE_ID, EVIDENCE_MODULE_SOURCE};

pub(super) struct EvidenceLoader<'a> {
    delegate: &'a dyn SourceLoader,
}

impl<'a> EvidenceLoader<'a> {
    pub(super) fn new(delegate: &'a dyn SourceLoader) -> Self {
        Self { delegate }
    }
}

impl SourceLoader for EvidenceLoader<'_> {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String> {
        if requested == EVIDENCE_MODULE_ID {
            Ok(LoadedSource {
                id: EVIDENCE_MODULE_ID.to_owned(),
                source: EVIDENCE_MODULE_SOURCE.to_owned(),
            })
        } else {
            self.delegate.load(importer, requested)
        }
    }

    fn authority(&self, source_id: &str) -> SourceAuthority {
        if source_id == EVIDENCE_MODULE_ID {
            SourceAuthority::ReadOnlyDependency
        } else {
            self.delegate.authority(source_id)
        }
    }
}

pub(super) struct RejectLoader;

impl SourceLoader for RejectLoader {
    fn load(&self, _importer: &str, requested: &str) -> Result<LoadedSource, String> {
        Err(format!(
            "source-string evidence cannot resolve import `{requested}`"
        ))
    }

    fn authority(&self, _source_id: &str) -> SourceAuthority {
        SourceAuthority::Project
    }
}
