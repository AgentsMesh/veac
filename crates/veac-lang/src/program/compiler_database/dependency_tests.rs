use std::collections::BTreeMap;

use super::*;

const SOURCE: &str =
    "fn main(context: Context) -> Project { project(identifier(\"p\"), project_settings(1)) }";

fn route(requested: &str, resolved: &str) -> BTreeMap<String, String> {
    [(requested.into(), resolved.into())].into_iter().collect()
}

#[test]
fn route_edges_capture_requested_paths_and_invalidate_on_rerouting() {
    let database = CompilerDatabase::default();
    let (_, mut admission) = database
        .parse_with_route_admission("entry.veac", SOURCE)
        .unwrap();
    database.parse("module.veac", SOURCE).unwrap();
    database.parse("other.veac", SOURCE).unwrap();
    database.commit_routes(
        &mut admission,
        BTreeMap::from([(String::from("./module.veac"), String::from("module.veac"))]),
    );
    let snapshot = database.dependency_snapshot("entry.veac").unwrap();
    assert_eq!(snapshot.routes.len(), 1);
    assert_eq!(snapshot.routes[0].importer, "entry.veac");
    assert_eq!(snapshot.routes[0].requested_path, "./module.veac");
    assert_eq!(snapshot.routes[0].resolved_source_id, "module.veac");
    assert_eq!(database.statistics().dependency_route_edges, 1);

    database.commit_routes(
        &mut admission,
        BTreeMap::from([(String::from("./alias.veac"), String::from("module.veac"))]),
    );
    assert_eq!(database.pending_invalidations(), ["entry.veac"]);
    assert_eq!(database.statistics().semantic_invalidations, 1);
    assert_eq!(database.statistics().dependency_route_edges, 1);
    assert!(database.consume_invalidation(&admission));

    database.commit_routes(
        &mut admission,
        BTreeMap::from([(String::from("./alias.veac"), String::from("other.veac"))]),
    );
    assert_eq!(
        database.dependency_snapshot("entry.veac").unwrap().routes[0].resolved_source_id,
        "other.veac"
    );
    assert_eq!(database.pending_invalidations(), ["entry.veac"]);
}

#[test]
fn dependency_snapshots_are_sorted_and_mark_semantic_dependents() {
    let database = CompilerDatabase::default();
    let (_, mut admission) = database
        .parse_with_route_admission("entry.veac", SOURCE)
        .unwrap();
    database.parse("module.veac", SOURCE).unwrap();
    database.commit_routes(&mut admission, route("./module.veac", "module.veac"));
    let snapshot = database.dependency_snapshot("entry.veac").unwrap();
    assert_eq!(snapshot.direct_dependencies, ["module.veac"]);

    database
        .parse("module.veac", &SOURCE.replace("settings(1)", "settings(2)"))
        .unwrap();
    let (_, admission) = database
        .parse_with_route_admission("entry.veac", SOURCE)
        .unwrap();
    let statistics = database.statistics();
    assert_eq!(statistics.syntax_hits, 1);
    assert_eq!(statistics.syntax_misses, 3);
    assert_eq!(database.pending_invalidations(), ["entry.veac"]);
    assert!(database.consume_invalidation(&admission));
    assert!(database.pending_invalidations().is_empty());
}

#[test]
fn clear_releases_dependency_state_but_preserves_invalidation_telemetry() {
    let database = CompilerDatabase::default();
    let (_, mut admission) = database
        .parse_with_route_admission("entry.veac", SOURCE)
        .unwrap();
    database.parse("module.veac", SOURCE).unwrap();
    database.commit_routes(&mut admission, route("./module.veac", "module.veac"));
    database
        .parse("module.veac", &SOURCE.replace("settings(1)", "settings(2)"))
        .unwrap();
    assert_eq!(database.statistics().semantic_invalidations, 1);
    assert_eq!(database.pending_invalidations(), ["entry.veac"]);

    database.clear();

    let stats = database.statistics();
    assert_eq!(stats.pending_semantic_invalidations, 0);
    assert_eq!(stats.semantic_invalidations, 1);
    assert!(database.dependency_snapshot("entry.veac").is_none());
}
