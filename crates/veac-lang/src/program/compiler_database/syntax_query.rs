use std::sync::Arc;

use super::key::SyntaxQueryKey;
use super::lifecycle::QueryEpoch;
use super::{syntax_retained, CompilerDatabase, DependencyRouteAdmission};
use crate::program::diagnostic::Diagnostic;
use crate::program::model::SurfaceFile;

impl CompilerDatabase {
    #[allow(dead_code)]
    pub(crate) fn parse(
        &self,
        path: &str,
        source: &str,
    ) -> Result<Arc<SurfaceFile>, Vec<Diagnostic>> {
        self.parse_with_route_admission(path, source)
            .map(|(file, _)| file)
    }

    pub(crate) fn parse_with_route_admission(
        &self,
        path: &str,
        source: &str,
    ) -> Result<(Arc<SurfaceFile>, DependencyRouteAdmission), Vec<Diagnostic>> {
        let key = SyntaxQueryKey::new(path, source);
        let admission = self.dependency_route_admission(path, source);
        if let Some(value) = self.cached_syntax(admission.epoch(), &key) {
            return Ok((value, admission));
        }
        let parsed = Arc::new(super::super::parser::parse(path, source)?);
        let bytes = syntax_retained::bytes(&parsed);
        let parsed = self.cache_syntax(admission.epoch(), key, bytes, parsed);
        Ok((parsed, admission))
    }

    pub(crate) fn reuse_parse_with_route_admission(
        &self,
        path: &str,
        source: &str,
    ) -> Result<(Arc<SurfaceFile>, DependencyRouteAdmission), Vec<Diagnostic>> {
        let admission = self.dependency_route_admission(path, source);
        let key = SyntaxQueryKey::new(path, source);
        let lifecycle = self
            .lifecycle
            .read()
            .expect("compiler database lifecycle lock poisoned");
        let cached = lifecycle.admits(admission.epoch()).then(|| {
            self.syntax
                .lock()
                .expect("syntax cache lock poisoned")
                .peek(&key)
        });
        let parsed = match cached.flatten() {
            Some(value) => value,
            None => Arc::new(super::super::parser::parse(path, source)?),
        };
        Ok((parsed, admission))
    }

    fn cached_syntax(&self, epoch: &QueryEpoch, key: &SyntaxQueryKey) -> Option<Arc<SurfaceFile>> {
        let lifecycle = self
            .lifecycle
            .read()
            .expect("compiler database lifecycle lock poisoned");
        lifecycle.admits(epoch).then(|| {
            self.syntax
                .lock()
                .expect("syntax cache lock poisoned")
                .get(key)
        })?
    }

    pub(super) fn cache_syntax(
        &self,
        epoch: &QueryEpoch,
        key: SyntaxQueryKey,
        bytes: usize,
        value: Arc<SurfaceFile>,
    ) -> Arc<SurfaceFile> {
        let lifecycle = self
            .lifecycle
            .read()
            .expect("compiler database lifecycle lock poisoned");
        let mut cache = self.syntax.lock().expect("syntax cache lock poisoned");
        if lifecycle.admits(epoch) {
            cache.insert(key, bytes, value)
        } else {
            cache.bypass(value)
        }
    }
}
