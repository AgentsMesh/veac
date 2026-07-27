use std::collections::BTreeSet;

use veac_ir::AnnotationTarget;

use crate::{export::extension, OtioError, OtioImportResult, OtioLossReport, OtioTimeline};

pub(super) fn import(timeline: &OtioTimeline) -> Result<OtioImportResult, OtioError> {
    let value = timeline
        .metadata
        .get(extension::KEY)
        .ok_or_else(|| OtioError::contract("timeline has no VEAC extension"))?;
    let extension = extension::decode(value)?;
    extension::verify(timeline, &extension)?;
    unique(
        extension.materials.iter().map(|item| item.id.as_str()),
        "material",
    )?;
    unique(
        extension
            .multicam_groups
            .iter()
            .map(|item| item.id.as_str()),
        "multicam group",
    )?;
    ordered_annotations(&extension.annotations)?;
    annotation_targets(&extension)?;
    veac_ir::validate_sequence_relations(&extension.sequence, &extension.relations)
        .map_err(|errors| OtioError::contract(format!("invalid VEAC relation graph: {errors}")))?;
    Ok(OtioImportResult {
        sequence: extension.sequence,
        relations: extension.relations,
        materials: extension.materials,
        multicam_groups: extension.multicam_groups,
        annotations: extension.annotations,
        loss_report: OtioLossReport::default(),
    })
}

fn annotation_targets(value: &extension::VeacExtension) -> Result<(), OtioError> {
    let tracks = value
        .sequence
        .tracks
        .iter()
        .map(|track| track.id.as_str())
        .collect::<BTreeSet<_>>();
    let clips = value
        .sequence
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .map(|clip| clip.id.as_str())
        .collect::<BTreeSet<_>>();
    let materials = value
        .materials
        .iter()
        .map(|material| material.id.as_str())
        .collect::<BTreeSet<_>>();
    let groups = value
        .multicam_groups
        .iter()
        .map(|group| group.id.as_str())
        .collect::<BTreeSet<_>>();
    let valid = value
        .annotations
        .iter()
        .all(|annotation| match &annotation.target {
            AnnotationTarget::Project => true,
            AnnotationTarget::Sequence { sequence_id } => sequence_id == &value.sequence.id,
            AnnotationTarget::Track { track_id } => tracks.contains(track_id.as_str()),
            AnnotationTarget::Clip { clip_id } => clips.contains(clip_id.as_str()),
            AnnotationTarget::Material { material_id } => materials.contains(material_id.as_str()),
            AnnotationTarget::MulticamGroup { group_id } => groups.contains(group_id.as_str()),
        });
    if valid {
        Ok(())
    } else {
        Err(OtioError::contract(
            "VEAC extension annotation target is outside the imported graph",
        ))
    }
}

fn ordered_annotations(values: &[veac_ir::Annotation]) -> Result<(), OtioError> {
    if values
        .iter()
        .any(|value| veac_ir::AnnotationId::new(value.id.as_str()).is_err())
        || values.windows(2).any(|pair| pair[0].id >= pair[1].id)
    {
        Err(OtioError::contract(
            "VEAC extension annotations must have valid unique sorted IDs",
        ))
    } else {
        Ok(())
    }
}

fn unique<'a>(mut values: impl Iterator<Item = &'a str>, role: &str) -> Result<(), OtioError> {
    let mut seen = BTreeSet::new();
    if values.any(|value| !seen.insert(value)) {
        Err(OtioError::contract(format!(
            "VEAC extension contains a duplicate {role} ID"
        )))
    } else {
        Ok(())
    }
}
