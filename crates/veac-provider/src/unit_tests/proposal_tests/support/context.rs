use veac_ir::*;

use crate::*;

use super::{time, visual};

pub(crate) fn asr_context() -> ApplicationContext {
    ApplicationContext::AsrCaptions(Box::new(AsrCaptionApplication {
        header: header("op_provider_asr"),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        track_id: TrackId::new("trk_captions").unwrap(),
        style: TextStyle::default(),
        visual: visual(),
        item_id_prefix: "itm_provider_".into(),
    }))
}

pub(crate) fn translation_context() -> ApplicationContext {
    ApplicationContext::Translation(TranslationApplication {
        header: header("op_provider_translation"),
        bindings: vec![TranslationBinding {
            unit_id: "unit-1".into(),
            clip_id: ItemId::new("itm_text").unwrap(),
        }],
    })
}

pub(crate) fn transform_context(capability: Capability) -> ApplicationContext {
    let value = TransformApplication {
        header: header("op_provider_transform"),
        clip_id: ItemId::new(match capability {
            Capability::Stabilization => "itm_video",
            _ => "itm_visual",
        })
        .unwrap(),
        time: time_binding(),
        keyframe_id_prefix: "kf_provider".into(),
    };
    match capability {
        Capability::MotionTracking => ApplicationContext::MotionTracking(value),
        Capability::Stabilization => ApplicationContext::Stabilization(value),
        _ => panic!("transform fixture requires tracking or stabilization"),
    }
}

pub(crate) fn reframe_context() -> ApplicationContext {
    ApplicationContext::AutoReframe(AutoReframeApplication {
        header: header("op_provider_reframe"),
        clip_id: ItemId::new("itm_video").unwrap(),
        time: time_binding(),
        keyframe_id_prefix: "kf_reframe".into(),
    })
}

pub(crate) fn color_context() -> ApplicationContext {
    ApplicationContext::ColorMatch(ColorMatchApplication {
        header: header("op_provider_color"),
        clip_id: ItemId::new("itm_visual").unwrap(),
        color_space: ColorSpace {
            primaries: ColorPrimaries::Bt709,
            transfer: ColorTransfer::Bt709,
            matrix: ColorMatrix::Bt709,
            range: ColorRange::Limited,
        },
        effect_id: EffectId::new("fx_provider_color").unwrap(),
    })
}

pub(super) fn header(id: &str) -> ApplicationHeader {
    ApplicationHeader {
        project_revision: 7,
        operation_id: OperationId::new(id).unwrap(),
    }
}

pub(crate) fn header_mut(value: &mut ApplicationContext) -> &mut ApplicationHeader {
    match value {
        ApplicationContext::AsrCaptions(value) => &mut value.header,
        ApplicationContext::Translation(value) => &mut value.header,
        ApplicationContext::AnalysisAnnotations(value) => &mut value.header,
        ApplicationContext::TextToSpeech(value) => &mut value.header,
        ApplicationContext::Dubbing(value) => &mut value.header,
        ApplicationContext::MotionTracking(value) | ApplicationContext::Stabilization(value) => {
            &mut value.header
        }
        ApplicationContext::Segmentation(value) | ApplicationContext::Matte(value) => {
            &mut value.header
        }
        ApplicationContext::Denoise(value) | ApplicationContext::Removal(value) => {
            &mut value.header
        }
        ApplicationContext::VocalSeparation(value) => &mut value.header,
        ApplicationContext::AutoReframe(value) => &mut value.header,
        ApplicationContext::Retouch(value) => &mut value.header,
        ApplicationContext::ColorMatch(value) => &mut value.header,
        ApplicationContext::MulticamSync(value) => &mut value.header,
    }
}

pub(crate) fn time_binding() -> ClipTimeBinding {
    ClipTimeBinding {
        provider_origin: time(0),
        clip_local_origin: time(0),
    }
}
