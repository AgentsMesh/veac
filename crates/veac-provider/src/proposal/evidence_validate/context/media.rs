use veac_ir::{EditOperation, MaterialId, StructureEdit};

use crate::{ApplicationContext, AudioClipInsertion, MaterialInsertion, MatteClipInsertion};

pub(super) fn material(
    application: &ApplicationContext,
    operation: &EditOperation,
    material_id: &MaterialId,
) -> bool {
    let Some(insertion) = material_context(application, material_id) else {
        return false;
    };
    matches!(
        operation,
        EditOperation::EditStructure {
            edit: StructureEdit::InsertMaterial { material, before_id, after_id }
        } if material.as_ref() == &insertion.material
            && *before_id == insertion.before_id
            && *after_id == insertion.after_id
    )
}

pub(super) fn generated_audio(
    application: &ApplicationContext,
    operation: &EditOperation,
    material_id: &MaterialId,
) -> bool {
    let insertion = match application {
        ApplicationContext::TextToSpeech(value) => &value.output,
        ApplicationContext::Dubbing(value) => &value.output,
        _ => return false,
    };
    insertion.material.material.id == *material_id && inserted(operation, insertion)
}

pub(super) fn replacement(
    application: &ApplicationContext,
    operation: &EditOperation,
    material_id: &MaterialId,
) -> bool {
    let context = match application {
        ApplicationContext::Denoise(value) | ApplicationContext::Removal(value) => value.as_ref(),
        _ => return false,
    };
    context.material.material.id == *material_id
        && matches!(operation, EditOperation::ReplaceSource { clip_id, .. } if *clip_id == context.clip_id)
}

pub(super) fn stem(
    application: &ApplicationContext,
    operation: &EditOperation,
    material_id: &MaterialId,
) -> bool {
    let ApplicationContext::VocalSeparation(context) = application else {
        return false;
    };
    context
        .bindings
        .iter()
        .find(|binding| binding.output.material.material.id == *material_id)
        .is_some_and(|binding| inserted(operation, &binding.output))
}

pub(super) fn visual_clip(
    application: &ApplicationContext,
    operation: &EditOperation,
    material_id: &MaterialId,
) -> bool {
    let insertion = match application {
        ApplicationContext::Segmentation(value) | ApplicationContext::Matte(value) => &value.matte,
        ApplicationContext::Retouch(value) => {
            let Some(matte) = &value.matte else {
                return false;
            };
            &matte.insertion
        }
        _ => return false,
    };
    insertion.material.material.id == *material_id && inserted_visual(operation, insertion)
}

fn material_context<'a>(
    application: &'a ApplicationContext,
    material_id: &MaterialId,
) -> Option<&'a MaterialInsertion> {
    let direct = match application {
        ApplicationContext::TextToSpeech(value) => Some(&value.output.material),
        ApplicationContext::Dubbing(value) => Some(&value.output.material),
        ApplicationContext::Denoise(value) | ApplicationContext::Removal(value) => {
            Some(&value.material)
        }
        ApplicationContext::Segmentation(value) | ApplicationContext::Matte(value) => {
            Some(&value.matte.material)
        }
        ApplicationContext::Retouch(value) => {
            value.matte.as_ref().map(|matte| &matte.insertion.material)
        }
        _ => None,
    };
    direct
        .filter(|value| value.material.id == *material_id)
        .or_else(|| stem_material(application, material_id))
}

fn stem_material<'a>(
    application: &'a ApplicationContext,
    material_id: &MaterialId,
) -> Option<&'a MaterialInsertion> {
    let ApplicationContext::VocalSeparation(context) = application else {
        return None;
    };
    context
        .bindings
        .iter()
        .map(|binding| &binding.output.material)
        .find(|value| value.material.id == *material_id)
}

fn inserted(operation: &EditOperation, value: &AudioClipInsertion) -> bool {
    matches!(
        operation,
        EditOperation::InsertClip {
            sequence_id, track_id, clip, before_id, after_id
        } if *sequence_id == value.sequence_id
            && *track_id == value.track_id
            && clip.id == value.clip_id
            && *before_id == value.before_id
            && *after_id == value.after_id
    )
}

fn inserted_visual(operation: &EditOperation, value: &MatteClipInsertion) -> bool {
    matches!(
        operation,
        EditOperation::InsertClip {
            sequence_id, track_id, clip, before_id, after_id
        } if *sequence_id == value.sequence_id
            && *track_id == value.track_id
            && clip.id == value.clip_id
            && *before_id == value.before_id
            && *after_id == value.after_id
    )
}
