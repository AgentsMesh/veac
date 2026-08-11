use veac_lang::program::{LoadedSource, SourceLoader};

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
}

pub(super) struct RejectLoader;

impl SourceLoader for RejectLoader {
    fn load(&self, _importer: &str, requested: &str) -> Result<LoadedSource, String> {
        Err(format!(
            "source-string evidence cannot resolve import `{requested}`"
        ))
    }
}
