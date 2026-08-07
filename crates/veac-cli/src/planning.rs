use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use veac_artifact::ExecutionBindings;
use veac_ir::{MaterialId, RenderConfigId};
use veac_plan::ResolvedRenderPlan;

use crate::canonical;
use crate::diagnostic;
use crate::environment::Environment;
use crate::error::{CliError, CliResult};
use crate::material_root::MaterialRoot;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct InputResolution<'a> {
    material_root: Option<&'a Path>,
    binding_file: Option<&'a Path>,
}

impl<'a> InputResolution<'a> {
    pub(crate) fn new(material_root: Option<&'a Path>, binding_file: Option<&'a Path>) -> Self {
        Self {
            material_root,
            binding_file,
        }
    }
}

#[derive(Debug)]
pub(crate) struct PreparedPlan {
    pub project: veac_ir::ProjectEnvelope,
    pub plan: ResolvedRenderPlan,
    pub bindings: ExecutionBindings,
    pub project_file: PathBuf,
    pub material_root: Option<MaterialRoot>,
    pub material_paths: BTreeMap<MaterialId, PathBuf>,
    pub binding_file: Option<PathBuf>,
}

pub(crate) fn prepare_with_material_root(
    project: &Path,
    config: Option<&str>,
    material_root: Option<&Path>,
    environment: &dyn Environment,
) -> CliResult<PreparedPlan> {
    let loaded = canonical::load_local(project)?;
    let config_id = select_config(&loaded.envelope, config)?;
    let required = veac_plan::required_material_ids_one(&loaded.envelope, &config_id)
        .map_err(diagnostic::resolution)?;
    let hydrated =
        canonical::hydrate_with_material_root(loaded, &required, material_root, environment)?;
    let plan =
        veac_plan::resolve_one(&hydrated.envelope, &config_id).map_err(diagnostic::resolution)?;
    let bindings = input_bindings(&plan, &hydrated.material_paths)?;
    Ok(PreparedPlan {
        project: hydrated.envelope,
        plan,
        bindings,
        project_file: hydrated.project_file,
        material_root: Some(hydrated.material_root),
        material_paths: hydrated.material_paths,
        binding_file: None,
    })
}

pub(crate) fn prepare_with_input_resolution(
    project: &Path,
    config: Option<&str>,
    resolution: InputResolution<'_>,
    environment: &dyn Environment,
) -> CliResult<PreparedPlan> {
    let InputResolution {
        material_root,
        binding_file,
    } = resolution;
    if material_root.is_some() && binding_file.is_some() {
        return Err(CliError::new(
            "MATERIAL_ROOT_BINDINGS_CONFLICT",
            "--material-root and --bindings are alternative input-resolution authorities",
        ));
    }
    let Some(binding_file) = binding_file else {
        return prepare_with_material_root(project, config, material_root, environment);
    };
    let loaded = canonical::load_local(project)?;
    let config_id = select_config(&loaded.envelope, config)?;
    let plan =
        veac_plan::resolve_one(&loaded.envelope, &config_id).map_err(diagnostic::resolution)?;
    let binding_file = crate::fs::canonical_file(binding_file, "execution bindings")?;
    let manifest = crate::commands::workflow_io::read_json(&binding_file, "execution bindings")?;
    let bindings = veac_artifact::resolve_binding_manifest(&plan, &manifest)
        .map_err(|error| CliError::new("EXECUTION_BINDINGS_FAILED", error.to_string()))?;
    let material_paths = plan
        .inputs
        .iter()
        .filter_map(|input| {
            input.material_id.as_ref().and_then(|id| {
                bindings
                    .input(&input.id)
                    .and_then(|binding| binding.resource())
                    .map(|resource| (id.clone(), resource.path().to_owned()))
            })
        })
        .collect();
    Ok(PreparedPlan {
        project: loaded.envelope,
        plan,
        bindings,
        project_file: loaded.project_file,
        material_root: None,
        material_paths,
        binding_file: Some(binding_file),
    })
}

pub(crate) fn input_bindings(
    plan: &ResolvedRenderPlan,
    material_paths: &BTreeMap<MaterialId, PathBuf>,
) -> CliResult<ExecutionBindings> {
    let mut paths = BTreeMap::new();
    for input in &plan.inputs {
        let Some(material_id) = input.material_id.as_ref() else {
            return Err(CliError::new(
                "INPUT_MATERIAL_MISSING",
                format!("plan input {} has no canonical material", input.id),
            ));
        };
        let Some(path) = material_paths.get(material_id) else {
            return Err(CliError::new(
                "INPUT_BINDING_MISSING",
                format!("material {material_id} has no machine-local path"),
            ));
        };
        paths.insert(input.id.clone(), path.clone());
    }
    ExecutionBindings::from_originals(plan, &paths)
        .map_err(|error| CliError::new("EXECUTION_BINDINGS_FAILED", error.to_string()))
}

pub(crate) fn select_config(
    envelope: &veac_ir::ProjectEnvelope,
    requested: Option<&str>,
) -> CliResult<RenderConfigId> {
    if let Some(value) = requested {
        return RenderConfigId::new(value)
            .map_err(|error| CliError::new("INVALID_RENDER_CONFIG_ID", error.to_string()));
    }
    match envelope.project.render_configs.as_slice() {
        [] => Err(CliError::new(
            "RENDER_CONFIG_MISSING",
            "canonical project has no render config",
        )),
        [config] => Ok(config.id.clone()),
        _ => Err(CliError::new(
            "RENDER_CONFIG_REQUIRED",
            "project has multiple render configs; select one with --config",
        )),
    }
}
