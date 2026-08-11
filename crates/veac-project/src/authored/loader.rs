use veac_lang::program::{LoadedSource, SourceLoader};

use super::{PROJECT_MODULE_ID, PROJECT_MODULE_SOURCE};

pub(super) struct ProjectLoader<'a> {
    delegate: &'a dyn SourceLoader,
}

impl<'a> ProjectLoader<'a> {
    pub(super) fn new(delegate: &'a dyn SourceLoader) -> Self {
        Self { delegate }
    }
}

impl SourceLoader for ProjectLoader<'_> {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String> {
        if requested == PROJECT_MODULE_ID {
            Ok(LoadedSource {
                id: PROJECT_MODULE_ID.to_owned(),
                source: PROJECT_MODULE_SOURCE.to_owned(),
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
            "source-string project cannot resolve import `{requested}`"
        ))
    }
}
