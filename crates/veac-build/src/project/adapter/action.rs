use std::collections::BTreeMap;

use veac_project::{ProjectPath, ProjectTargetEntry, ResolvedInputSource, TargetInstance};

use super::ProjectGraphAdapter;
use crate::{
    BuildResult, ProjectAction, ProjectBoundSource, ProjectComputation, ProjectFileSnapshot,
    ProjectSourceGraphRevision, ProjectSourceRole,
};

type CapturedGraph = (ProjectFileSnapshot, ProjectSourceGraphRevision);

#[derive(Default)]
pub(super) struct ActionSources {
    renders: BTreeMap<ProjectPath, CapturedGraph>,
    evidence: BTreeMap<ProjectPath, CapturedGraph>,
}

impl ActionSources {
    fn render(
        &mut self,
        adapter: &ProjectGraphAdapter,
        path: &ProjectPath,
    ) -> BuildResult<CapturedGraph> {
        capture_once(&mut self.renders, path, || adapter.roots.source_graph(path))
    }

    fn evidence(
        &mut self,
        adapter: &ProjectGraphAdapter,
        path: &ProjectPath,
    ) -> BuildResult<CapturedGraph> {
        capture_once(&mut self.evidence, path, || {
            adapter.roots.evidence_graph(path)
        })
    }
}

impl ProjectGraphAdapter {
    pub(super) fn action(
        &self,
        instance: &TargetInstance,
        sources: &mut ActionSources,
    ) -> BuildResult<ProjectAction> {
        let computation = ProjectComputation {
            instance: instance.id.clone(),
            target: instance.target.clone(),
            profile: instance.profile.clone(),
            locale: instance.locale.clone(),
            matrix: instance.matrix.clone(),
            inputs: instance.inputs.clone(),
            outputs: instance.outputs.clone(),
            bound_sources: self.bound_sources(instance)?,
            package_mounts: self.roots.package_revision(),
        };
        Ok(match &instance.entry {
            ProjectTargetEntry::Veac { source } => {
                let (source, source_graph) = sources.render(self, source)?;
                ProjectAction::VeacRender {
                    computation,
                    source,
                    source_graph,
                }
            }
            ProjectTargetEntry::MediaDerivation { operation } => ProjectAction::MediaDerivation {
                computation,
                operation: operation.clone(),
            },
            ProjectTargetEntry::Evidence { contract } => {
                let (contract, source_graph) = sources.evidence(self, contract)?;
                ProjectAction::Evidence {
                    computation,
                    contract,
                    source_graph,
                }
            }
        })
    }

    fn bound_sources(&self, instance: &TargetInstance) -> BuildResult<Vec<ProjectBoundSource>> {
        instance
            .inputs
            .iter()
            .filter_map(|input| match &input.source {
                ResolvedInputSource::ProjectMaterial { path } => {
                    Some((input.id.clone(), ProjectSourceRole::Material, path))
                }
                ResolvedInputSource::AssetFact { path, .. } => {
                    Some((input.id.clone(), ProjectSourceRole::AssetFact, path))
                }
                _ => None,
            })
            .map(|(input, role, path)| {
                Ok(ProjectBoundSource {
                    input,
                    role,
                    snapshot: self.roots.material(path)?,
                })
            })
            .collect()
    }
}

fn capture_once(
    values: &mut BTreeMap<ProjectPath, CapturedGraph>,
    path: &ProjectPath,
    capture: impl FnOnce() -> BuildResult<CapturedGraph>,
) -> BuildResult<CapturedGraph> {
    if let Some(value) = values.get(path) {
        return Ok(value.clone());
    }
    let value = capture()?;
    values.insert(path.clone(), value.clone());
    Ok(value)
}

#[cfg(test)]
#[path = "action/tests.rs"]
mod tests;
