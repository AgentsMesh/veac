use super::mechanism_helpers::text_clip;
use super::support::*;
use crate::{canonical::*, resolve, ResolvedClipSource};

#[test]
fn caption_speaker_survives_resolution_as_typed_plan_data() {
    let mut value = project();
    value.project.materials.push(font_material("med_font"));
    let mut clip = text_clip(true);
    let ClipSource::Caption { speaker, .. } = &mut clip.source else {
        panic!("caption fixture")
    };
    *speaker = Some("Narrator".to_owned());
    clip.effects.push(EffectInstance {
        id: EffectId::new("fx_caption_blur").unwrap(),
        effect_type: "video.blur".to_owned(),
        enabled: true,
        enable_range: None,
        parameters: BTreeMap::from([("radius".to_owned(), ParameterValue::Number { value: 2.0 })]),
    });
    value.project.sequences[0].tracks.push(track(
        "trk_captions",
        TrackKind::Caption,
        20,
        vec![clip],
    ));
    let plan = resolve(&value, None).unwrap().remove(0);
    let clip = &plan.sequences[0].tracks[1].clips[0];
    assert!(clip.visual.is_some());
    assert_eq!(clip.effects.len(), 1);
    let ResolvedClipSource::Caption { speaker, .. } = &clip.source else {
        panic!("resolved caption")
    };
    assert_eq!(speaker.as_deref(), Some("Narrator"));
}
use std::collections::BTreeMap;
