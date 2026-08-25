use std::collections::BTreeMap;

use super::*;
use crate::program::loader::{LoadedSource, MemoryLoader};

const OLD: &str = "module { import \"./old.veac\" as value; }";
const CURRENT: &str = "module { import \"./current.veac\" as value; }";
const LEAF: &str = "module { export const time value = 1s; }";
const CHANGED_LEAF: &str = "module { export const time value = 2s; }";

#[test]
fn old_revision_writer_cannot_replace_current_routes_or_reverse_edges() {
    let database = CompilerDatabase::default();
    let (_, stale) = database
        .parse_with_route_admission("root.veac", OLD)
        .unwrap();
    let (_, mut current) = database
        .parse_with_route_admission("root.veac", CURRENT)
        .unwrap();
    assert_eq!(
        database.commit_routes(&mut current, route("./current.veac", "current.veac")),
        DependencyRouteCommit::Committed
    );
    let expected = database.dependency_snapshot("root.veac").unwrap();
    let statistics = database.statistics();

    assert_eq!(
        database.commit_routes(&mut stale.clone(), route("./old.veac", "old.veac")),
        DependencyRouteCommit::RetryRequired
    );
    assert_eq!(database.dependency_snapshot("root.veac"), Some(expected));
    assert_eq!(database.statistics(), statistics);
}

#[test]
fn rejected_late_writer_preserves_clean_interface_semantics() {
    let database = CompilerDatabase::default();
    let (_, stale) = database
        .parse_with_route_admission("root.veac", OLD)
        .unwrap();
    database
        .parse_with_route_admission("root.veac", CURRENT)
        .unwrap();
    assert_eq!(
        database.commit_routes(&mut stale.clone(), route("./old.veac", "old.veac")),
        DependencyRouteCommit::RetryRequired
    );
    let root = LoadedSource {
        id: "root.veac".into(),
        source: CURRENT.into(),
    };
    let loader = MemoryLoader::new(BTreeMap::from([("current.veac".into(), LEAF.into())]));
    let cached = database.module_interface(root.clone(), &loader).unwrap();
    let clean = CompilerDatabase::default()
        .module_interface(root, &loader)
        .unwrap();
    assert_eq!(cached, clean);
}

#[test]
fn clear_epoch_rejects_a_route_writer_even_after_same_revision_returns() {
    let database = CompilerDatabase::default();
    let (_, stale) = database
        .parse_with_route_admission("root.veac", CURRENT)
        .unwrap();
    database.clear();
    database
        .parse_with_route_admission("root.veac", CURRENT)
        .unwrap();
    assert_eq!(
        database.commit_routes(&mut stale.clone(), route("./old.veac", "old.veac")),
        DependencyRouteCommit::RetryRequired
    );
    assert!(database
        .dependency_snapshot("root.veac")
        .unwrap()
        .routes
        .is_empty());
}

#[test]
fn source_revision_a_b_a_never_readmits_the_first_a_writer() {
    let database = CompilerDatabase::default();
    let (_, first_a) = database
        .parse_with_route_admission("root.veac", OLD)
        .unwrap();
    database
        .parse_with_route_admission("root.veac", CURRENT)
        .unwrap();
    let (_, mut second_a) = database
        .parse_with_route_admission("root.veac", OLD)
        .unwrap();
    assert_eq!(
        database.commit_routes(&mut second_a, route("./new.veac", "new.veac")),
        DependencyRouteCommit::Committed
    );

    assert_eq!(
        database.commit_routes(&mut first_a.clone(), route("./old.veac", "old.veac")),
        DependencyRouteCommit::RetryRequired
    );
    let snapshot = database.dependency_snapshot("root.veac").unwrap();
    assert_eq!(
        snapshot.revision,
        database.source_revision("root.veac", OLD)
    );
    assert_eq!(snapshot.direct_dependencies, ["new.veac"]);
}

#[test]
fn rejected_writer_cannot_restore_an_old_reverse_edge() {
    let database = CompilerDatabase::default();
    let (_, stale) = database
        .parse_with_route_admission("root.veac", OLD)
        .unwrap();
    database.parse("old.veac", LEAF).unwrap();
    database.parse("current.veac", LEAF).unwrap();
    let (_, mut current) = database
        .parse_with_route_admission("root.veac", CURRENT)
        .unwrap();
    assert_eq!(
        database.commit_routes(&mut current, route("./current.veac", "current.veac")),
        DependencyRouteCommit::Committed
    );
    assert_eq!(
        database.commit_routes(&mut stale.clone(), route("./old.veac", "old.veac")),
        DependencyRouteCommit::RetryRequired
    );

    database.parse("old.veac", CHANGED_LEAF).unwrap();
    assert!(database.pending_invalidations().is_empty());
    database.parse("current.veac", CHANGED_LEAF).unwrap();
    assert_eq!(database.pending_invalidations(), ["root.veac"]);
}

fn route(requested: &str, resolved: &str) -> BTreeMap<String, String> {
    BTreeMap::from([(requested.into(), resolved.into())])
}
