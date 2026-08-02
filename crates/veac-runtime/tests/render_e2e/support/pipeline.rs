use veac_artifact::{ArtifactStore, ExecutionBindings};
use veac_codegen::emitter::{self, BackendAction, BackendCommand, BackendProduct};
use veac_plan::{resolve_one, ResolvedRenderPlan};
use veac_runtime::{
    asset::{probe_with_intent, sha256_identity},
    executor,
};

use super::*;

pub(crate) struct Rendered {
    pub plan: ResolvedRenderPlan,
    pub command: BackendCommand,
}

pub(crate) fn render(
    mut project: ProjectEnvelope,
    assets: &BTreeMap<String, PathBuf>,
    output: &Path,
) -> Rendered {
    hydrate(&mut project, assets);
    let output_id = project.project.render_configs[0].id.clone();
    let plan = resolve_one(&project, &output_id).unwrap_or_else(|error| {
        panic!(
            "canonical plan resolution failed: {error}\n{:#?}",
            error.diagnostics()
        )
    });
    let input_paths = plan
        .inputs
        .iter()
        .map(|input| {
            let material_id = input
                .material_id
                .as_ref()
                .unwrap_or_else(|| panic!("input {} has no material", input.id));
            let path = assets
                .get(material_id.as_str())
                .unwrap_or_else(|| panic!("missing fixture binding for {material_id}"));
            (input.id.clone(), path.clone())
        })
        .collect();
    let mut bindings = ExecutionBindings::from_originals(&plan, &input_paths).unwrap();
    bindings
        .bind_output(plan.output.deliverables[0].id.clone(), output.to_path_buf())
        .unwrap();
    let bundle = emitter::emit_all(&plan, &bindings).unwrap_or_else(|error| {
        panic!(
            "canonical emitter failed: {error}\n{:#?}",
            error.diagnostics()
        )
    });
    let command = bundle
        .tasks()
        .iter()
        .find_map(|task| match (&task.action, task.product) {
            (BackendAction::Ffmpeg(command), BackendProduct::VideoMaster) => Some(command.clone()),
            _ => None,
        })
        .expect("render bundle has a video master task");
    let store = ArtifactStore::new(
        output
            .parent()
            .unwrap_or(Path::new("."))
            .join(".veac-test-artifacts"),
    );
    executor::execute_bundle(&bundle, &store).unwrap_or_else(|error| {
        panic!(
            "canonical FFmpeg render failed: {error}\nfilter graph:\n{}",
            command.filter_graph.as_deref().unwrap_or("<none>")
        )
    });
    Rendered { plan, command }
}

pub(crate) fn hydrate(project: &mut ProjectEnvelope, assets: &BTreeMap<String, PathBuf>) {
    for material in &mut project.project.materials {
        let path = assets
            .get(material.id.as_str())
            .unwrap_or_else(|| panic!("missing source fixture for {}", material.id));
        if matches!(
            material.kind,
            MaterialKind::Font | MaterialKind::Lut1d | MaterialKind::Lut3d
        ) {
            material.identity =
                Some(sha256_identity(path).unwrap_or_else(|error| {
                    panic!("resource hash {} failed: {error}", material.id)
                }));
            material.probe = None;
            continue;
        }
        let snapshot = probe_with_intent(path, material.stream_intent.clone())
            .unwrap_or_else(|error| panic!("probe {} failed: {error}", material.id));
        material.identity = Some(snapshot.observed_identity.clone());
        material.probe = Some(snapshot);
    }
}
