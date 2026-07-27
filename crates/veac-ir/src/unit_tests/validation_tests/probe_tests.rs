#[path = "probe_tests/header_time_tests.rs"]
mod header_time_tests;
#[path = "probe_tests/inventory_tests.rs"]
mod inventory_tests;
#[path = "probe_tests/selection_tests.rs"]
mod selection_tests;

use super::*;

fn material_mut(project: &mut ProjectEnvelope) -> &mut Material {
    &mut project.project.materials[0]
}

fn probe_mut(project: &mut ProjectEnvelope) -> &mut MediaProbeSnapshot {
    material_mut(project).probe.as_mut().unwrap()
}

fn probe_codes(project: &ProjectEnvelope) -> Vec<String> {
    validate(project)
        .err()
        .map(|errors| {
            errors
                .into_diagnostics()
                .into_iter()
                .map(|diagnostic| diagnostic.code)
                .collect()
        })
        .unwrap_or_default()
}

fn assert_probe_code(project: &ProjectEnvelope, code: &str) {
    assert_code(&probe_codes(project), code);
}

fn selection(global_index: u32, type_index: u32) -> StreamSelection {
    StreamSelection {
        global_index,
        type_index,
    }
}
