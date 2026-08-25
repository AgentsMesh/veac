use std::sync::Arc;

use super::interface_key::InterfaceQueryKey;
use super::lifecycle::QueryEpoch;
use super::CompilerDatabase;
use crate::program::diagnostic::Diagnostic;
use crate::program::loader::{LoadedSource, SourceLoader};
use crate::program::{ModuleInterface, PreparedSourceGraph};

impl CompilerDatabase {
    pub(crate) fn interface_source_graph(
        &self,
        root: LoadedSource,
        loader: &dyn SourceLoader,
    ) -> Result<(PreparedSourceGraph, super::DependencyRouteAdmission), Vec<Diagnostic>> {
        super::source_graph::discover_admitted(self, root, loader)
    }

    pub(crate) fn prepare_source_graph(
        &self,
        root: LoadedSource,
        loader: &dyn SourceLoader,
        root_imports: &[String],
    ) -> Result<PreparedSourceGraph, Vec<Diagnostic>> {
        super::source_graph::discover_with_root_imports(self, root, loader, root_imports)
    }

    pub(crate) fn cached_interface(
        &self,
        epoch: &QueryEpoch,
        key: &InterfaceQueryKey,
    ) -> Option<Arc<ModuleInterface>> {
        let lifecycle = self
            .lifecycle
            .read()
            .expect("compiler database lifecycle lock poisoned");
        lifecycle.admits(epoch).then(|| {
            self.interface
                .lock()
                .expect("interface cache lock poisoned")
                .get(key)
        })?
    }

    pub(crate) fn cache_interface(
        &self,
        epoch: &QueryEpoch,
        key: InterfaceQueryKey,
        retained_bytes: Option<usize>,
        value: Arc<ModuleInterface>,
    ) -> Arc<ModuleInterface> {
        let lifecycle = self
            .lifecycle
            .read()
            .expect("compiler database lifecycle lock poisoned");
        let mut cache = self
            .interface
            .lock()
            .expect("interface cache lock poisoned");
        let retained_bytes = lifecycle.admits(epoch).then_some(retained_bytes).flatten();
        cache.insert(key, retained_bytes, value)
    }
}
