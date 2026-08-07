use std::collections::{BTreeMap, BTreeSet};

use crate::*;

use super::{ChangeSet, MarkChanged};

#[derive(PartialEq, Eq)]
struct ProjectOwnership {
    multicam_groups: Vec<MulticamGroupId>,
    annotations: Vec<AnnotationId>,
    deliveries: Vec<RenderConfigId>,
}

#[derive(PartialEq, Eq)]
struct SequenceOwnership {
    tracks: Vec<TrackId>,
    relations: Vec<RelationId>,
    applies: Vec<ApplyId>,
}

pub(super) struct AuthorshipSnapshot {
    project: ProjectOwnership,
    sequences: BTreeMap<SequenceId, SequenceOwnership>,
    material_ids: BTreeSet<MaterialId>,
    clip_ids: BTreeSet<ItemId>,
    replaced_material: Option<(MaterialId, Material)>,
}

impl AuthorshipSnapshot {
    pub(super) fn capture(project: &Project, operation: &EditOperation) -> Self {
        Self {
            project: project_ownership(project),
            sequences: sequence_ownership(project),
            material_ids: project
                .materials
                .iter()
                .map(|material| material.id.clone())
                .collect(),
            clip_ids: project
                .sequences
                .iter()
                .flat_map(|sequence| &sequence.tracks)
                .flat_map(|track| &track.clips)
                .map(|clip| clip.id.clone())
                .collect(),
            replaced_material: replaced_material(project, operation),
        }
    }

    pub(super) fn reconcile(self, project: &mut Project, changed: &mut ChangeSet) {
        self.reconcile_project(project, changed);
        self.reconcile_sequences(project, changed);
        self.reconcile_materials(project, changed);
        self.reconcile_clips(project, changed);
    }

    fn reconcile_project(&self, project: &mut Project, changed: &mut ChangeSet) {
        if self.project != project_ownership(project) && project.authorship.take().is_some() {
            changed.project(project.id.clone());
        }
    }

    fn reconcile_sequences(&self, project: &mut Project, changed: &mut ChangeSet) {
        let current = sequence_ownership(project);
        for sequence in &mut project.sequences {
            if self.sequences.get(&sequence.id) != current.get(&sequence.id)
                && sequence.authorship.take().is_some()
            {
                changed.sequence(sequence.id.clone());
            }
        }
    }

    fn reconcile_materials(&self, project: &mut Project, changed: &mut ChangeSet) {
        for material in &mut project.materials {
            let inserted = !self.material_ids.contains(&material.id);
            let replaced = self
                .replaced_material
                .as_ref()
                .is_some_and(|(id, before)| id == &material.id && before != material);
            if (inserted || replaced) && material.authorship.take().is_some() {
                changed.material(material.id.clone());
            }
        }
    }

    fn reconcile_clips(&self, project: &mut Project, changed: &mut ChangeSet) {
        for clip in project
            .sequences
            .iter_mut()
            .flat_map(|sequence| &mut sequence.tracks)
            .flat_map(|track| &mut track.clips)
        {
            if !self.clip_ids.contains(&clip.id) && clip.authorship.take().is_some() {
                changed.item(clip.id.clone());
            }
        }
    }
}

fn project_ownership(project: &Project) -> ProjectOwnership {
    ProjectOwnership {
        multicam_groups: project
            .multicam_groups
            .iter()
            .map(|group| group.id.clone())
            .collect(),
        annotations: project
            .annotations
            .iter()
            .map(|annotation| annotation.id.clone())
            .collect(),
        deliveries: project
            .render_configs
            .iter()
            .map(|output| output.id.clone())
            .collect(),
    }
}

fn sequence_ownership(project: &Project) -> BTreeMap<SequenceId, SequenceOwnership> {
    project
        .sequences
        .iter()
        .map(|sequence| {
            let ownership = SequenceOwnership {
                tracks: sequence
                    .tracks
                    .iter()
                    .map(|track| track.id.clone())
                    .collect(),
                relations: project
                    .relations
                    .iter()
                    .filter(|relation| relation.sequence_id == sequence.id)
                    .map(|relation| relation.id.clone())
                    .collect(),
                applies: sequence
                    .applies
                    .iter()
                    .map(|apply| apply.id.clone())
                    .collect(),
            };
            (sequence.id.clone(), ownership)
        })
        .collect()
}

fn replaced_material(
    project: &Project,
    operation: &EditOperation,
) -> Option<(MaterialId, Material)> {
    let EditOperation::EditStructure {
        edit: StructureEdit::SetMaterial { material_id, .. },
    } = operation
    else {
        return None;
    };
    project
        .materials
        .iter()
        .find(|material| material.id == *material_id)
        .cloned()
        .map(|material| (material_id.clone(), material))
}

#[cfg(test)]
#[path = "provenance/tests.rs"]
mod tests;
