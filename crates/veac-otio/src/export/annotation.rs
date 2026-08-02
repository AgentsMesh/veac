use std::collections::BTreeSet;

use veac_ir::{Annotation, AnnotationTarget, Project, Sequence};

use crate::OtioLossReport;

pub(super) fn select(
    project: &Project,
    sequence: &Sequence,
    losses: &mut OtioLossReport,
) -> Vec<Annotation> {
    let track_ids = sequence
        .tracks
        .iter()
        .map(|track| track.id.as_str())
        .collect::<BTreeSet<_>>();
    let clip_ids = sequence
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .map(|clip| clip.id.as_str())
        .collect::<BTreeSet<_>>();
    let selected = project
        .annotations
        .iter()
        .filter(|annotation| {
            let included = target_in_export(annotation, sequence, &track_ids, &clip_ids);
            if !included {
                losses.push(
                    format!("/project/annotations/{}", annotation.id),
                    "target",
                    "annotation target is outside the exported sequence graph",
                    false,
                );
            }
            included
        })
        .cloned()
        .collect::<Vec<_>>();
    if !selected.is_empty() {
        losses.push(
            "",
            "annotations",
            "standard OTIO has no exact projection for typed VEAC annotations",
            true,
        );
    }
    selected
}

fn target_in_export(
    annotation: &Annotation,
    sequence: &Sequence,
    tracks: &BTreeSet<&str>,
    clips: &BTreeSet<&str>,
) -> bool {
    match &annotation.target {
        AnnotationTarget::Project
        | AnnotationTarget::Material { .. }
        | AnnotationTarget::MulticamGroup { .. } => true,
        AnnotationTarget::Sequence { sequence_id } => sequence_id == &sequence.id,
        AnnotationTarget::Track { track_id } => tracks.contains(track_id.as_str()),
        AnnotationTarget::Clip { clip_id } => clips.contains(clip_id.as_str()),
    }
}
