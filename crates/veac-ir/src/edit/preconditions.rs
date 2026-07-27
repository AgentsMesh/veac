use crate::*;

use super::{diagnostics::precondition_error, find_apply, find_clip, find_track};

pub(super) fn check_preconditions(
    project: &Project,
    preconditions: &[Precondition],
) -> Result<(), Diagnostic> {
    for precondition in preconditions {
        let satisfied = match precondition {
            Precondition::ClipExists { clip_id } => find_clip(project, clip_id).is_some(),
            Precondition::ClipSourceEquals { clip_id, source } => {
                find_clip(project, clip_id).map(|clip| &clip.source) == Some(source.as_ref())
            }
            Precondition::TrackUnlocked { track_id } => {
                find_track(project, track_id).is_some_and(|track| !track.state.locked)
            }
            Precondition::ApplyExists { apply_id } => find_apply(project, apply_id).is_some(),
            Precondition::ApplyEquals { apply_id, apply } => {
                find_apply(project, apply_id) == Some(apply.as_ref())
            }
        };
        if !satisfied {
            let id = match precondition {
                Precondition::ClipExists { clip_id }
                | Precondition::ClipSourceEquals { clip_id, .. } => clip_id.as_str(),
                Precondition::TrackUnlocked { track_id } => track_id.as_str(),
                Precondition::ApplyExists { apply_id }
                | Precondition::ApplyEquals { apply_id, .. } => apply_id.as_str(),
            };
            return Err(precondition_error(id));
        }
    }
    Ok(())
}
