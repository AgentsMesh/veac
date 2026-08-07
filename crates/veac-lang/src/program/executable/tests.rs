#[path = "../../../tests/executable_temporal/budget.rs"]
mod budget;
mod build;
#[path = "../../../tests/executable_temporal/clock_owners.rs"]
mod clock_owners;
#[path = "../../../tests/executable_temporal/collisions.rs"]
mod collisions;
#[path = "../../../tests/executable_temporal/errors.rs"]
mod errors;
#[path = "../../../tests/support/function_resolution_contract_matrix.rs"]
mod function_resolution_contract_matrix;
#[path = "../../../tests/executable_temporal/identity.rs"]
mod identity;
#[path = "../../../tests/executable_temporal/lowering.rs"]
mod lowering;
mod path;
#[path = "../../../tests/support/resolution_contract_matrix.rs"]
mod resolution_contract_matrix;
mod signature;
#[path = "../../../tests/executable_temporal/sink_matrix.rs"]
mod sink_matrix;
#[path = "../../../tests/executable_temporal/sink_support.rs"]
mod sink_support;
#[path = "../../../tests/support/source_index_contract_matrix.rs"]
mod source_index_contract_matrix;
#[path = "../../../tests/executable_temporal/source_time.rs"]
mod source_time;
#[path = "../../../tests/executable_temporal/support.rs"]
mod support;
#[path = "../../../tests/support/type_registry_contract_matrix.rs"]
mod type_registry_contract_matrix;
#[path = "../../../tests/executable_temporal/validation_edges.rs"]
mod validation_edges;

const MAIN: &str = r#"
fn main(context: Context) -> Project {
    let timeline = sequence(
        identifier("main"), "主时间线",
        sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
    );
    project(identifier("demo"), project_settings(600))
        .with_sequence(timeline)
        .entry(timeline)
}
"#;

fn error(source: &str) -> crate::program::Diagnostic {
    super::prepare_source(source)
        .unwrap_err()
        .as_slice()
        .first()
        .unwrap()
        .clone()
}
