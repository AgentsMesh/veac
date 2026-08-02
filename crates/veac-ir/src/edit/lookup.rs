use crate::*;

use super::operation_error;

pub(super) fn find_track<'a>(project: &'a Project, id: &TrackId) -> Option<&'a Track> {
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .find(|track| track.id == *id)
}

pub(super) fn find_clip<'a>(project: &'a Project, id: &ItemId) -> Option<&'a Clip> {
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
        .find(|clip| clip.id == *id)
}

pub(super) fn find_clip_mut<'a>(project: &'a mut Project, id: &ItemId) -> Option<&'a mut Clip> {
    project
        .sequences
        .iter_mut()
        .flat_map(|sequence| &mut sequence.tracks)
        .flat_map(|track| &mut track.clips)
        .find(|clip| clip.id == *id)
}

pub(super) fn find_apply<'a>(project: &'a Project, id: &ApplyId) -> Option<&'a Apply> {
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.applies)
        .find(|apply| apply.id == *id)
}

pub(super) fn find_clip_track_mut<'a>(
    project: &'a mut Project,
    id: &ItemId,
) -> Option<&'a mut Track> {
    project
        .sequences
        .iter_mut()
        .flat_map(|sequence| &mut sequence.tracks)
        .find(|track| track.clips.iter().any(|clip| clip.id == *id))
}

pub(super) fn ensure_clip_unlocked(project: &Project, id: &ItemId) -> Result<(), Diagnostic> {
    match project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .find(|track| track.clips.iter().any(|clip| clip.id == *id))
    {
        None => Err(operation_error(id.as_str(), "clip does not exist")),
        Some(track) if track.state.locked => Err(operation_error(
            id.as_str(),
            "clip belongs to a locked track",
        )),
        Some(_) => Ok(()),
    }
}
