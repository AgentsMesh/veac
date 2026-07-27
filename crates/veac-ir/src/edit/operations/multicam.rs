use crate::*;

use crate::edit::{ensure_clip_unlocked, find_clip_mut, operation_error, ChangeSet, MarkChanged};

pub(super) fn apply(
    project: &mut Project,
    operation: &EditOperation,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    match operation {
        EditOperation::SetMulticamSwitches { clip_id, switches } => {
            ensure_clip_unlocked(project, clip_id)?;
            let clip = find_clip_mut(project, clip_id)
                .ok_or_else(|| operation_error(clip_id.as_str(), "clip does not exist"))?;
            let ClipSource::Multicam {
                switches: current, ..
            } = &mut clip.source
            else {
                return Err(operation_error(
                    clip_id.as_str(),
                    "multicam switch target is not a multicam clip",
                ));
            };
            if current != switches {
                *current = switches.clone();
                changed.item(clip_id.clone());
            }
            Ok(())
        }
        EditOperation::SetMulticamGroup {
            group_id,
            sync,
            angles,
        } => set_group(project, group_id, sync, angles, changed),
        _ => unreachable!("multicam dispatcher received another operation"),
    }
}

fn set_group(
    project: &mut Project,
    group_id: &MulticamGroupId,
    sync: &MulticamSync,
    angles: &[MulticamAngle],
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    if project
        .sequences
        .iter()
        .flat_map(|value| &value.tracks)
        .any(|track| {
            track.state.locked
            && track.clips.iter().any(|clip| {
                matches!(&clip.source, ClipSource::Multicam { group_id: id, .. } if id == group_id)
            })
        })
    {
        return Err(operation_error(
            group_id.as_str(),
            "multicam group is used by a clip on a locked track",
        ));
    }
    let group = project
        .multicam_groups
        .iter_mut()
        .find(|group| group.id == *group_id)
        .ok_or_else(|| operation_error(group_id.as_str(), "multicam group does not exist"))?;
    if group.sync != *sync || group.angles != angles {
        group.sync = sync.clone();
        group.angles = angles.to_vec();
        changed.multicam_group(group_id.clone());
    }
    Ok(())
}
