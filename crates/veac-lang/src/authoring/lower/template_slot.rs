use crate::authoring::{TemplateFillDecl, TemplateMediaKindDecl, TemplateSlotDecl};
use veac_ir::{FillMode, SlotConstraint, SlotKind};

use super::{context::Context, value};

pub(super) fn lower(
    ctx: &mut Context,
    declaration: Option<&TemplateSlotDecl>,
) -> Option<(Option<SlotConstraint>, bool)> {
    match declaration {
        None => Some((None, false)),
        Some(TemplateSlotDecl::Text { .. }) => Some((None, true)),
        Some(TemplateSlotDecl::Media {
            accepts,
            fill,
            label,
            minimum_source_duration,
            ..
        }) => {
            let min_source_duration = match minimum_source_duration {
                Some(duration) => Some(value::time(ctx, duration)?),
                None => None,
            };
            Some((
                Some(SlotConstraint {
                    kind: media_kind(accepts.value),
                    label: label.value.clone(),
                    fill: fill_mode(fill.value),
                    min_source_duration,
                }),
                false,
            ))
        }
    }
}

fn media_kind(value: TemplateMediaKindDecl) -> SlotKind {
    match value {
        TemplateMediaKindDecl::Video => SlotKind::Video,
        TemplateMediaKindDecl::Image => SlotKind::Image,
        TemplateMediaKindDecl::VideoOrImage => SlotKind::VideoOrImage,
    }
}

fn fill_mode(value: TemplateFillDecl) -> FillMode {
    match value {
        TemplateFillDecl::FitDuration => FillMode::FitDuration,
        TemplateFillDecl::TakeHead => FillMode::TakeHead,
        TemplateFillDecl::TakeCenter => FillMode::TakeCenter,
    }
}
