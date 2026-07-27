use std::collections::BTreeMap;

use veac_ir::*;

use crate::*;

use super::{header, output_material, time, visual};

pub(crate) fn matte_context(
    capability: Capability,
    artifact: &ProviderArtifact,
) -> ApplicationContext {
    let value = MatteApplication {
        header: header(match capability {
            Capability::Segmentation => "op_provider_segmentation",
            Capability::Matte => "op_provider_matte",
            _ => panic!("matte fixture requires segmentation or matte"),
        }),
        target_clip_id: ItemId::new("itm_video").unwrap(),
        matte: matte_output(artifact, "med_provider_matte", "itm_provider_matte"),
        mode: TrackMatteMode::Alpha,
        invert: false,
    };
    match capability {
        Capability::Segmentation => ApplicationContext::Segmentation(Box::new(value)),
        Capability::Matte => ApplicationContext::Matte(Box::new(value)),
        _ => unreachable!(),
    }
}

pub(crate) fn retouch_context(result: &RetouchResult) -> ApplicationContext {
    ApplicationContext::Retouch(Box::new(RetouchApplication {
        header: header("op_provider_retouch"),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        target_clip_id: ItemId::new("itm_video").unwrap(),
        record_range: TimeRange::new(time(0), time(100)).unwrap(),
        matte: result
            .masks
            .first()
            .map(|artifact| RetouchMatteApplication {
                insertion: matte_output(
                    artifact,
                    "med_provider_retouch_mask",
                    "itm_provider_retouch_mask",
                ),
                mode: TrackMatteMode::Alpha,
                invert: false,
            }),
        apply_id: ApplyId::new("apl_provider_retouch").unwrap(),
        apply_mix: ApplyMix::default(),
        time: super::time_binding(),
        effects: vec![RetouchEffectApplication {
            stage_id: ApplyStageId::new("aps_provider_retouch").unwrap(),
            active_range: None,
            effect: EffectInstance {
                id: EffectId::new("fx_provider_retouch").unwrap(),
                effect_type: "video.blur".into(),
                enabled: true,
                enable_range: None,
                parameters: BTreeMap::new(),
            },
        }],
        controls: vec![RetouchControlApplication {
            control: result.controls[0].parameter.clone(),
            effect_id: EffectId::new("fx_provider_retouch").unwrap(),
            effect_parameter: "radius".into(),
            keyframe_id_prefix: "kf_provider_retouch_smoothing".into(),
        }],
        before_apply_id: None,
        after_apply_id: None,
    }))
}

fn matte_output(
    artifact: &ProviderArtifact,
    material_id: &str,
    clip_id: &str,
) -> MatteClipInsertion {
    MatteClipInsertion {
        material: output_material(artifact, material_id, MaterialKind::Video, time(100)),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        track_id: TrackId::new("trk_visual").unwrap(),
        clip_id: ItemId::new(clip_id).unwrap(),
        record_range: TimeRange::new(time(0), time(100)).unwrap(),
        source_start: time(0),
        visual: visual(),
        before_id: None,
        after_id: None,
    }
}
