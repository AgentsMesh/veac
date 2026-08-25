use std::collections::BTreeMap;

use super::*;

const CURRENT: &str = "module { import \"./current.veac\" as value; }";

#[test]
fn later_same_revision_observation_readmits_an_initial_writer() {
    let database = CompilerDatabase::default();
    let stale = admission(&database);
    let mut current = admission(&database);

    committed(&database, &mut current, "./current.veac", "current.veac");
    committed(
        &database,
        &mut stale.clone(),
        "./current.veac",
        "current.veac",
    );
    assert_eq!(
        database.dependency_snapshot("root.veac").unwrap().routes[0].resolved_source_id,
        "current.veac"
    );
}

#[test]
fn newer_observation_retries_an_old_writer_with_different_routes() {
    let database = CompilerDatabase::default();
    let mut writer = admission(&database);
    committed(&database, &mut writer, "./old.veac", "old.veac");
    let stale = admission(&database);
    let mut current = admission(&database);
    committed(&database, &mut current, "./current.veac", "current.veac");
    let expected = database.dependency_snapshot("root.veac").unwrap();
    let statistics = database.statistics();

    retry_required(&database, &mut stale.clone(), "./old.veac", "old.veac");
    assert_eq!(database.dependency_snapshot("root.veac"), Some(expected));
    assert_eq!(database.statistics(), statistics);
    assert_eq!(database.pending_invalidations(), ["root.veac"]);
}

#[test]
fn committed_empty_routes_are_distinct_from_no_route_commit() {
    let database = CompilerDatabase::default();
    let mut stale = admission(&database);
    let mut current = admission(&database);
    assert_eq!(
        database.commit_routes(&mut current, BTreeMap::new()),
        DependencyRouteCommit::Committed
    );

    retry_required(&database, &mut stale, "./old.veac", "old.veac");
    assert_eq!(
        database.commit_routes(&mut stale, BTreeMap::new()),
        DependencyRouteCommit::Committed
    );
}

#[test]
fn stale_consumer_cannot_clear_a_newer_generation() {
    let database = CompilerDatabase::default();
    let mut writer = admission(&database);
    committed(&database, &mut writer, "./old.veac", "old.veac");
    let stale = admission(&database);
    let mut current = admission(&database);
    committed(&database, &mut current, "./current.veac", "current.veac");
    let current = admission(&database);

    assert!(!database.consume_invalidation(&stale));
    assert_eq!(database.pending_invalidations(), ["root.veac"]);
    assert!(database.consume_invalidation(&current));
    assert!(database.pending_invalidations().is_empty());
    assert!(!database.consume_invalidation(&current));
}

#[test]
fn consumed_invalidation_still_retries_a_divergent_older_writer() {
    let database = CompilerDatabase::default();
    let mut writer = admission(&database);
    committed(&database, &mut writer, "./old.veac", "old.veac");
    let stale = admission(&database);
    let mut current = admission(&database);
    committed(&database, &mut current, "./current.veac", "current.veac");
    let current = admission(&database);
    assert!(database.consume_invalidation(&current));
    retry_required(&database, &mut stale.clone(), "./old.veac", "old.veac");
    assert!(database.pending_invalidations().is_empty());
}

#[test]
fn clear_epoch_rejects_a_consumer_after_same_source_returns() {
    let database = CompilerDatabase::default();
    let stale = admission(&database);
    database.clear();
    let mut writer = admission(&database);
    committed(&database, &mut writer, "./old.veac", "old.veac");
    committed(&database, &mut writer, "./current.veac", "current.veac");

    assert!(!database.consume_invalidation(&stale));
    assert_eq!(database.pending_invalidations(), ["root.veac"]);
}

#[test]
fn only_the_latest_same_revision_observer_consumes_pending_work() {
    let database = CompilerDatabase::default();
    let mut writer = admission(&database);
    committed(&database, &mut writer, "./old.veac", "old.veac");
    committed(&database, &mut writer, "./current.veac", "current.veac");
    let stale = admission(&database);
    let current = admission(&database);

    assert!(!database.consume_invalidation(&stale));
    assert_eq!(database.pending_invalidations(), ["root.veac"]);
    assert!(database.consume_invalidation(&current));
}

fn admission(database: &CompilerDatabase) -> DependencyRouteAdmission {
    database
        .parse_with_route_admission("root.veac", CURRENT)
        .unwrap()
        .1
}

fn route(requested: &str, resolved: &str) -> BTreeMap<String, String> {
    BTreeMap::from([(requested.into(), resolved.into())])
}

fn committed(
    database: &CompilerDatabase,
    admission: &mut DependencyRouteAdmission,
    requested: &str,
    resolved: &str,
) {
    assert_eq!(
        database.commit_routes(admission, route(requested, resolved)),
        DependencyRouteCommit::Committed
    );
}

fn retry_required(
    database: &CompilerDatabase,
    admission: &mut DependencyRouteAdmission,
    requested: &str,
    resolved: &str,
) {
    assert_eq!(
        database.commit_routes(admission, route(requested, resolved)),
        DependencyRouteCommit::RetryRequired
    );
}
