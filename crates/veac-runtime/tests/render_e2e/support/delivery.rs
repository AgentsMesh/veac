use veac_artifact::{ArtifactStore, ExecutionBindings};
use veac_codegen::emitter::{self, BackendBundle};
use veac_plan::{resolve_one, ResolvedRenderPlan};
use veac_runtime::executor::{execute_bundle, BundleExecution};

use super::*;

pub(crate) struct PreparedDelivery {
    pub plan: ResolvedRenderPlan,
    pub bundle: BackendBundle,
    pub outputs: BTreeMap<String, PathBuf>,
}

impl PreparedDelivery {
    pub fn execute(&self, store: &ArtifactStore) -> BundleExecution {
        execute_bundle(&self.bundle, store)
            .unwrap_or_else(|error| panic!("bundle execution failed: {error}"))
    }

    pub fn path(&self, id: &str) -> &Path {
        self.outputs
            .get(id)
            .unwrap_or_else(|| panic!("missing delivery path for {id}"))
    }
}

pub(crate) fn prepare_delivery(
    mut project: ProjectEnvelope,
    assets: &BTreeMap<String, PathBuf>,
    output_directory: &Path,
) -> PreparedDelivery {
    hydrate(&mut project, assets);
    let output_id = project.project.render_configs[0].id.clone();
    let plan = resolve_one(&project, &output_id).unwrap_or_else(|error| {
        panic!(
            "canonical delivery resolution failed: {error}\n{:#?}",
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
                .unwrap_or_else(|| panic!("missing binding for {material_id}"));
            (input.id.clone(), path.clone())
        })
        .collect();
    let mut bindings = ExecutionBindings::from_originals(&plan, &input_paths).unwrap();
    let mut outputs = BTreeMap::new();
    for deliverable in &plan.output.deliverables {
        let path = match &deliverable.target {
            DeliverableTarget::File { name } | DeliverableTarget::Package { name } => {
                output_directory.join(name)
            }
            DeliverableTarget::ImageSequence { pattern } => output_directory.join(pattern),
        };
        bindings
            .bind_output(deliverable.id.clone(), path.clone())
            .unwrap();
        outputs.insert(deliverable.id.to_string(), path);
    }
    let bundle = emitter::emit_all(&plan, &bindings).unwrap_or_else(|error| {
        panic!(
            "typed delivery emission failed: {error}\n{:#?}",
            error.diagnostics()
        )
    });
    PreparedDelivery {
        plan,
        bundle,
        outputs,
    }
}
