use std::collections::BTreeSet;

use veac_ir::{
    BlendMode, MaskShape, ProjectEnvelope, RelationKind, TrackMatteMode, TransitionKind,
};

use crate::support::{assert_preview_evidence, clips};

#[test]
fn preview_composition_rows_have_typed_variants_in_their_target() {
    assert_preview_evidence("transitions-composition.json", evidence);
}

fn evidence(envelope: &ProjectEnvelope) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for clip in clips(envelope) {
        if let Some(visual) = &clip.visual {
            found.insert(blend_id(visual.compositing.blend_mode).to_owned());
            found.extend(
                visual
                    .masks
                    .iter()
                    .map(|mask| mask_id(&mask.shape).to_owned()),
            );
        }
    }
    for relation in &envelope.project.relations {
        match &relation.kind {
            RelationKind::Transition { transition, .. } => {
                found.insert(transition_id(&transition.kind).to_owned());
            }
            RelationKind::Matte { parameters, .. } => {
                found.insert(matte_id(parameters.mode).to_owned());
                if parameters.invert {
                    found.insert("matte.invert".to_owned());
                }
            }
            _ => {}
        }
    }
    found
}

fn transition_id(kind: &TransitionKind) -> &'static str {
    match kind {
        TransitionKind::Dissolve => "transition.dissolve",
        TransitionKind::Fade { .. } => "transition.fade",
        TransitionKind::Wipe { .. } => "transition.wipe",
        TransitionKind::Slide { .. } => "transition.slide",
        TransitionKind::Zoom { .. } => "transition.zoom",
        TransitionKind::Circle { .. } => "transition.circle",
        TransitionKind::Pixelize { .. } => "transition.pixelize",
    }
}

fn blend_id(mode: BlendMode) -> &'static str {
    match mode {
        BlendMode::Normal => "blend.normal",
        BlendMode::Multiply => "blend.multiply",
        BlendMode::Screen => "blend.screen",
        BlendMode::Overlay => "blend.overlay",
        BlendMode::Darken => "blend.darken",
        BlendMode::Lighten => "blend.lighten",
        BlendMode::ColorDodge => "blend.color-dodge",
        BlendMode::ColorBurn => "blend.color-burn",
        BlendMode::HardLight => "blend.hard-light",
        BlendMode::SoftLight => "blend.soft-light",
        BlendMode::Difference => "blend.difference",
        BlendMode::Exclusion => "blend.exclusion",
    }
}

fn mask_id(shape: &MaskShape) -> &'static str {
    match shape {
        MaskShape::Linear => "mask.linear",
        MaskShape::Mirror => "mask.mirror",
        MaskShape::Circle => "mask.circle",
        MaskShape::Rectangle => "mask.rectangle",
        MaskShape::RoundedRectangle { .. } => "mask.rounded-rectangle",
        MaskShape::Ellipse => "mask.ellipse",
        MaskShape::Polygon { .. } => "mask.polygon",
        MaskShape::Heart => "mask.heart",
        MaskShape::Star => "mask.star",
        MaskShape::Path { .. } => "mask.path",
    }
}

fn matte_id(mode: TrackMatteMode) -> &'static str {
    match mode {
        TrackMatteMode::Alpha => "matte.alpha",
        TrackMatteMode::Luma => "matte.luma",
    }
}
