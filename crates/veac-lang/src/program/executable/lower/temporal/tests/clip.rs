use super::*;
use crate::program::executable::{ClipTemporalProperty, TextTemporalProperty};
use veac_ir::{EffectParameter, ItemId};

#[test]
fn static_owners_and_effect_parameter_catalog_are_closed() {
    let project = fixture();
    let (picture, title, effect) = clip_ids(&project);

    let mut missing = project.clone();
    let target = Sink::clip(
        ItemId::new("itm_missing").unwrap(),
        ClipTemporalProperty::VisualPosition,
    );
    assert_reason(
        sink::attach(&mut missing, &target, binding()).unwrap_err(),
        "EXECUTABLE_TEMPORAL_SINK_MISSING",
    );

    let mut no_visual = project.clone();
    no_visual.sequences[0]
        .tracks
        .iter_mut()
        .flat_map(|track| &mut track.clips)
        .find(|clip| clip.id == title)
        .unwrap()
        .visual = None;
    let target = Sink::clip(title, ClipTemporalProperty::VisualPosition);
    assert_reason(
        sink::attach(&mut no_visual, &target, binding()).unwrap_err(),
        "EXECUTABLE_TEMPORAL_VISUAL_SINK",
    );

    let mut no_audio = project.clone();
    let target = Sink::clip(picture.clone(), ClipTemporalProperty::AudioGain);
    assert_reason(
        sink::attach(&mut no_audio, &target, binding()).unwrap_err(),
        "EXECUTABLE_TEMPORAL_AUDIO_SINK",
    );

    let mut wrong_parameter = project.clone();
    let target = Sink::ClipEffect {
        item_id: picture.clone(),
        effect_id: effect,
        parameter: EffectParameter::Brightness,
    };
    assert_reason(
        sink::attach(&mut wrong_parameter, &target, binding()).unwrap_err(),
        "EXECUTABLE_TEMPORAL_SINK_OWNER",
    );

    let mut wrong_text = project;
    let target = Sink::ClipText {
        item_id: picture,
        property: TextTemporalProperty::Position,
    };
    assert_reason(
        sink::attach(&mut wrong_text, &target, binding()).unwrap_err(),
        "EXECUTABLE_TEMPORAL_SINK_OWNER",
    );
}

fn clip_ids(project: &veac_ir::Project) -> (ItemId, ItemId, veac_ir::EffectId) {
    let clips = project.sequences[0]
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .collect::<Vec<_>>();
    let picture = clips.iter().find(|clip| !clip.effects.is_empty()).unwrap();
    let title = clips
        .iter()
        .find(|clip| matches!(&clip.source, veac_ir::ClipSource::Text { .. }))
        .unwrap();
    (
        picture.id.clone(),
        title.id.clone(),
        picture.effects[0].id.clone(),
    )
}
