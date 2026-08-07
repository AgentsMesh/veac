use veac_ir::{ApplyOperation, EffectParameter};
use veac_lang::program::build_source;

const SOURCE: &str = include_str!("../fixtures/component_sink_matrix.veac");

#[test]
fn every_owner_relative_sink_reaches_closed_canonical_temporal_ir() {
    let built = build_source(SOURCE).unwrap();
    let envelope = built.envelope();
    assert_eq!(envelope.temporal.bindings.len(), 19);
    let sequence = &envelope.project.sequences[0];
    let picture = &sequence.tracks[0].clips[0];
    assert_mask(&picture.visual.as_ref().unwrap().masks[0]);
    assert!(picture.effects[0]
        .effect
        .curve(EffectParameter::Radius)
        .is_some());
    let text = &sequence.tracks[1].clips[0];
    let veac_ir::ClipSource::Text { style, .. } = &text.source else {
        panic!("expected text source")
    };
    let animation = style.animation.as_ref().unwrap();
    assert!(animation.transform.position_offset.binding_id().is_some());
    assert!(animation.transform.scale.binding_id().is_some());
    assert!(animation.transform.rotation_degrees.binding_id().is_some());
    assert!(animation.reveal.binding_id().is_some());
    assert!(animation
        .highlight
        .as_ref()
        .unwrap()
        .progress
        .binding_id()
        .is_some());
    assert!(animation.opacity.binding_id().is_some());
    let apply = &sequence.applies[0];
    assert!(apply.mix.opacity.binding_id().is_some());
    assert_mask(&apply.mix.masks[0]);
    let ApplyOperation::Effect { effect } = &apply.stages[0].operation else {
        panic!("expected effect apply stage")
    };
    assert!(effect.effect.curve(EffectParameter::Radius).is_some());
    veac_ir::validate(envelope).unwrap();
}

fn assert_mask(mask: &veac_ir::Mask) {
    assert!(mask.position.binding_id().is_some());
    assert!(mask.scale.binding_id().is_some());
    assert!(mask.rotation_degrees.binding_id().is_some());
    assert!(mask.feather_pixels.binding_id().is_some());
    assert!(mask.expansion_pixels.binding_id().is_some());
}
