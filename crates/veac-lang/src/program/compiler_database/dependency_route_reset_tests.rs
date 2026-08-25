use std::collections::BTreeMap;

use super::*;
use crate::program::loader::{LoadedSource, MemoryLoader};

const ROOT: &str = concat!(
    "module { import \"./leaf.veac\" as leaf; ",
    "export fn value() -> time { leaf.value() } }"
);
const LEAF: &str = "module { export fn value() -> time { 1s } }";

#[test]
fn route_growth_reset_matches_a_clean_database() {
    let probe = CompilerDatabase::default();
    let (_, mut probe_admission) = probe.parse_with_route_admission("root.veac", ROOT).unwrap();
    probe.commit_routes(&mut probe_admission, route("a", "leaf.veac"));
    let exact = probe.statistics().dependency_retained_bytes;
    let database = CompilerDatabase::with_limits(CompilerDatabaseLimits {
        dependency_retained_bytes: exact,
        ..Default::default()
    });
    let (_, mut admission) = database
        .parse_with_route_admission("root.veac", ROOT)
        .unwrap();
    database.commit_routes(&mut admission, route("a", "leaf.veac"));
    assert_eq!(database.statistics().dependency_resets, 0);

    database.commit_routes(
        &mut admission,
        route("a-request-that-exceeds-the-exact-budget", "leaf.veac"),
    );
    assert_eq!(database.statistics().dependency_resets, 1);
    let root = LoadedSource {
        id: "root.veac".into(),
        source: ROOT.into(),
    };
    let loader = MemoryLoader::new(BTreeMap::from([("leaf.veac".into(), LEAF.into())]));
    let cached = database.module_interface(root.clone(), &loader).unwrap();
    let clean = CompilerDatabase::default()
        .module_interface(root, &loader)
        .unwrap();

    assert_eq!(cached, clean);
    assert!(database.statistics().dependency_resets > 0);
}

fn route(requested: &str, resolved: &str) -> BTreeMap<String, String> {
    BTreeMap::from([(requested.into(), resolved.into())])
}
