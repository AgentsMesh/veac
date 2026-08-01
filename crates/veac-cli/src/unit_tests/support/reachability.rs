use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use veac_ir::MaterialId;

use super::FakeEnvironment;
use crate::planning::PreparedPlan;

pub(crate) fn write_project(path: &Path, envelope: &veac_ir::ProjectEnvelope) {
    std::fs::write(path, veac_ir::canonical_json(envelope).unwrap()).unwrap();
}

pub(crate) fn accessed_names(paths: &RefCell<Vec<PathBuf>>) -> BTreeSet<String> {
    paths
        .borrow()
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect()
}

pub(crate) fn assert_exact_inputs(prepared: &PreparedPlan, expected: &[&str]) {
    let expected: BTreeSet<_> = expected
        .iter()
        .map(|value| MaterialId::new(*value).unwrap())
        .collect();
    let required = veac_plan::required_material_ids_one(
        &prepared.project,
        &prepared.plan.output.render_config_id,
    )
    .unwrap();
    let planned: BTreeSet<_> = prepared
        .plan
        .inputs
        .iter()
        .map(|input| input.material_id.clone().expect("canonical input material"))
        .collect();
    let hydrated: BTreeSet<_> = prepared.material_paths.keys().cloned().collect();
    assert_eq!(required, expected);
    assert_eq!(planned, expected);
    assert_eq!(hydrated, expected);
    assert_eq!(prepared.plan.inputs.len(), expected.len());
    assert_eq!(prepared.bindings.inputs().len(), expected.len());
}

pub(crate) fn assert_project_inputs(
    project: &Path,
    envelope: &veac_ir::ProjectEnvelope,
    expected: &[&str],
    identities: &[&str],
    probes: &[&str],
) {
    write_project(project, envelope);
    let environment = FakeEnvironment::success();
    let prepared = crate::planning::prepare(project, None, &environment).unwrap();
    assert_exact_inputs(&prepared, expected);
    assert_eq!(environment.identity_paths.borrow().len(), identities.len());
    assert_eq!(environment.probe_paths.borrow().len(), probes.len());
    assert_eq!(
        accessed_names(&environment.identity_paths),
        names(identities)
    );
    assert_eq!(accessed_names(&environment.probe_paths), names(probes));
}

fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}
