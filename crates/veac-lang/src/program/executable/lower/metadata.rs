use crate::program::expression::runtime::domain_graph::FrozenEntity;
use crate::program::DomainType;
use veac_ir::{
    AnnotationAuthorship, ApplyAuthorship, DeliveryAuthorship, EntityAuthorship,
    MulticamAuthorship, ProjectAuthorship, RelationAuthorship, SequenceAuthorship, TrackAuthorship,
};

use super::error::ExecutableLowerError;

pub(super) fn project(root: FrozenEntity<'_>) -> Result<ProjectAuthorship, ExecutableLowerError> {
    let mut multicam_groups = Vec::new();
    let mut annotations = Vec::new();
    let mut deliveries = Vec::new();
    for child in root.children() {
        let path = child.logical_path().ok_or_else(missing)?;
        match child.domain_type() {
            Some(DomainType::MulticamGroup) => multicam_groups.push(MulticamAuthorship {
                group_id: super::id::multicam(&path),
                entity: entity(child)?,
            }),
            Some(DomainType::Annotation) => annotations.push(AnnotationAuthorship {
                annotation_id: super::id::annotation(&path),
                entity: entity(child)?,
            }),
            Some(DomainType::Delivery) => deliveries.push(DeliveryAuthorship {
                render_config_id: super::id::delivery(&path),
                entity: entity(child)?,
            }),
            _ => {}
        }
    }
    multicam_groups.sort_by(|left, right| left.group_id.cmp(&right.group_id));
    annotations.sort_by(|left, right| left.annotation_id.cmp(&right.annotation_id));
    deliveries.sort_by(|left, right| left.render_config_id.cmp(&right.render_config_id));
    Ok(ProjectAuthorship {
        entity: entity(root)?,
        multicam_groups,
        annotations,
        deliveries,
    })
}

pub(super) fn entity(value: FrozenEntity<'_>) -> Result<EntityAuthorship, ExecutableLowerError> {
    value.provenance().ok_or_else(missing)
}

pub(super) fn sequence(
    sequence: FrozenEntity<'_>,
) -> Result<SequenceAuthorship, ExecutableLowerError> {
    let mut tracks = Vec::new();
    let mut relations = Vec::new();
    let mut applies = Vec::new();
    for child in sequence.children() {
        let path = child.logical_path().ok_or_else(missing)?;
        match child.domain_type() {
            Some(DomainType::Layer) => tracks.push(TrackAuthorship {
                track_id: super::id::track(&path),
                entity: entity(child)?,
            }),
            Some(DomainType::Relation) => relations.push(RelationAuthorship {
                relation_id: super::id::relation(&path),
                entity: entity(child)?,
            }),
            Some(DomainType::Apply) => applies.push(ApplyAuthorship {
                apply_id: super::id::apply(&path),
                entity: entity(child)?,
            }),
            _ => return Err(missing()),
        }
    }
    tracks.sort_by(|left, right| left.track_id.cmp(&right.track_id));
    relations.sort_by(|left, right| left.relation_id.cmp(&right.relation_id));
    applies.sort_by(|left, right| left.apply_id.cmp(&right.apply_id));
    Ok(SequenceAuthorship::Veac {
        entity: entity(sequence)?,
        tracks,
        relations,
        applies,
    })
}

fn missing() -> ExecutableLowerError {
    ExecutableLowerError::lower(
        "EXECUTABLE_LOWER_PROVENANCE",
        "the executable graph is missing canonical authored provenance",
    )
}
