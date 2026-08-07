use crate::*;

use crate::edit::{ChangeSet, MarkChanged};

pub(super) fn clip(clip: &Clip, changed: &mut ChangeSet) {
    changed.item(clip.id.clone());
    if let Some(visual) = &clip.visual {
        curve(&visual.transform.position, changed);
        curve(&visual.transform.scale, changed);
        curve(&visual.transform.rotation_degrees, changed);
        if let Some(crop) = &visual.transform.crop {
            curve(crop, changed);
        }
        curve(&visual.opacity, changed);
        for mask in &visual.masks {
            self::mask(mask, changed);
        }
    }
    if let Some(audio) = &clip.audio {
        curve(&audio.gain, changed);
        curve(&audio.pan, changed);
    }
    if let ClipSource::Text { style, .. } | ClipSource::Caption { style, .. } = &clip.source {
        if let Some(animation) = &style.animation {
            curve(&animation.reveal, changed);
            if let Some(highlight) = &animation.highlight {
                curve(&highlight.progress, changed);
            }
            curve(&animation.opacity, changed);
            curve(&animation.transform.position_offset, changed);
            curve(&animation.transform.scale, changed);
            curve(&animation.transform.rotation_degrees, changed);
        }
    }
    for value in &clip.effects {
        effect(value, changed);
    }
}

pub(super) fn track(track: &Track, changed: &mut ChangeSet) {
    changed.track(track.id.clone());
    for item in &track.clips {
        clip(item, changed);
    }
}

pub(super) fn sequence(sequence: &Sequence, changed: &mut ChangeSet) {
    changed.sequence(sequence.id.clone());
    for value in &sequence.tracks {
        track(value, changed);
    }
    for value in &sequence.applies {
        apply(value, changed);
    }
}

pub(super) fn apply(value: &Apply, changed: &mut ChangeSet) {
    changed.apply(value.id.clone());
    for stage in &value.stages {
        if let ApplyOperation::Effect { effect } = &stage.operation {
            self::effect(effect, changed);
        }
    }
    curve(&value.mix.opacity, changed);
    for mask in &value.mix.masks {
        self::mask(mask, changed);
    }
}

pub(super) fn effect(effect: &EffectInstance, changed: &mut ChangeSet) {
    changed.effect(effect.id.clone());
    for parameter in EffectParameter::ALL {
        if let Some(value) = effect.effect.curve(parameter) {
            curve(value, changed);
        }
    }
}

pub(super) fn visual(value: &VisualProperties, changed: &mut ChangeSet) {
    curve(&value.transform.position, changed);
    curve(&value.transform.scale, changed);
    curve(&value.transform.rotation_degrees, changed);
    if let Some(crop) = &value.transform.crop {
        curve(crop, changed);
    }
    curve(&value.opacity, changed);
    for value in &value.masks {
        mask(value, changed);
    }
}

pub(super) fn audio(value: &AudioProperties, changed: &mut ChangeSet) {
    curve(&value.gain, changed);
    curve(&value.pan, changed);
}

pub(super) fn mask(value: &Mask, changed: &mut ChangeSet) {
    curve(&value.position, changed);
    curve(&value.scale, changed);
    curve(&value.rotation_degrees, changed);
    curve(&value.feather_pixels, changed);
    curve(&value.expansion_pixels, changed);
}

pub(super) fn curve<T>(value: &Animatable<T>, changed: &mut ChangeSet) {
    if let Animatable::Keyframes { keyframes } = value {
        for keyframe in keyframes {
            changed.keyframe(keyframe.id.clone());
        }
    }
}
