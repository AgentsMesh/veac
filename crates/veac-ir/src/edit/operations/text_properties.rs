use crate::*;

use super::changed_tree;
use crate::edit::{ensure_clip_unlocked, find_clip_mut, operation_error, ChangeSet, MarkChanged};

pub(super) fn apply(
    project: &mut Project,
    clip_id: &ItemId,
    property: &TextProperty,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    ensure_clip_unlocked(project, clip_id)?;
    let clip = find_clip_mut(project, clip_id)
        .ok_or_else(|| operation_error(clip_id.as_str(), "clip does not exist"))?;
    let style = match &mut clip.source {
        ClipSource::Text { style, .. } | ClipSource::Caption { style, .. } => style,
        _ => return Err(operation_error(clip_id.as_str(), "clip is not text")),
    };
    let did_change = match property {
        TextProperty::Font(value) => replace(&mut style.font, value.clone()),
        TextProperty::FallbackFonts(value) => replace(&mut style.fallback_fonts, value.clone()),
        TextProperty::FontWeight(value) => replace(&mut style.font_weight, *value),
        TextProperty::FontStyle(value) => replace(&mut style.font_style, *value),
        TextProperty::SizePixels(value) => replace(&mut style.size_pixels, *value),
        TextProperty::Color(value) => replace(&mut style.color, *value),
        TextProperty::TrackingPixels(value) => replace(&mut style.tracking_pixels, *value),
        TextProperty::LineHeight(value) => replace(&mut style.line_height, *value),
        TextProperty::Layout(value) => replace(&mut style.layout, *value),
        TextProperty::WritingMode(value) => replace(&mut style.layout.writing_mode, *value),
        TextProperty::Orientation(value) => replace(&mut style.layout.orientation, *value),
        TextProperty::Path(value) => replace(&mut style.path, value.clone()),
        TextProperty::Background(value) => replace(&mut style.background, value.clone()),
        TextProperty::Outline(value) => replace(&mut style.outline, value.clone()),
        TextProperty::Shadow(value) => replace(&mut style.shadow, value.clone()),
        TextProperty::Spans(value) => replace(&mut style.spans, value.clone()),
        TextProperty::Animation(value) => animation(&mut style.animation, value, changed),
    };
    if did_change {
        changed.item(clip_id.clone());
    }
    Ok(())
}

fn animation(
    target: &mut Option<TextAnimation>,
    value: &Option<TextAnimation>,
    changed: &mut ChangeSet,
) -> bool {
    if target == value {
        return false;
    }
    for animation in target.iter().chain(value) {
        changed_tree::curve(&animation.reveal, changed);
        changed_tree::curve(&animation.opacity, changed);
        changed_tree::curve(&animation.transform.position_offset, changed);
        changed_tree::curve(&animation.transform.scale, changed);
        changed_tree::curve(&animation.transform.rotation_degrees, changed);
    }
    *target = value.clone();
    true
}

fn replace<T: PartialEq>(target: &mut T, value: T) -> bool {
    if *target == value {
        false
    } else {
        *target = value;
        true
    }
}
