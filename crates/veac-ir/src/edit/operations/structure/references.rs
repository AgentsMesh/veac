use crate::*;

#[cfg(test)]
mod tests;

pub(super) fn clip_uses_material(clip: &Clip, id: &MaterialId) -> bool {
    matches!(
        &clip.source,
        ClipSource::Media { material_id } | ClipSource::FreezeFrame { material_id, .. }
            if material_id == id
    ) || clip.source.font_material() == Some(id)
}

pub(super) fn material_is_referenced(project: &Project, id: &MaterialId) -> bool {
    project
        .multicam_groups
        .iter()
        .flat_map(|group| &group.angles)
        .any(|angle| angle.material_id == *id)
        || clips(project).any(|(_, clip)| clip_uses_material(clip, id))
}

pub(super) fn material_is_used_on_locked_track(project: &Project, id: &MaterialId) -> bool {
    clips(project).any(|(track, clip)| {
        track.state.locked
            && (clip_uses_material(clip, id) || locked_multicam_uses(project, clip, id))
    })
}

fn locked_multicam_uses(project: &Project, clip: &Clip, id: &MaterialId) -> bool {
    let ClipSource::Multicam { group_id, .. } = &clip.source else {
        return false;
    };
    project
        .multicam_groups
        .iter()
        .find(|group| group.id == *group_id)
        .is_some_and(|group| group.angles.iter().any(|angle| angle.material_id == *id))
}

pub(super) fn sequence_is_referenced(project: &Project, id: &SequenceId) -> bool {
    project.entry_sequence_id == *id
        || project
            .render_configs
            .iter()
            .any(|output| output.sequence_id == *id)
        || clips(project).any(|(_, clip)| {
            matches!(&clip.source, ClipSource::Sequence { sequence_id } if sequence_id == id)
        })
}

pub(super) fn multicam_group_is_referenced(project: &Project, id: &MulticamGroupId) -> bool {
    clips(project).any(
        |(_, clip)| matches!(&clip.source, ClipSource::Multicam { group_id, .. } if group_id == id),
    )
}

pub(super) fn multicam_group_is_used_on_locked_track(
    project: &Project,
    id: &MulticamGroupId,
) -> bool {
    clips(project).any(|(track, clip)| {
        track.state.locked
            && matches!(&clip.source, ClipSource::Multicam { group_id, .. } if group_id == id)
    })
}

fn clips(project: &Project) -> impl Iterator<Item = (&Track, &Clip)> {
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| track.clips.iter().map(move |clip| (track, clip)))
}
