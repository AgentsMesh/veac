use crate::*;

use super::slice::crop_animatable;
use crate::edit::ChangeSet;

pub(super) fn crop(
    visual: &mut VisualProperties,
    start: RationalTime,
    end: RationalTime,
    owner: &ItemId,
    reidentify: bool,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    for (index, mask) in visual.masks.iter_mut().enumerate() {
        let prefix = format!("mask_{index}");
        crop_animatable(
            &mut mask.position,
            start,
            end,
            owner,
            &format!("{prefix}_position"),
            reidentify,
            changed,
        )?;
        crop_animatable(
            &mut mask.scale,
            start,
            end,
            owner,
            &format!("{prefix}_scale"),
            reidentify,
            changed,
        )?;
        crop_animatable(
            &mut mask.rotation_degrees,
            start,
            end,
            owner,
            &format!("{prefix}_rotation"),
            reidentify,
            changed,
        )?;
        crop_animatable(
            &mut mask.feather_pixels,
            start,
            end,
            owner,
            &format!("{prefix}_feather"),
            reidentify,
            changed,
        )?;
        crop_animatable(
            &mut mask.expansion_pixels,
            start,
            end,
            owner,
            &format!("{prefix}_expansion"),
            reidentify,
            changed,
        )?;
    }
    Ok(())
}
