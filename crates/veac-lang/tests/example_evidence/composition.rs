use std::collections::BTreeSet;

use veac_ir::{
    Animatable, BlendMode, ClipSource, MaskShape, Placement, ProjectEnvelope, RelationKind,
    TrackMatteMode, TransitionKind,
};

use crate::support::{assert_preview_evidence, clips, lower_example};

#[test]
fn preview_composition_rows_have_typed_variants_in_their_target() {
    assert_preview_evidence("transitions-composition.json", evidence);
}

#[test]
fn image_overlay_keeps_the_asset_inside_its_bottom_right_inset() {
    let envelope = crate::support::lower_example("image-overlay/main.veac");
    let logo = envelope
        .project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
        .find(|clip| clip.id.as_str() == "itm_logo")
        .expect("image-overlay logo clip");
    let visual = logo.visual.as_ref().expect("logo visual properties");
    let frame = visual.frame.as_ref().expect("logo fit frame");
    assert_eq!(frame.width.value, 320.0);
    assert_eq!(frame.height.value, 320.0);
    assert_eq!(visual.transform.anchor.x, 1.0);
    assert_eq!(visual.transform.anchor.y, 1.0);
    assert!(matches!(
        visual.transform.scale,
        Animatable::Constant { ref value } if value.x == 0.75 && value.y == 0.75
    ));
    assert!(matches!(
        visual.placement,
        Placement::Anchor { inset, .. } if inset.x == 48.0 && inset.y == 48.0
    ));
}

#[test]
fn blend_gallery_changes_only_the_mode_between_segments() {
    let envelope = lower_example("blend-modes/main.veac");
    let overlays: Vec<_> = clips(&envelope)
        .filter(|clip| matches!(clip.source, ClipSource::Sequence { .. }))
        .collect();
    assert_eq!(overlays.len(), 12);

    let mut modes = BTreeSet::new();
    for (index, clip) in overlays.iter().enumerate() {
        assert!(matches!(&clip.source,
            ClipSource::Sequence { sequence_id } if sequence_id.as_str() == "seq_foreground"));
        assert_eq!(clip.record_range.start.value, index as i64 * 1_000);
        assert_eq!(clip.record_range.duration.value, 1_000);
        let visual = clip.visual.as_ref().expect("blend visual properties");
        assert!(matches!(visual.opacity, Animatable::Constant { value }
            if (value - 0.82).abs() < f64::EPSILON));
        modes.insert(blend_id(visual.compositing.blend_mode));
    }
    assert_eq!(modes.len(), 12);
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
            if visual.transform.shear.x != 0.0 || visual.transform.shear.y != 0.0 {
                found.insert("transform.shear".to_owned());
            }
            if (visual.transform.anchor.x - 0.5).abs() > f64::EPSILON
                || (visual.transform.anchor.y - 0.5).abs() > f64::EPSILON
            {
                found.insert("transform.pivot".to_owned());
            }
            if let Some(card) = &visual.card {
                found.insert("surface.card".to_owned());
                if card.shadow.is_some() {
                    found.insert("surface.shadow".to_owned());
                }
            }
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
