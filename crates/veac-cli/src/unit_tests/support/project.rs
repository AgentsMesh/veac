use std::path::{Path, PathBuf};

use tempfile::TempDir;
use veac_ir::DeliverableId;

use super::source_file;
use crate::environment::Environment;
use crate::error::CliResult;

pub(crate) fn canonical_project(temp: &TempDir, source: &str) -> PathBuf {
    let source = source_file(temp, source);
    let envelope = crate::frontend::check(&source, None, &[], &[], 0).unwrap();
    let path = temp.path().join("project.json");
    std::fs::write(&path, veac_ir::canonical_json(&envelope).unwrap()).unwrap();
    path
}

pub(crate) fn add_second_video(project: &Path) {
    let mut envelope = crate::canonical::load(project).unwrap();
    let mut second = envelope.project.render_configs[0].deliverables[0].clone();
    second.id = DeliverableId::new("dlv_second").unwrap();
    second.target = veac_ir::DeliverableTarget::File {
        name: "second.mp4".to_owned(),
    };
    envelope.project.render_configs[0].deliverables.push(second);
    std::fs::write(project, veac_ir::canonical_json(&envelope).unwrap()).unwrap();
}

pub(crate) fn pin_first_material(project: &Path, identity: veac_ir::MediaIdentity) {
    let mut envelope = crate::canonical::load(project).unwrap();
    envelope.project.materials[0].identity = Some(identity);
    std::fs::write(project, veac_ir::canonical_json(&envelope).unwrap()).unwrap();
}

pub(crate) fn render(
    project: &Path,
    destination: Option<&Path>,
    environment: &dyn Environment,
) -> CliResult {
    crate::commands::render(
        project,
        None,
        crate::planning::InputResolution::default(),
        destination,
        crate::arguments::SubstitutionPolicy::Original,
        crate::arguments::SubstitutionPolicy::Original,
        environment,
    )
}
