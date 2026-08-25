use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::*;
use crate::program::expression::ExecutionBudget;
use crate::program::loader::{LoadedSource, SourceAuthority, SourceLoader};

const ROOT: &str = concat!(
    "module { import \"./leaf.veac\" as leaf; ",
    "export fn value() -> time { leaf.value() } }"
);
const DRIFT: &str = "module { export fn drift() -> time { 3s } }";
const LEAF: &str = "module { export fn value() -> time { 1s } }";

struct DriftLoader {
    database: Arc<CompilerDatabase>,
    calls: AtomicUsize,
    always: bool,
}

impl DriftLoader {
    fn new(database: Arc<CompilerDatabase>, always: bool) -> Self {
        Self {
            database,
            calls: AtomicUsize::new(0),
            always,
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl SourceLoader for DriftLoader {
    fn load(&self, _: &str, requested: &str) -> Result<LoadedSource, String> {
        assert_eq!(requested, "./leaf.veac");
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        if self.always || call == 0 {
            self.database.parse("root.veac", DRIFT).unwrap();
        }
        Ok(LoadedSource {
            id: "leaf.veac".into(),
            source: LEAF.into(),
        })
    }

    fn authority(&self, _: &str) -> SourceAuthority {
        SourceAuthority::Project
    }
}

#[test]
fn discovery_restarts_after_revision_generation_drift() {
    let database = Arc::new(CompilerDatabase::default());
    let loader = DriftLoader::new(Arc::clone(&database), false);
    let graph = database.prepare_source_graph(root(), &loader, &[]).unwrap();

    assert_eq!(loader.calls(), 2);
    assert_eq!(
        graph.resolved_id("root.veac", "./leaf.veac"),
        Some("leaf.veac")
    );
    assert_registered(&database);
}

#[test]
fn resolver_restarts_after_revision_generation_drift() {
    let database = Arc::new(CompilerDatabase::default());
    let loader = DriftLoader::new(Arc::clone(&database), false);
    crate::program::resolve::module_interface_scope(
        root(),
        &loader,
        &ExecutionBudget::default(),
        &database,
    )
    .unwrap();

    assert_eq!(loader.calls(), 2);
    assert_registered(&database);
}

#[test]
fn bounded_discovery_fails_instead_of_returning_unregistered_routes() {
    let database = Arc::new(CompilerDatabase::default());
    let loader = DriftLoader::new(Arc::clone(&database), true);
    let errors = database
        .prepare_source_graph(root(), &loader, &[])
        .unwrap_err();

    assert_eq!(loader.calls(), DEPENDENCY_ROUTE_ATTEMPTS);
    assert_eq!(errors[0].code, "PROGRAM_QUERY_CHANGED");
}

#[test]
fn bounded_resolver_fails_instead_of_returning_unregistered_routes() {
    let database = Arc::new(CompilerDatabase::default());
    let loader = DriftLoader::new(Arc::clone(&database), true);
    let errors = crate::program::resolve::module_interface_scope(
        root(),
        &loader,
        &ExecutionBudget::default(),
        &database,
    )
    .unwrap_err();

    assert_eq!(loader.calls(), DEPENDENCY_ROUTE_ATTEMPTS);
    assert_eq!(errors[0].code, "PROGRAM_QUERY_CHANGED");
}

fn assert_registered(database: &CompilerDatabase) {
    let snapshot = database.dependency_snapshot("root.veac").unwrap();
    assert_eq!(snapshot.direct_dependencies, ["leaf.veac"]);
    assert_eq!(snapshot.routes[0].resolved_source_id, "leaf.veac");
}

fn root() -> LoadedSource {
    LoadedSource {
        id: "root.veac".into(),
        source: ROOT.into(),
    }
}
