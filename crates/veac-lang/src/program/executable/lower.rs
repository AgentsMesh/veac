use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::DomainType;
use veac_ir::{Project, ProjectEnvelope};

mod animation;
mod annotation;
mod apply;
mod audio;
mod clip;
mod color;
mod defaults;
mod delivery;
mod effect;
mod error;
mod generator;
pub(in crate::program::executable) mod id;
mod manifest;
mod material;
mod metadata;
mod multicam;
mod relation;
mod sequence;
mod settings;
mod source_timing;
mod template;
mod temporal;
mod text;
mod time;
mod value;
mod visual;

pub(super) use error::ExecutableLowerError;

pub(super) fn project_with_inputs(
    graph: &FrozenDomainGraph,
    temporal_leaves: &[super::ExecutableTemporalLeaf],
    build_inputs: &crate::program::expression::ResidualBuildBindings,
    ledger: &crate::program::expression::ExecutionBudget,
) -> Result<ProjectEnvelope, ExecutableLowerError> {
    let root = graph.root_entity();
    if root.domain_type() != Some(DomainType::Project) {
        return Err(value::graph("the executable graph root is not a Project"));
    }
    let path = root
        .logical_path()
        .ok_or_else(|| value::graph("the executable Project root has no complete logical path"))?;
    let settings = settings::project(graph, root)?;
    let entry_key = graph
        .entry_key()
        .ok_or_else(|| value::graph("the executable Project has no verified entry sequence key"))?;
    let mut materials = Vec::new();
    let mut multicam_groups = Vec::new();
    let mut annotations = Vec::new();
    let mut render_configs = Vec::new();
    let mut sequences = Vec::new();
    let mut relations = Vec::new();
    let mut entry_sequence_id = None;
    for entity in root.children() {
        match entity.domain_type() {
            Some(DomainType::Resource) => materials.push(material::lower(graph, entity)?),
            Some(DomainType::Sequence) => {
                let lowered = sequence::lower(graph, entity, settings.timebase)?;
                let sequence = lowered.sequence;
                relations.extend(lowered.relations);
                if entity.key() == Some(entry_key) {
                    entry_sequence_id = Some(sequence.id.clone());
                }
                sequences.push(sequence);
            }
            Some(DomainType::MulticamGroup) => {
                multicam_groups.push(multicam::group(graph, entity, settings.timebase)?)
            }
            Some(DomainType::Annotation) => {
                annotations.push(annotation::lower(graph, entity, settings.timebase)?)
            }
            Some(DomainType::Delivery) => {
                render_configs.push(delivery::lower(graph, entity, settings.timebase)?)
            }
            _ => {
                return Err(value::graph(
                    "the executable Project owns an invalid entity type",
                ))
            }
        }
    }
    let entry_sequence_id = entry_sequence_id
        .ok_or_else(|| value::graph("the executable Project entry sequence is missing"))?;
    multicam_groups.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    annotations.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    let mut project = Project {
        id: id::project(&path),
        revision: 0,
        timebase: settings.timebase,
        entry_sequence_id,
        render_configs,
        materials,
        multicam_groups,
        annotations,
        relations,
        sequences,
        applied_operations: Vec::new(),
        authorship: Some(metadata::project(root)?),
    };
    let temporal = temporal::lower(graph, &mut project, temporal_leaves, build_inputs, ledger)?;
    let envelope = ProjectEnvelope::new(project, manifest::executable(graph)?, temporal);
    veac_ir::canonical_json(&envelope).map_err(ExecutableLowerError::ir)?;
    Ok(envelope)
}

#[cfg(test)]
#[path = "lower/tests.rs"]
mod tests;
