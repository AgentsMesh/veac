use veac_ir::{AnnotationTarget, ItemId};

use crate::*;

use super::{header, time};

pub(crate) fn analysis_context(capability: Capability) -> ApplicationContext {
    let timed = match capability {
        Capability::LanguageDetection => false,
        Capability::SceneDetection
        | Capability::BeatDetection
        | Capability::SilenceDetection
        | Capability::FillerDetection
        | Capability::HighlightDetection => true,
        _ => panic!("analysis annotation fixture requires an analysis capability"),
    };
    ApplicationContext::AnalysisAnnotations(AnnotationApplication {
        header: header("op_provider_annotation"),
        capability,
        target: if timed {
            AnnotationTarget::Clip {
                clip_id: ItemId::new("itm_visual").unwrap(),
            }
        } else {
            AnnotationTarget::Project
        },
        time: timed.then_some(AnnotationTimeBinding {
            provider_origin: time(0),
            target_origin: time(0),
        }),
        annotation_id_prefix: "ann_provider".into(),
    })
}
