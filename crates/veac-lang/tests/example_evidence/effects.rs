use std::collections::BTreeSet;

use veac_ir::{Animatable, Effect, EffectKind, FitMode, ProjectEnvelope};

use crate::support::{
    assert_preview_evidence, authored_key, clips, preview_claims, sequence_by_key,
};

#[path = "effects/directional.rs"]
mod directional;

#[test]
fn preview_effect_rows_have_registry_instances_in_their_target() {
    assert_preview_evidence("effects.json", evidence);
}

#[test]
fn preview_effect_rows_name_the_matching_built_in_registry_key() {
    for claim in preview_claims("effects.json") {
        let key = claim
            .registry_key
            .unwrap_or_else(|| panic!("{} lacks registry_key", claim.id));
        assert!(
            EffectKind::from_type_name(&key).is_some(),
            "{} names unknown registry key {key}",
            claim.id
        );
        assert_eq!(
            EffectKind::from_type_name(&key).and_then(mechanism_id),
            Some(claim.id.as_str()),
            "{key}"
        );
    }
}

#[test]
fn video_effects_use_explicit_parameters_in_distinct_segments() {
    let envelope = crate::support::lower_example("video-effects/main.veac");
    let main = sequence_by_key(&envelope, "main");
    let clips = main
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .collect::<Vec<_>>();
    let segments = clips
        .iter()
        .copied()
        .filter(|clip| !clip.effects.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        segments
            .iter()
            .map(|clip| authored_key(&clip.authorship).unwrap())
            .collect::<Vec<_>>(),
        vec![
            "focus",
            "finish",
            "chroma",
            "luma",
            "stabilize-after",
            "plugin-after",
            "directional-horizontal",
            "directional-vertical",
            "directional-animated",
        ],
        "each effect stage needs its own attributable clip"
    );
    assert_eq!(
        segments
            .iter()
            .map(|clip| clip.effects.len())
            .collect::<Vec<_>>(),
        vec![1, 4, 2, 1, 1, 1, 1, 1, 1]
    );
    for id in [
        "focus",
        "finish",
        "chroma",
        "luma",
        "stabilize-before",
        "stabilize-after",
        "plugin-before",
        "plugin-after",
        "directional-horizontal",
        "directional-vertical",
        "directional-animated",
    ] {
        let frame = clips
            .iter()
            .find(|clip| authored_key(&clip.authorship) == Some(id))
            .and_then(|clip| clip.visual.as_ref()?.frame)
            .unwrap_or_else(|| panic!("{id} must fill the effects canvas"));
        assert_eq!((frame.width.value, frame.height.value), (640.0, 360.0));
        assert_eq!(frame.fit, FitMode::Fill);
    }
    for effect in segments.iter().flat_map(|clip| &clip.effects) {
        assert!(
            has_explicit_parameters(effect),
            "{} must not rely on an invisible default",
            effect.kind().type_name()
        );
    }
    assert!(matches!(
        &segments[0].effects[0].effect,
        Effect::VideoBlur { .. }
    ));

    let before = clips
        .iter()
        .find(|clip| authored_key(&clip.authorship) == Some("stabilize-before"))
        .expect("stabilize-before clip");
    let after = clips
        .iter()
        .find(|clip| authored_key(&clip.authorship) == Some("stabilize-after"))
        .expect("stabilize-after clip");
    assert_eq!(
        (
            before.record_range.start.value,
            before.record_range.duration.value
        ),
        (2_400, 1_200)
    );
    assert_eq!(
        (
            after.record_range.start.value,
            after.record_range.duration.value
        ),
        (3_600, 1_200)
    );
    assert_eq!(
        before.source, after.source,
        "comparison must use one source"
    );
    assert_eq!(
        before.source_mapping, after.source_mapping,
        "comparison must use the same source window and playback rate"
    );
    assert!(before.effects.is_empty());
    assert!(matches!(after.effects.as_slice(), [effect]
        if matches!(&effect.effect, Effect::VideoStabilize { enabled: true })));
    let plugin_before = clips
        .iter()
        .find(|clip| authored_key(&clip.authorship) == Some("plugin-before"))
        .expect("plugin color reference clip");
    let plugin_after = clips
        .iter()
        .find(|clip| authored_key(&clip.authorship) == Some("plugin-after"))
        .expect("plugin monochrome clip");
    assert_eq!(plugin_before.source, plugin_after.source);
    assert_eq!(plugin_before.source_mapping, plugin_after.source_mapping);
    assert!(plugin_before.effects.is_empty());
    assert!(matches!(plugin_after.effects.as_slice(), [effect]
    if matches!(&effect.effect, Effect::VideoPluginReferenceMonochromeV1 { amount, .. }
        if amount == &Animatable::constant(1.0))));
}

fn evidence(envelope: &ProjectEnvelope) -> BTreeSet<String> {
    clips(envelope)
        .flat_map(|clip| &clip.effects)
        .filter_map(|effect| mechanism_id(effect.kind()))
        .map(str::to_owned)
        .collect()
}

fn mechanism_id(kind: EffectKind) -> Option<&'static str> {
    Some(match kind {
        EffectKind::VideoColorAdjust => "effect.video.color-adjust",
        EffectKind::VideoBlur => "effect.video.blur",
        EffectKind::VideoDirectionalBlur => "effect.video.directional-blur",
        EffectKind::VideoSharpen => "effect.video.sharpen",
        EffectKind::VideoVignette => "effect.video.vignette",
        EffectKind::VideoGrain => "effect.video.grain",
        EffectKind::VideoChromaKey => "effect.video.chroma-key",
        EffectKind::VideoLumaKey => "effect.video.luma-key",
        EffectKind::VideoChromaSpill => "effect.video.chroma-spill",
        EffectKind::VideoStabilize => "effect.video.stabilize",
        EffectKind::VideoPluginReferenceMonochromeV1 => "effect.video.plugin-monochrome-v1",
        EffectKind::AudioNormalize => "effect.audio.normalize",
    })
}

fn has_explicit_parameters(instance: &veac_ir::EffectInstance) -> bool {
    if let Some(spec) = veac_ir::built_in_effect(instance.kind()) {
        return !spec.parameters.is_empty()
            && spec
                .parameters
                .iter()
                .all(|parameter| instance.effect.parameter(parameter.parameter).is_some());
    }
    matches!(
        &instance.effect,
        Effect::VideoPluginReferenceMonochromeV1 { .. }
    )
}
