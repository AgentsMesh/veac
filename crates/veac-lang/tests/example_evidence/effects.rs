use std::collections::BTreeSet;

use veac_ir::{FitMode, ParameterValue, ProjectEnvelope};

use crate::support::{assert_preview_evidence, clips, preview_claims};

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
            veac_ir::built_in_effect(&key).is_some(),
            "{} names unknown registry key {key}",
            claim.id
        );
        assert_eq!(mechanism_id(&key), Some(claim.id.as_str()), "{key}");
    }
}

#[test]
fn video_effects_use_explicit_parameters_in_distinct_segments() {
    let envelope = crate::support::lower_example("video-effects/main.veac");
    let main = envelope
        .project
        .sequences
        .iter()
        .find(|sequence| sequence.id.as_str() == "seq_main")
        .expect("video-effects main sequence");
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
            .map(|clip| clip.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "itm_focus",
            "itm_finish",
            "itm_chroma",
            "itm_luma",
            "itm_stabilize-after",
        ],
        "each effect stage needs its own attributable clip"
    );
    assert_eq!(
        segments
            .iter()
            .map(|clip| clip.effects.len())
            .collect::<Vec<_>>(),
        vec![1, 4, 2, 1, 1]
    );
    for id in [
        "itm_focus",
        "itm_finish",
        "itm_chroma",
        "itm_luma",
        "itm_stabilize-before",
        "itm_stabilize-after",
    ] {
        let frame = clips
            .iter()
            .find(|clip| clip.id.as_str() == id)
            .and_then(|clip| clip.visual.as_ref()?.frame)
            .unwrap_or_else(|| panic!("{id} must fill the effects canvas"));
        assert_eq!((frame.width.value, frame.height.value), (1280.0, 720.0));
        assert_eq!(frame.fit, FitMode::Fill);
    }
    for effect in segments.iter().flat_map(|clip| &clip.effects) {
        assert!(
            !effect.parameters.is_empty(),
            "{} must not rely on an invisible default",
            effect.effect_type
        );
    }
    assert!(matches!(
        segments[0].effects[0].parameters.get("radius"),
        Some(ParameterValue::NumberCurve { .. })
    ));

    let before = clips
        .iter()
        .find(|clip| clip.id.as_str() == "itm_stabilize-before")
        .expect("stabilize-before clip");
    let after = clips
        .iter()
        .find(|clip| clip.id.as_str() == "itm_stabilize-after")
        .expect("stabilize-after clip");
    assert_eq!(
        (
            before.record_range.start.value,
            before.record_range.duration.value
        ),
        (4_000, 2_000)
    );
    assert_eq!(
        (
            after.record_range.start.value,
            after.record_range.duration.value
        ),
        (6_000, 2_000)
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
        if effect.effect_type == "video.stabilize"
            && matches!(effect.parameters.get("enabled"),
                Some(ParameterValue::Boolean { value: true }))));
}

fn evidence(envelope: &ProjectEnvelope) -> BTreeSet<String> {
    clips(envelope)
        .flat_map(|clip| &clip.effects)
        .filter_map(|effect| mechanism_id(&effect.effect_type))
        .map(str::to_owned)
        .collect()
}

fn mechanism_id(effect_type: &str) -> Option<&'static str> {
    match effect_type {
        "video.color_adjust" => Some("effect.video.color-adjust"),
        "video.blur" => Some("effect.video.blur"),
        "video.sharpen" => Some("effect.video.sharpen"),
        "video.vignette" => Some("effect.video.vignette"),
        "video.grain" => Some("effect.video.grain"),
        "video.chroma_key" => Some("effect.video.chroma-key"),
        "video.luma_key" => Some("effect.video.luma-key"),
        "video.chroma_spill" => Some("effect.video.chroma-spill"),
        "video.stabilize" => Some("effect.video.stabilize"),
        "audio.normalize" => Some("effect.audio.normalize"),
        _ => None,
    }
}
