use super::MAIN;
use crate::program::{DomainOperationId, DomainType};

#[test]
fn executable_build_runs_main_and_freezes_a_connected_graph() {
    let prepared = super::super::prepare_source(MAIN).unwrap();
    assert_eq!(prepared.root_module(), "main.veac");
    assert_eq!(prepared.entry_function().name(), "main");
    assert_eq!(prepared.sources()["main.veac"], MAIN);
    assert!(prepared.type_registry().is_empty());
    assert!(prepared.method_registry().is_empty());
    assert!(prepared.build_input_declarations().is_empty());
    let revision = prepared.source_revision().unwrap();
    let inventory = prepared.source_inventory().unwrap();
    assert_eq!(inventory.revision, revision);

    let built = prepared.execute().unwrap();
    assert_eq!(built.root_module(), "main.veac");
    assert_eq!(built.entry_function().name(), "main");
    assert_eq!(built.sources()["main.veac"], MAIN);
    assert!(built.type_registry().is_empty());
    assert!(built.method_registry().is_empty());
    assert!(built.build_input_declarations().is_empty());
    assert_eq!(built.source_revision().unwrap(), revision);
    assert_eq!(built.source_inventory().unwrap(), inventory);
    assert_eq!(built.graph().root().domain_type(), DomainType::Project);
    assert_eq!(built.graph().root_logical_key(), ["demo"]);
    assert_eq!(
        built.graph().operation(built.graph().root()),
        Some(DomainOperationId::ProjectEntry)
    );
}

#[test]
fn repeated_execution_uses_fresh_graphs_with_deterministic_results() {
    let prepared = super::super::prepare_source(MAIN).unwrap();
    let first = prepared.execute().unwrap();
    let second = prepared.execute().unwrap();
    assert_eq!(
        first.graph().root_logical_key(),
        second.graph().root_logical_key()
    );
    assert_eq!(first.graph().record_count(), second.graph().record_count());
    assert!(!std::ptr::eq(first.graph().root(), second.graph().root()));
}

#[test]
fn main_and_direct_helpers_share_one_graph_transaction() {
    let source = r#"
fn root(context: Context) -> Project {
    let timeline = sequence(
        identifier("main"), "主时间线",
        sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
    );
    project(identifier("demo"), project_settings(600))
        .with_sequence(timeline)
        .entry(timeline)
}
fn main(context: Context) -> Project { root(context) }
"#;
    let built = super::super::build_source(source).unwrap();
    assert_eq!(built.graph().entity_count(), 2);
    assert_eq!(built.graph().root_logical_key(), ["demo"]);
}

#[test]
fn executable_prepare_rejects_unknown_declarations() {
    let source = "unknown declaration {}";
    assert_eq!(
        super::super::prepare_source(source).unwrap_err().as_slice()[0].code,
        "PROGRAM_DECLARATION"
    );
}
