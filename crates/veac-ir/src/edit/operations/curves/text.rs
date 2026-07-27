use crate::*;

use super::slice::crop_animatable;
use crate::edit::ChangeSet;

pub(super) fn crop(
    clip: &mut Clip,
    start: RationalTime,
    end: RationalTime,
    owner: &ItemId,
    reidentify: bool,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let (ClipSource::Text { style, .. } | ClipSource::Caption { style, .. }) = &mut clip.source
    else {
        return Ok(());
    };
    let Some(animation) = &mut style.animation else {
        return Ok(());
    };
    for (curve, path) in [
        (&mut animation.reveal, "text_reveal"),
        (&mut animation.opacity, "text_opacity"),
        (&mut animation.transform.rotation_degrees, "text_rotation"),
    ] {
        crop_animatable(curve, start, end, owner, path, reidentify, changed)?;
    }
    crop_animatable(
        &mut animation.transform.position_offset,
        start,
        end,
        owner,
        "text_position_offset",
        reidentify,
        changed,
    )?;
    crop_animatable(
        &mut animation.transform.scale,
        start,
        end,
        owner,
        "text_scale",
        reidentify,
        changed,
    )
}
