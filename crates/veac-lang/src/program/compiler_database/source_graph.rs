mod discovery;

use crate::program::diagnostic::Diagnostic;
use crate::program::loader::{LoadedSource, SourceLoader};
use crate::program::PreparedSourceGraph;

use super::interface_key::InterfaceQueryKey;
use super::CompilerDatabase;

#[allow(dead_code)]
pub(super) fn discover(
    database: &CompilerDatabase,
    root: LoadedSource,
    loader: &dyn SourceLoader,
) -> Result<PreparedSourceGraph, Vec<Diagnostic>> {
    discovery::run(database, root, loader, &[]).map(|(graph, _)| graph)
}

pub(super) fn discover_admitted(
    database: &CompilerDatabase,
    root: LoadedSource,
    loader: &dyn SourceLoader,
) -> Result<(PreparedSourceGraph, super::DependencyRouteAdmission), Vec<Diagnostic>> {
    discovery::run(database, root, loader, &[])
}

pub(super) fn discover_with_root_imports(
    database: &CompilerDatabase,
    root: LoadedSource,
    loader: &dyn SourceLoader,
    root_imports: &[String],
) -> Result<PreparedSourceGraph, Vec<Diagnostic>> {
    discovery::run(database, root, loader, root_imports).map(|(graph, _)| graph)
}

impl PreparedSourceGraph {
    pub(crate) fn interface_key(&self) -> InterfaceQueryKey {
        InterfaceQueryKey::new(
            self.root_module(),
            self.sources()
                .iter()
                .map(|(id, source)| (id.clone(), super::CompilerSourceRevision::new(id, source))),
            self.resolutions()
                .iter()
                .map(|((importer, requested), resolved)| {
                    (importer.clone(), requested.clone(), resolved.clone())
                }),
        )
    }
}
