use std::collections::BTreeMap;

use super::*;
use crate::program::loader::{LoadedSource, MemoryLoader};

const SOURCE: &str =
    "fn main(context: Context) -> Project { project(identifier(\"p\"), project_settings(1)) }";
const PROJECT: &str = r#"
import "./leaf.veac" as leaf;
fn main(context: Context) -> Project {
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let layer = visual_layer(identifier("content"), 0, placement_free(), state,
    track_routing_default()).with_item(item(identifier("result"), item_enabled(),
    during(0s, leaf.value()), source_generated(generator_transparent()), source_timing_native()));
  let timeline = sequence(identifier("main"), "预算测试",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)).with_layer(layer);
  project(identifier("budget"), project_settings(1000)).with_sequence(timeline).entry(timeline)
}
"#;

fn limits(entries: usize, bytes: usize) -> CompilerDatabaseLimits {
    CompilerDatabaseLimits {
        dependency_entries: entries,
        dependency_retained_bytes: bytes,
        ..Default::default()
    }
}

#[test]
fn zero_budget_bypasses_dependency_retention_without_affecting_parsing() {
    let database = CompilerDatabase::with_limits(limits(0, 0));
    database.parse("main.veac", SOURCE).unwrap();
    database.parse("main.veac", SOURCE).unwrap();

    let stats = database.statistics();
    assert_eq!(stats.dependency_retained_entries, 0);
    assert_eq!(stats.dependency_retained_bytes, 0);
    assert_eq!(stats.dependency_bypasses, 2);
    assert_eq!(stats.dependency_resets, 2);
    assert_eq!(stats.syntax_hits, 1);
    assert!(database.dependency_snapshot("main.veac").is_none());
}

#[test]
fn zero_budget_reports_a_conservative_route_bypass() {
    let database = CompilerDatabase::with_limits(limits(0, 0));
    let (_, mut admission) = database
        .parse_with_route_admission("main.veac", SOURCE)
        .unwrap();

    assert_eq!(
        database.commit_routes(&mut admission, route("./leaf.veac", "leaf.veac")),
        DependencyRouteCommit::ConservativeBypass
    );
    assert!(database.dependency_snapshot("main.veac").is_none());
    assert!(database.statistics().dependency_bypasses >= 1);
}

#[test]
fn entry_pressure_resets_the_graph_and_retains_the_current_observation() {
    let database = CompilerDatabase::with_limits(limits(1, usize::MAX));
    database.parse("one.veac", SOURCE).unwrap();
    database.parse("two.veac", SOURCE).unwrap();

    let stats = database.statistics();
    assert_eq!(stats.dependency_resets, 1);
    assert_eq!(stats.dependency_evictions, 1);
    assert_eq!(stats.dependency_bypasses, 0);
    assert_eq!(stats.dependency_retained_entries, 1);
    assert!(database.dependency_snapshot("one.veac").is_none());
    assert!(database.dependency_snapshot("two.veac").is_some());
}

#[test]
fn byte_pressure_is_deterministic_and_accounts_for_every_route_projection() {
    let probe = CompilerDatabase::default();
    probe.parse("one.veac", SOURCE).unwrap();
    let one_bytes = probe.statistics().dependency_retained_bytes;
    let database = CompilerDatabase::with_limits(limits(usize::MAX, one_bytes));
    database.parse("one.veac", SOURCE).unwrap();
    database.parse("two.veac", SOURCE).unwrap();
    assert_eq!(database.statistics().dependency_retained_bytes, one_bytes);

    let routed = CompilerDatabase::default();
    let (_, mut admission) = routed
        .parse_with_route_admission("entry.veac", SOURCE)
        .unwrap();
    routed.commit_routes(
        &mut admission,
        BTreeMap::from([("./module.veac".into(), "module.veac".into())]),
    );
    let stats = routed.statistics();
    assert_eq!(stats.dependency_retained_entries, 7);
    assert!(stats.dependency_retained_bytes > one_bytes);
}

#[test]
fn graph_reset_clears_semantic_caches_and_preserves_clean_diagnostics() {
    let root = LoadedSource {
        id: "root.veac".into(),
        source: concat!(
            "module { import \"./leaf.veac\" as leaf; ",
            "export fn value() -> time { leaf.value() } }"
        )
        .into(),
    };
    let valid = loader("module { export fn value() -> time { 1s } }");
    let invalid = loader("module { export fn value() -> int { 1 } }");
    let database = CompilerDatabase::with_limits(limits(1, usize::MAX));
    database.module_interface(root.clone(), &valid).unwrap();
    let cached = database
        .module_interface(root.clone(), &invalid)
        .unwrap_err();
    let clean = CompilerDatabase::default()
        .module_interface(root, &invalid)
        .unwrap_err();

    assert_eq!(cached, clean);
    assert!(database.statistics().dependency_resets > 0);
    assert!(cached.as_slice()[0].message.contains("return type"));
}

#[test]
fn pressured_database_matches_clean_canonical_output_after_dependency_change() {
    let database = CompilerDatabase::with_limits(limits(1, usize::MAX));
    let root = LoadedSource {
        id: "main.veac".into(),
        source: PROJECT.into(),
    };
    let first = database
        .build_with_loader(root.clone(), &loader(&module("1s")))
        .unwrap();
    let cached = database
        .build_with_loader(root.clone(), &loader(&module("2s")))
        .unwrap();
    let clean = CompilerDatabase::default()
        .build_with_loader(root, &loader(&module("2s")))
        .unwrap();
    let first = veac_ir::canonical_json(first.envelope()).unwrap();
    let cached = veac_ir::canonical_json(cached.envelope()).unwrap();
    let clean = veac_ir::canonical_json(clean.envelope()).unwrap();

    assert_ne!(first, cached);
    assert_eq!(cached, clean);
    assert!(database.statistics().dependency_resets > 0);
}

#[test]
fn clear_releases_dependency_memory_but_preserves_budget_telemetry() {
    let database = CompilerDatabase::with_limits(limits(1, usize::MAX));
    database.parse("one.veac", SOURCE).unwrap();
    database.parse("two.veac", SOURCE).unwrap();
    let before = database.statistics();
    database.clear();
    let after = database.statistics();

    assert_eq!(after.dependency_retained_entries, 0);
    assert_eq!(after.dependency_retained_bytes, 0);
    assert_eq!(after.dependency_resets, before.dependency_resets);
    assert_eq!(after.dependency_evictions, before.dependency_evictions);
}

fn loader(source: &str) -> MemoryLoader {
    MemoryLoader::new(BTreeMap::from([("leaf.veac".into(), source.into())]))
}

fn module(value: &str) -> String {
    format!("module {{ export fn value() -> time {{ {value} }} }}")
}

fn route(requested: &str, resolved: &str) -> BTreeMap<String, String> {
    BTreeMap::from([(requested.into(), resolved.into())])
}
