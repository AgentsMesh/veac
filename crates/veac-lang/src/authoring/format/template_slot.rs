use crate::authoring::{TemplateFillDecl, TemplateMediaKindDecl, TemplateSlotDecl};

use super::value::quoted;
use super::writer::Writer;

pub(super) fn template_slot(writer: &mut Writer, value: &TemplateSlotDecl) {
    match value {
        TemplateSlotDecl::Text { .. } => writer.line("template-slot text;"),
        TemplateSlotDecl::Media {
            accepts,
            fill,
            label,
            minimum_source_duration,
            ..
        } => writer.block("template-slot media", |writer| {
            writer.line(format!("accepts {};", accepts_name(accepts.value)));
            writer.line(format!("fill {};", fill_name(fill.value)));
            writer.line(format!("label {};", quoted(&label.value)));
            if let Some(value) = minimum_source_duration {
                writer.line(format!("minimum-source-duration {};", value.raw));
            }
        }),
    }
}

fn accepts_name(value: TemplateMediaKindDecl) -> &'static str {
    match value {
        TemplateMediaKindDecl::Video => "video",
        TemplateMediaKindDecl::Image => "image",
        TemplateMediaKindDecl::VideoOrImage => "video-or-image",
    }
}

fn fill_name(value: TemplateFillDecl) -> &'static str {
    match value {
        TemplateFillDecl::FitDuration => "fit-duration",
        TemplateFillDecl::TakeHead => "take-head",
        TemplateFillDecl::TakeCenter => "take-center",
    }
}
