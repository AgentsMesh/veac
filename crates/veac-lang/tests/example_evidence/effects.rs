use std::collections::BTreeSet;

use veac_ir::ProjectEnvelope;

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
        "audio.normalize" => Some("effect.audio.normalize"),
        _ => None,
    }
}
